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

static PANIC_LOCK: OnceCell<()> = OnceCell::new();

#[derive(Debug, Default)]
struct Storage {
    output_tx: Option<mpsc::UnboundedSender<WebWorkerRunnerOutput>>,
}

impl eframe::Storage for Storage {
    fn get_string(&self, key: &str) -> Option<String> {
        if let Some(output_tx) = &self.output_tx {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            output_tx
                .send(WebWorkerRunnerOutput(
                    WebWorkerRunnerOutputInner::StorageGet(key.to_string(), oneshot_tx),
                ))
                .unwrap();
            oneshot_rx.blocking_recv().unwrap()
        } else {
            None
        }
    }

    fn set_string(&mut self, key: &str, value: String) {
        if let Some(output_tx) = &self.output_tx {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            output_tx
                .send(WebWorkerRunnerOutput(
                    WebWorkerRunnerOutputInner::StorageSet(
                        key.to_string(),
                        value.to_string(),
                        oneshot_tx,
                    ),
                ))
                .unwrap();
            if !oneshot_rx.blocking_recv().unwrap() {
                tracing::warn!("Failed to save to local storage key {key}");
            }
        }
    }

    fn flush(&mut self) {}
}

pub struct WebWorkerRunnerEvent(WebWorkerRunnerEventInner);

enum WebWorkerRunnerEventInner {
    /// (window.innerWidth, window.innerHeight, window.devicePixelRatio)
    ScreenResize(u32, u32, f32),
    /// This should be sent whenever the modifiers change
    Modifiers(egui::Modifiers),
    /// This should be sent whenever the app needs to save immediately
    Save,
}

pub struct WebWorkerRunnerOutput(WebWorkerRunnerOutputInner);

enum WebWorkerRunnerOutputInner {
    PlatformOutput(egui::PlatformOutput),
    StorageGet(String, oneshot::Sender<Option<String>>),
    StorageSet(String, String, oneshot::Sender<bool>),
}

struct WebWorkerRunnerState {
    app: Box<dyn CustomApp>,
    app_id: String,
    save_time: f64,
    render_state: egui_wgpu::RenderState,
    canvas: web_sys::OffscreenCanvas,
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    modifiers: egui::Modifiers,

    /// Width of the canvas in points. `surface_configuration.width` is the width in pixels.
    width: u32,
    /// Height of the canvas in points. `surface_configuration.height` is the height in pixels.
    height: u32,
    /// Length of a pixel divided by length of a point.
    pixel_ratio: f32,

    event_rx: Option<mpsc::UnboundedReceiver<egui::Event>>,
    custom_event_rx: Option<mpsc::UnboundedReceiver<WebWorkerRunnerEvent>>,
    output_tx: Option<mpsc::UnboundedSender<WebWorkerRunnerOutput>>,
}

/// A runner for wgpu egui applications intended to be run in a web worker.
/// Currently only targets WebGPU, not WebGL.
#[derive(Clone)]
pub struct WebWorkerRunner {
    state: std::rc::Rc<std::cell::RefCell<WebWorkerRunnerState>>,
    storage: std::rc::Rc<std::cell::RefCell<Storage>>,
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
        app_id: &str,
        prefers_color_scheme_dark: Option<bool>,
        event_rx: Option<mpsc::UnboundedReceiver<egui::Event>>,
        custom_event_rx: Option<mpsc::UnboundedReceiver<WebWorkerRunnerEvent>>,
        output_tx: Option<mpsc::UnboundedSender<WebWorkerRunnerOutput>>,
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

