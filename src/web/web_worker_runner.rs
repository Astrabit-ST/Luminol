// Copyright (C) 2023 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.
use super::bindings;
use crate::prelude::*;
use eframe::{egui_wgpu, wgpu};
use wasm_bindgen::prelude::*;

#[derive(Debug, Default)]
struct Storage {}

// TODO: Implement saving and loading egui data
impl eframe::Storage for Storage {
    fn get_string(&self, key: &str) -> Option<String> {
        None
    }

    fn set_string(&mut self, key: &str, value: String) {}

    fn flush(&mut self) {}
}

pub struct WebWorkerRunnerEvent(WebWorkerRunnerEventInner);

enum WebWorkerRunnerEventInner {
    ScreenResize(u32, u32),
    Modifiers(egui::Modifiers),
}

struct WebWorkerRunnerState {
    app: Box<dyn CustomApp>,
    render_state: egui_wgpu::RenderState,
    canvas: web_sys::OffscreenCanvas,
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    modifiers: egui::Modifiers,
    native_pixels_per_point: Option<f32>,

    event_rx: Option<mpsc::UnboundedReceiver<egui::Event>>,
    custom_event_rx: Option<mpsc::UnboundedReceiver<WebWorkerRunnerEvent>>,
}

/// A runner for wgpu egui applications intended to be run in a web worker.
/// Currently only targets WebGPU, not WebGL.
#[derive(Clone)]
pub struct WebWorkerRunner {
    state: std::rc::Rc<std::cell::RefCell<WebWorkerRunnerState>>,
    context: egui::Context,
    time_lock: Arc<RwLock<f64>>,
}

impl WebWorkerRunner {
    /// Creates a new `WebWorkerRunner` to render onto the given `OffscreenCanvas` with the
    /// given configuration options.
    ///
    /// This function MUST be run in a web worker.
    pub async fn new(
        app_creator: Box<dyn FnOnce(&eframe::CreationContext<'_>) -> Box<dyn CustomApp>>,
        canvas: web_sys::OffscreenCanvas,
        web_options: eframe::WebOptions,
        device_pixel_ratio: f32,
        prefers_color_scheme_dark: Option<bool>,
        event_rx: Option<mpsc::UnboundedReceiver<egui::Event>>,
        custom_event_rx: Option<mpsc::UnboundedReceiver<WebWorkerRunnerEvent>>,
    ) -> Self {
        let Some(worker) = bindings::worker() else {
            panic!("cannot use `WebWorkerRunner::new()` outside of a web worker");
        };

        let time_lock = Arc::new(RwLock::new(0.));

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: web_options.wgpu_options.supported_backends,
            dx12_shader_compiler: Default::default(),
        });
        let surface = instance
            .create_surface_from_offscreen_canvas(canvas.clone())
            .unwrap_or_else(|e| panic!("failed to create surface: {e}"));

        let depth_format = egui_wgpu::depth_format_from_bits(0, 0);
        let render_state = egui_wgpu::RenderState::create(
            &web_options.wgpu_options,
            &instance,
            &surface,
            depth_format,
            1,
        )
        .await
        .unwrap_or_else(|e| panic!("failed to initialize renderer: {e}"));

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_state.target_format,
            present_mode: web_options.wgpu_options.present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![render_state.target_format],
            width: canvas.width(),
            height: canvas.height(),
        };
        surface.configure(&render_state.device, &surface_configuration);

        let location = worker.location();
        let native_pixels_per_point = if device_pixel_ratio > 0. && device_pixel_ratio.is_finite() {
            Some(device_pixel_ratio)
        } else {
            None
        };
        let integration_info = eframe::IntegrationInfo {
            system_theme: if web_options.follow_system_theme {
                prefers_color_scheme_dark.map(|x| {
                    if x {
                        eframe::Theme::Dark
                    } else {
                        eframe::Theme::Light
                    }
                })
            } else {
                None
            },
            web_info: eframe::WebInfo {
                user_agent: worker.navigator().user_agent().unwrap_or_default(),
                location: eframe::Location {
                    url: location
                        .href()
                        .strip_suffix("/worker.js")
                        .unwrap_or(location.href().as_str())
                        .to_string(),
                    protocol: location.protocol(),
                    host: location.host(),
                    hostname: location.hostname(),
                    port: location.port(),
                    hash: Default::default(),
                    query: Default::default(),
                    query_map: Default::default(),
                    origin: location.origin(),
                },
            },
            native_pixels_per_point,
            cpu_usage: None,
        };

        let context = egui::Context::default();
        context.set_os(egui::os::OperatingSystem::from_user_agent(
            integration_info.web_info.user_agent.as_str(),
        ));
        context.set_visuals(
            integration_info
                .system_theme
                .unwrap_or(web_options.default_theme)
                .egui_visuals(),
        );
        {
            let time_lock = time_lock.clone();
            context.set_request_repaint_callback(move |i| {
                *time_lock.write() = bindings::performance(&bindings::worker().unwrap()).now()
                    / 1000.
                    + i.after.as_secs_f64();
            });
        }

        Self {
            state: std::rc::Rc::new(std::cell::RefCell::new(WebWorkerRunnerState {
                app: app_creator(&eframe::CreationContext {
                    egui_ctx: context.clone(),
                    integration_info: integration_info.clone(),
                    storage: Some(&Storage::default()),
                    wgpu_render_state: Some(render_state.clone()),
                }),
                render_state,
                canvas,
                surface,
                surface_configuration,
                modifiers: Default::default(),
                native_pixels_per_point,
                event_rx,
                custom_event_rx,
            })),
            context,
            time_lock,
        }
    }

    /// Sets up the hook to render the app to the canvas.
    pub fn setup_render_hooks(self) {
        let callback = Closure::once(move || {
            let mut state = self.state.borrow_mut();
            let worker = bindings::worker().unwrap();

            let mut width = state.surface_configuration.width;
            let mut height = state.surface_configuration.height;
            let mut modifiers = state.modifiers;

            if let Some(custom_event_rx) = &mut state.custom_event_rx {
                for event in std::iter::from_fn(|| custom_event_rx.try_recv().ok()) {
                    match event.0 {
                        WebWorkerRunnerEventInner::ScreenResize(new_width, new_height) => {
                            width = new_width;
                            height = new_height;
                        }

                        WebWorkerRunnerEventInner::Modifiers(new_modifiers) => {
                            modifiers = new_modifiers;
                        }
                    }
                }
            }

            // Resize the canvas if the screen size has changed
            if width != state.surface_configuration.width
                || height != state.surface_configuration.height
            {
                state.canvas.set_width(state.surface_configuration.width);
                state.canvas.set_height(state.surface_configuration.height);
                state.surface_configuration.width = width;
                state.surface_configuration.height = height;
                state
                    .surface
                    .configure(&state.render_state.device, &state.surface_configuration);

                // Also trigger a rerender immediately
                *self.time_lock.write() = 0.;
            }

            // If the modifiers have changed, trigger a rerender
            if modifiers != state.modifiers {
                state.modifiers = modifiers;
                *self.time_lock.write() = 0.;
            }

            let events = if let Some(event_rx) = &mut state.event_rx {
                std::iter::from_fn(|| event_rx.try_recv().ok()).collect_vec()
            } else {
                Default::default()
            };
            if !events.is_empty() {
                // Render immediately if there are any pending events
                *self.time_lock.write() = 0.;
            }

            // Render only if sufficient time has passed since the last render
            if bindings::performance(&worker).now() / 1000. >= *self.time_lock.read() {
                // Ask the app to paint the next frame
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_max(
                        egui::pos2(0., 0.),
                        egui::pos2(
                            state.surface_configuration.width as f32,
                            state.surface_configuration.height as f32,
                        ),
                    )),
                    pixels_per_point: state.native_pixels_per_point,
                    time: Some(bindings::performance(&worker).now() / 1000.),
                    max_texture_side: Some(
                        state.render_state.device.limits().max_texture_dimension_2d as usize,
                    ),
                    events,
                    modifiers,
                    ..Default::default()
                };
                let output = self
                    .context
                    .run(input, |_| state.app.custom_update(&self.context));
                let clear_color = state.app.clear_color(&self.context.style().visuals);
                let paint_jobs = self.context.tessellate(output.shapes);

                let mut encoder = state.render_state.device.create_command_encoder(
                    &wgpu::CommandEncoderDescriptor {
                        label: Some("Luminol WebWorkerRunner Encoder"),
                    },
                );
                let screen_descriptor = egui_wgpu::renderer::ScreenDescriptor {
                    size_in_pixels: [
                        state.surface_configuration.width,
                        state.surface_configuration.height,
                    ],
                    pixels_per_point: state.native_pixels_per_point.unwrap_or(1.),
                };

                // Upload textures to GPU that are newly created in the current frame
                let command_buffers = {
                    let mut renderer = state.render_state.renderer.write();
                    for (id, delta) in output.textures_delta.set.iter() {
                        renderer.update_texture(
                            &state.render_state.device,
                            &state.render_state.queue,
                            *id,
                            delta,
                        );
                    }
                    renderer.update_buffers(
                        &state.render_state.device,
                        &state.render_state.queue,
                        &mut encoder,
                        &paint_jobs[..],
                        &screen_descriptor,
                    )
                };

                // Execute egui's render pass
                {
                    let renderer = state.render_state.renderer.read();
                    let view = state
                        .surface
                        .get_current_texture()
                        .unwrap()
                        .texture
                        .create_view(&Default::default());
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: clear_color[0].into(),
                                    g: clear_color[1].into(),
                                    b: clear_color[2].into(),
                                    a: clear_color[3].into(),
                                }),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                        label: Some("Luminol WebWorkerRunner Renderer"),
                    });
                    renderer.render(&mut render_pass, &paint_jobs[..], &screen_descriptor);
                }

                // Remove textures that are no longer needed after this frame
                {
                    let mut renderer = state.render_state.renderer.write();
                    for id in output.textures_delta.free.iter() {
                        renderer.free_texture(id);
                    }
                }

                // Copy from the internal drawing buffer onto the HTML canvas
                state.render_state.queue.submit(
                    command_buffers
                        .into_iter()
                        .chain(std::iter::once(encoder.finish())),
                );
                state.surface.get_current_texture().unwrap().present();

                *self.time_lock.write() = bindings::performance(&worker).now() / 1000.
                    + output.repaint_after.as_secs_f64();
            }

            self.clone().setup_render_hooks();
        });

        let _ = bindings::worker()
            .unwrap()
            .request_animation_frame(callback.as_ref().unchecked_ref());
        callback.forget();
    }
}