        let width = canvas.width();
        let height = canvas.height();
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: render_state.target_format,
            present_mode: web_options.wgpu_options.present_mode,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![render_state.target_format],
            width,
            height,
        };
        surface.configure(&render_state.device, &surface_configuration);

        let location = worker.location();
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
            native_pixels_per_point: Some(1.),
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

        if let Some(output_tx) = &output_tx {
            let (oneshot_tx, oneshot_rx) = oneshot::channel();
            output_tx
                .send(WebWorkerRunnerOutput(
                    WebWorkerRunnerOutputInner::StorageGet(app_id.to_string(), oneshot_tx),
                ))
                .unwrap();
            if let Some(memory) = oneshot_rx.await.ok().flatten() {
                match ron::from_str(&memory) {
                    Ok(memory) => {
                        context.memory_mut(|m| *m = memory);
                        tracing::info!("Successfully restored memory for {app_id}");
                    }
                    Err(e) => tracing::warn!("Failed to restore memory for {app_id}: {e}"),
                }
            } else {
                tracing::warn!("No memory found for {app_id}");
            }
        }

        let storage = Storage {
            output_tx: output_tx.clone(),
        };

        Self {
            state: std::rc::Rc::new(std::cell::RefCell::new(WebWorkerRunnerState {
                app: app_creator(&eframe::CreationContext {
                    egui_ctx: context.clone(),
                    integration_info: integration_info.clone(),
                    wgpu_render_state: Some(render_state.clone()),
                    storage: Some(&storage),
                }),
                app_id: app_id.to_string(),
                save_time: 0.,
                render_state,
                canvas,
                surface,
                surface_configuration,
                modifiers: Default::default(),
                width,
                height,
                pixel_ratio: 1.,
                event_rx,
                custom_event_rx,
                output_tx,
            })),
            storage: std::rc::Rc::new(std::cell::RefCell::new(storage)),
            context,
            time_lock,
        }
    }

    /// Sets up the hook to render the app to the canvas.
    pub fn setup_render_hooks(self) {
        let callback = Closure::once(move || {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            let mut state = self.state.borrow_mut();
            let worker = bindings::worker().unwrap();

            let mut width = state.width;
            let mut height = state.height;
            let mut pixel_ratio = state.pixel_ratio;
            let mut modifiers = state.modifiers;
            let mut should_save = false;

            let now = bindings::performance(&worker).now() / 1000.;

            if let Some(custom_event_rx) = &mut state.custom_event_rx {
                for event in std::iter::from_fn(|| custom_event_rx.try_recv().ok()) {
                    match event.0 {
                        WebWorkerRunnerEventInner::ScreenResize(
                            new_width,
                            new_height,
                            new_pixel_ratio,
                        ) => {
                            width = new_width;
                            height = new_height;
                            pixel_ratio = new_pixel_ratio;
                        }

                        WebWorkerRunnerEventInner::Modifiers(new_modifiers) => {
                            modifiers = new_modifiers;
                        }

                        WebWorkerRunnerEventInner::Save => {
                            should_save = true;
                        }
                    }
                }
            }

            if should_save || now >= state.save_time + state.app.auto_save_interval().as_secs_f64()
            {
                state.save_time = now;
                state.app.save(&mut *self.storage.borrow_mut());
                if let Some(output_tx) = &state.output_tx {
                    match self.context.memory(|memory| ron::to_string(memory)) {
                        Ok(ron) => {
                            let (oneshot_tx, oneshot_rx) = oneshot::channel();
                            output_tx
                                .send(WebWorkerRunnerOutput(
                                    WebWorkerRunnerOutputInner::StorageSet(
                                        state.app_id.clone(),
                                        ron,
                                        oneshot_tx,
                                    ),
                                ))
                                .unwrap();
                            if !oneshot_rx.blocking_recv().unwrap() {
                                tracing::warn!("Failed to save memory for {}", state.app_id);
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to serialize memory for {}: {e}", state.app_id)
                        }
                    }
                }
            }

            // Resize the canvas if the screen size has changed
            if width != state.width || height != state.height {
                state.pixel_ratio = pixel_ratio;
                state.width = width;
                state.height = height;
                state.surface_configuration.width = (width as f32 * pixel_ratio).round() as u32;
                state.surface_configuration.height = (height as f32 * pixel_ratio).round() as u32;
                state.canvas.set_width(state.surface_configuration.width);
                state.canvas.set_height(state.surface_configuration.height);
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
            if now >= *self.time_lock.read() {
                // Ask the app to paint the next frame
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_max(
                        egui::pos2(0., 0.),
                        egui::pos2(state.width as f32, state.height as f32),
                    )),
                    pixels_per_point: Some(state.pixel_ratio),
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
                if let Some(output_tx) = &state.output_tx {
                    let _ = output_tx.send(WebWorkerRunnerOutput(
                        WebWorkerRunnerOutputInner::PlatformOutput(output.platform_output),
                    ));
                }
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
                    pixels_per_point: state.pixel_ratio,
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
    mut output_rx: mpsc::UnboundedReceiver<WebWorkerRunnerOutput>,
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
        let f = {
            let custom_event_tx = custom_event_tx.clone();
            let window = window.clone();
            let canvas_id = canvas.id();
            move || {
                if PANIC_LOCK.get().is_some() {
                    return;
                }
                let pixel_ratio = window.device_pixel_ratio();
                let pixel_ratio = if pixel_ratio > 0. && pixel_ratio.is_finite() {
                    pixel_ratio as f32
                } else {
                    1.
                };
                let _ = custom_event_tx.send(WebWorkerRunnerEvent(
                    WebWorkerRunnerEventInner::ScreenResize(
                        window.inner_width().unwrap().as_f64().unwrap() as u32,
                        window.inner_height().unwrap().as_f64().unwrap() as u32,
                        pixel_ratio,
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
                if PANIC_LOCK.get().is_some() {
                    return;
                }
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
            if PANIC_LOCK.get().is_some() {
                return;
            }
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
        let custom_event_tx = custom_event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::MouseEvent| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            let _ = custom_event_tx.send(WebWorkerRunnerEvent(WebWorkerRunnerEventInner::Save));
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
        let window = window.clone();
        let event_tx = event_tx.clone();
        let custom_event_tx = custom_event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::WheelEvent| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
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

            let unit = match e.delta_mode() {
                web_sys::WheelEvent::DOM_DELTA_LINE => egui::MouseWheelUnit::Line,
                web_sys::WheelEvent::DOM_DELTA_PAGE => egui::MouseWheelUnit::Page,
                _ => egui::MouseWheelUnit::Point,
            };
            let delta = -egui::vec2(e.delta_x() as f32, e.delta_y() as f32);
            let _ = event_tx.send(egui::Event::MouseWheel {
                unit,
                delta,
                modifiers,
            });

            let delta = delta
                * match unit {
                    egui::MouseWheelUnit::Point => 1.,
                    egui::MouseWheelUnit::Line => 8.,
                    egui::MouseWheelUnit::Page => {
                        window.inner_height().unwrap().as_f64().unwrap() as f32
                    }
                };
            let _ = if ctrl {
                event_tx.send(egui::Event::Zoom((delta.y / 200.).exp()))
            } else if modifiers.shift {
                event_tx.send(egui::Event::Scroll(egui::vec2(delta.x + delta.y, 0.)))
            } else {
                event_tx.send(egui::Event::Scroll(delta))
            };

            e.stop_propagation();
            e.prevent_default();
        });
        canvas
            .add_event_listener_with_callback("wheel", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for mouse scrolling");
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
                if PANIC_LOCK.get().is_some() {
                    return;
                }
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
                let key = e.key();
                let matched_key = match key.as_str() {
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
                };
                if pressed && !ctrl && key.len() == 1 {
                    let _ = event_tx.send(egui::Event::Text(key));
                }
                if let Some(key) = matched_key {
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

    {
        let event_tx = event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::ClipboardEvent| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            if let Some(data) = e.clipboard_data() {
                if let Ok(text) = data.get_data("text") {
                    if !text.is_empty() {
                        let _ = event_tx.send(egui::Event::Paste(text.replace("\r\n", "\n")));
                    }
                }
            }
            e.stop_propagation();
            e.prevent_default();
        });
        document
            .add_event_listener_with_callback("paste", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for clipboard pasting");
        callback.forget();
    }

    {
        let event_tx = event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |_: web_sys::ClipboardEvent| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            let _ = event_tx.send(egui::Event::Copy);
        });
        document
            .add_event_listener_with_callback("copy", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for clipboard copying");
        callback.forget();
    }

    {
        let event_tx = event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |_: web_sys::ClipboardEvent| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            let _ = event_tx.send(egui::Event::Cut);
        });
        document
            .add_event_listener_with_callback("cut", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for clipboard cutting");
        callback.forget();
    }

    {
        let custom_event_tx = custom_event_tx.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |_: web_sys::Event| {
            let _ = custom_event_tx.send(WebWorkerRunnerEvent(WebWorkerRunnerEventInner::Save));
        });
        document
            .add_event_listener_with_callback("onbeforeunload", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for window unloading");
        document
            .add_event_listener_with_callback("blur", callback.as_ref().unchecked_ref())
            .expect("failed to register event listener for window blur");
        callback.forget();
    }

    {
        // The canvas automatically resizes itself whenever a frame is drawn.
        // The resizing does not take window.devicePixelRatio into account,
        // so this mutation observer is to detect canvas resizes and correct them.
        let window = window.clone();
        let callback: Closure<dyn Fn(_)> = Closure::new(move |mutations: js_sys::Array| {
            if PANIC_LOCK.get().is_some() {
                return;
            }
            let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
            let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
            mutations.for_each(&mut |mutation, _, _| {
                let mutation = mutation.unchecked_into::<web_sys::MutationRecord>();
                if mutation.type_().as_str() == "attributes" {
                    let canvas = mutation
                        .target()
                        .unwrap()
                        .unchecked_into::<web_sys::HtmlCanvasElement>();
                    if canvas.width() != width || canvas.height() != height {
                        let _ = canvas.set_attribute("width", width.to_string().as_str());
                        let _ = canvas.set_attribute("height", height.to_string().as_str());
                    }
                }
            });
        });
        let observer = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref())
            .expect("failed to create canvas mutation observer");
        let mut options = web_sys::MutationObserverInit::new();
        options.attributes(true);
        observer
            .observe_with_options(&canvas, &options)
            .expect("failed to register canvas mutation observer");
        callback.forget();
    }

    wasm_bindgen_futures::spawn_local(async move {
        let body_style = window.document().unwrap().body().unwrap().style();
        let local_storage = window.local_storage().unwrap().unwrap();
        loop {
            let Some(command) = output_rx.recv().await else {
                tracing::warn!(
                    "WebWorkerRunner main thread loop is stopping! This is not supposed to happen."
                );
                return;
            };

            match command.0 {
                WebWorkerRunnerOutputInner::PlatformOutput(output) => {
                    let _ = body_style.set_property(
                        "cursor",
                        match output.cursor_icon {
                            egui::CursorIcon::Default => "default",
                            egui::CursorIcon::None => "none",

                            egui::CursorIcon::ContextMenu => "context-menu",
                            egui::CursorIcon::Help => "help",
                            egui::CursorIcon::PointingHand => "pointer",
                            egui::CursorIcon::Progress => "progress",
                            egui::CursorIcon::Wait => "wait",

                            egui::CursorIcon::Cell => "cell",
                            egui::CursorIcon::Crosshair => "crosshair",
                            egui::CursorIcon::Text => "text",
                            egui::CursorIcon::VerticalText => "vertical-text",

                            egui::CursorIcon::Alias => "alias",
                            egui::CursorIcon::Copy => "copy",
                            egui::CursorIcon::Move => "move",
                            egui::CursorIcon::NoDrop => "no-drop",
                            egui::CursorIcon::NotAllowed => "not-allowed",
                            egui::CursorIcon::Grab => "grab",
                            egui::CursorIcon::Grabbing => "grabbing",

                            egui::CursorIcon::AllScroll => "all-scroll",
                            egui::CursorIcon::ResizeColumn => "col-resize",
                            egui::CursorIcon::ResizeRow => "row-resize",
                            egui::CursorIcon::ResizeNorth => "n-resize",
                            egui::CursorIcon::ResizeEast => "e-resize",
                            egui::CursorIcon::ResizeSouth => "s-resize",
                            egui::CursorIcon::ResizeWest => "w-resize",
                            egui::CursorIcon::ResizeNorthEast => "ne-resize",
                            egui::CursorIcon::ResizeNorthWest => "nw-resize",
                            egui::CursorIcon::ResizeSouthEast => "se-resize",
                            egui::CursorIcon::ResizeSouthWest => "sw-resize",
                            egui::CursorIcon::ResizeHorizontal => "ew-resize",
                            egui::CursorIcon::ResizeVertical => "ns-resize",
                            egui::CursorIcon::ResizeNwSe => "nwse-resize",
                            egui::CursorIcon::ResizeNeSw => "nesw-resize",

                            egui::CursorIcon::ZoomIn => "zoom-in",
                            egui::CursorIcon::ZoomOut => "zoom-out",
                        },
                    );

                    if !output.copied_text.is_empty() {
                        if let Err(e) = wasm_bindgen_futures::JsFuture::from(
                            window
                                .navigator()
                                .clipboard()
                                .unwrap()
                                .write_text(&output.copied_text),
                        )
                        .await
                        {
                            tracing::warn!(
                                "Failed to copy to clipboard: {}",
                                e.unchecked_into::<js_sys::Error>().to_string()
                            );
                        }
                    }

                    if let Some(url) = output.open_url {
                        if let Err(e) = window.open_with_url_and_target(
                            &url.url,
                            if url.new_tab { "_blank" } else { "_self" },
                        ) {
                            tracing::warn!(
                                "Failed to open URL: {}",
                                e.unchecked_into::<js_sys::Error>().to_string()
                            );
                        }
                    }
                }

                WebWorkerRunnerOutputInner::StorageGet(key, oneshot_tx) => {
                    let _ = oneshot_tx.send(local_storage.get(&key).ok().flatten());
                }

                WebWorkerRunnerOutputInner::StorageSet(key, value, oneshot_tx) => {
                    let _ = oneshot_tx.send(local_storage.set(&key, &value).is_ok());
                }
            }
        }
    });
}

/// This should be called when the application panics to stop the renderer and event listeners.
pub fn panic_hook() {
    let _ = PANIC_LOCK.set(());
}