/// Sets up the event listeners on the main thread in order to do things like respond to
/// mouse events and resize the canvas to fill the screen.
pub fn setup_main_thread_hooks(
    canvas: web_sys::HtmlCanvasElement,
    event_tx: mpsc::UnboundedSender<egui::Event>,
    custom_event_tx: mpsc::UnboundedSender<WebWorkerRunnerEvent>,
) {
    let window =
        web_sys::window().expect("cannot run `setup_main_thread_hooks()` outside of main thread");
    let document = window.document().unwrap();

    let is_mac = matches!(
        egui::os::OperatingSystem::from_user_agent(
            window.navigator().user_agent().unwrap_or_default().as_str()
        ),
        egui::os::OperatingSystem::Mac | egui::os::OperatingSystem::IOS
    );

    {
        let custom_event_tx = custom_event_tx.clone();
        let f = {
            let window = window.clone();
            move || {
                let _ = custom_event_tx.send(WebWorkerRunnerEvent(
                    WebWorkerRunnerEventInner::ScreenResize(
                        window.inner_width().unwrap().as_f64().unwrap() as u32,
                        window.inner_height().unwrap().as_f64().unwrap() as u32,
                    ),
                ));
            }
        };
        f();
        let callback: Closure<dyn Fn()> = Closure::new(f);
        window
            .add_event_listener_with_callback("resize", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for screen resizing");
        callback.forget();
    }

    {
        let f = |pressed| {
            let event_tx = event_tx.clone();
            let custom_event_tx = custom_event_tx.clone();
            move |e: web_sys::MouseEvent| {
                let ctrl = e.ctrl_key();
                let modifiers = egui::Modifiers {
                    alt: e.alt_key(),
                    ctrl: !is_mac && ctrl,
                    shift: e.shift_key(),
                    mac_cmd: is_mac && ctrl,
                    command: ctrl,
                };
                let _ = custom_event_tx.send(WebWorkerRunnerEvent(
                    WebWorkerRunnerEventInner::Modifiers(modifiers),
                ));
                if let Some(button) = match e.button() {
                    0 => Some(egui::PointerButton::Primary),
                    1 => Some(egui::PointerButton::Middle),
                    2 => Some(egui::PointerButton::Secondary),
                    3 => Some(egui::PointerButton::Extra1),
                    4 => Some(egui::PointerButton::Extra2),
                    _ => None,
                } {
                    let _ = event_tx.send(egui::Event::PointerButton {
                        pos: egui::pos2(e.client_x() as f32, e.client_y() as f32),
                        button,
                        pressed,
                        modifiers,
                    });
                }
                e.stop_propagation();
                if !pressed {
                    e.prevent_default();
                }
            }
        };

        let callback: Closure<dyn Fn(_)> = Closure::new(f(true));
        canvas
            .add_event_listener_with_callback("mousedown", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for mouse button presses");
        callback.forget();

        let callback: Closure<dyn Fn(_)> = Closure::new(f(false));
        canvas
            .add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for mouse button releases");
        callback.forget();
    }

    {
        let event_tx = event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::MouseEvent| {
            let _ = event_tx.send(egui::Event::PointerMoved(egui::pos2(
                e.client_x() as f32,
                e.client_y() as f32,
            )));
            e.stop_propagation();
            e.prevent_default();
        });
        canvas
            .add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for mouse movement");
        callback.forget();
    }

    {
        let event_tx = event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::MouseEvent| {
            let _ = event_tx.send(egui::Event::PointerGone);
            e.stop_propagation();
            e.prevent_default();
        });
        canvas
            .add_event_listener_with_callback("mouseleave", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for mouse leaving");
        callback.forget();
    }

    {
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::Event| {
            e.prevent_default();
        });
        canvas
            .add_event_listener_with_callback("contextmenu", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for context menu");
        canvas
            .add_event_listener_with_callback("afterprint", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for print shortcut keypress");
        callback.forget();
    }

    {
        let f = |pressed| {
            let event_tx = event_tx.clone();
            let custom_event_tx = custom_event_tx.clone();
            move |e: web_sys::KeyboardEvent| {
                let ctrl = e.ctrl_key();
                let modifiers = egui::Modifiers {
                    alt: e.alt_key(),
                    ctrl: !is_mac && ctrl,
                    shift: e.shift_key(),
                    mac_cmd: is_mac && ctrl,
                    command: ctrl,
                };
                let _ = custom_event_tx.send(WebWorkerRunnerEvent(
                    WebWorkerRunnerEventInner::Modifiers(modifiers),
                ));
                if e.is_composing() || e.key_code() == 229 {
                    return;
                }
                if let Some(key) = match e.key().as_str() {
                    // https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values
                    "Enter" => Some(egui::Key::Enter),
                    "Tab" => Some(egui::Key::Tab),
                    " " => Some(egui::Key::Space),

                    "ArrowDown" => Some(egui::Key::ArrowDown),
                    "ArrowLeft" => Some(egui::Key::ArrowLeft),
                    "ArrowRight" => Some(egui::Key::ArrowRight),
                    "ArrowUp" => Some(egui::Key::ArrowUp),
                    "End" => Some(egui::Key::End),
                    "Home" => Some(egui::Key::Home),
                    "PageDown" => Some(egui::Key::PageDown),
                    "PageUp" => Some(egui::Key::PageUp),

                    "Backspace" => Some(egui::Key::Backspace),
                    "Delete" => Some(egui::Key::Delete),
                    "Insert" => Some(egui::Key::Insert),

                    "Escape" => Some(egui::Key::Escape),

                    "F1" => Some(egui::Key::F1),
                    "F2" => Some(egui::Key::F2),
                    "F3" => Some(egui::Key::F3),
                    "F4" => Some(egui::Key::F4),
                    "F5" => Some(egui::Key::F5),
                    "F6" => Some(egui::Key::F6),
                    "F7" => Some(egui::Key::F7),
                    "F8" => Some(egui::Key::F8),
                    "F9" => Some(egui::Key::F9),
                    "F10" => Some(egui::Key::F10),
                    "F11" => Some(egui::Key::F11),
                    "F12" => Some(egui::Key::F12),
                    "F13" => Some(egui::Key::F13),
                    "F14" => Some(egui::Key::F14),
                    "F15" => Some(egui::Key::F15),
                    "F16" => Some(egui::Key::F16),
                    "F17" => Some(egui::Key::F17),
                    "F18" => Some(egui::Key::F18),
                    "F19" => Some(egui::Key::F19),
                    "F20" => Some(egui::Key::F20),

                    "-" => Some(egui::Key::Minus),
                    "+" | "=" => Some(egui::Key::PlusEquals),

                    "0" => Some(egui::Key::Num0),
                    "1" => Some(egui::Key::Num1),
                    "2" => Some(egui::Key::Num2),
                    "3" => Some(egui::Key::Num3),
                    "4" => Some(egui::Key::Num4),
                    "5" => Some(egui::Key::Num5),
                    "6" => Some(egui::Key::Num6),
                    "7" => Some(egui::Key::Num7),
                    "8" => Some(egui::Key::Num8),
                    "9" => Some(egui::Key::Num9),

                    "A" | "a" => Some(egui::Key::A),
                    "B" | "b" => Some(egui::Key::B),
                    "C" | "c" => Some(egui::Key::C),
                    "D" | "d" => Some(egui::Key::D),
                    "E" | "e" => Some(egui::Key::E),
                    "F" | "f" => Some(egui::Key::F),
                    "G" | "g" => Some(egui::Key::G),
                    "H" | "h" => Some(egui::Key::H),
                    "I" | "i" => Some(egui::Key::I),
                    "J" | "j" => Some(egui::Key::J),
                    "K" | "k" => Some(egui::Key::K),
                    "L" | "l" => Some(egui::Key::L),
                    "M" | "m" => Some(egui::Key::M),
                    "N" | "n" => Some(egui::Key::N),
                    "O" | "o" => Some(egui::Key::O),
                    "P" | "p" => Some(egui::Key::P),
                    "Q" | "q" => Some(egui::Key::Q),
                    "R" | "r" => Some(egui::Key::R),
                    "S" | "s" => Some(egui::Key::S),
                    "T" | "t" => Some(egui::Key::T),
                    "U" | "u" => Some(egui::Key::U),
                    "V" | "v" => Some(egui::Key::V),
                    "W" | "w" => Some(egui::Key::W),
                    "X" | "x" => Some(egui::Key::X),
                    "Y" | "y" => Some(egui::Key::Y),
                    "Z" | "z" => Some(egui::Key::Z),

                    _ => None,
                } {
                    let _ = event_tx.send(egui::Event::Key {
                        key,
                        pressed,
                        repeat: pressed,
                        modifiers,
                    });
                    if pressed
                        && (matches!(
                            key,
                            egui::Key::Tab
                                | egui::Key::Backspace
                                | egui::Key::ArrowDown
                                | egui::Key::ArrowLeft
                                | egui::Key::ArrowRight
                                | egui::Key::ArrowUp
                        ) || (ctrl && matches!(key, egui::Key::P | egui::Key::S)))
                    {
                        e.prevent_default();
                    }
                }
            }
        };

        let callback: Closure<dyn Fn(_)> = Closure::new(f(true));
        document
            .add_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for keyboard key presses");
        callback.forget();

        let callback: Closure<dyn Fn(_)> = Closure::new(f(false));
        document
            .add_event_listener_with_callback("keyup", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for keyboard key releases");
        callback.forget();
    }
}
