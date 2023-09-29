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
use super::get_worker;
use crate::prelude::*;
use eframe::{egui_wgpu, wgpu};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function is_worker() { return self instanceof DedicatedWorkerGlobalScope; }"
)]
extern "C" {
    fn is_worker() -> bool;
}

// A binding for this attribute was added in July 2023 but hasn't made its way into a release of
// web-sys as of September 2023
#[wasm_bindgen(inline_js = "export function performance(w) { return w.performance; }")]
extern "C" {
    fn performance(worker: &web_sys::DedicatedWorkerGlobalScope) -> Option<web_sys::Performance>;
}

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

struct WebWorkerRunnerState {
    app: Box<dyn CustomApp>,
    render_state: egui_wgpu::RenderState,
    canvas: web_sys::OffscreenCanvas,
    surface: wgpu::Surface,
    surface_configuration: wgpu::SurfaceConfiguration,
    native_pixels_per_point: Option<f32>,

    screen_resize_rx: Option<mpsc::Receiver<(u32, u32)>>,
}

/// A runner for wgpu egui applications intended to be run in a web worker.
/// Currently only targets WebGPU, not WebGL.
#[derive(Clone)]
pub struct WebWorkerRunner {
    state: std::rc::Rc<Mutex<WebWorkerRunnerState>>,
    integration_info: eframe::IntegrationInfo,
    context: egui::Context,
    time_lock: Arc<RwLock<f64>>,
}

impl WebWorkerRunner {
    /// Creates a new `WebWorkerRunner` to render onto the given `OffscreenCanvas` with the
    /// given configuration options.
    ///
    /// This function MUST be run in a web worker.
    ///
    /// `screen_resize_rx` should receive the new (x, y) inner size of the screen whenever
    /// the screen inner size changes.
    pub async fn new(
        app_creator: Box<dyn FnOnce(&eframe::CreationContext<'_>) -> Box<dyn CustomApp>>,
        canvas: web_sys::OffscreenCanvas,
        web_options: eframe::WebOptions,
        device_pixel_ratio: f32,
        prefers_color_scheme_dark: Option<bool>,
        screen_resize_rx: Option<mpsc::Receiver<(u32, u32)>>,
    ) -> Self {
        if !is_worker() {
            panic!("cannot use `WebWorkerRunner::new()` outside of a web worker");
        }

        let time_lock = Arc::new(RwLock::new(0.));
        let worker = get_worker();

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
                *time_lock.write() =
                    performance(&get_worker()).unwrap().now() / 1000. + i.after.as_secs_f64();
            });
        }

        Self {
            state: std::rc::Rc::new(Mutex::new(WebWorkerRunnerState {
                app: app_creator(&eframe::CreationContext {
                    egui_ctx: context.clone(),
                    integration_info: integration_info.clone(),
                    storage: Some(&Storage::default()),
                    wgpu_render_state: Some(render_state.clone()),
                }),
                render_state,
                surface,
                surface_configuration,
                native_pixels_per_point,
                canvas,
                screen_resize_rx,
            })),
            integration_info,
            context,
            time_lock,
        }
    }

    /// Sets up the hook to render the app to the canvas.
    pub fn setup_render_hook(self) {
        let callback = Closure::once(move || {
            let mut state = self.state.lock();
            let worker = get_worker();

            // Resize the canvas if the screen size has changed
            if let Some(screen_resize_rx) = &state.screen_resize_rx {
                if let Ok((width, height)) = screen_resize_rx.try_recv() {
                    if width != state.surface_configuration.width
                        || height != state.surface_configuration.height
                    {
                        state.canvas.set_width(width);
                        state.canvas.set_height(height);
                        state.surface_configuration.width = width;
                        state.surface_configuration.height = height;
                        state
                            .surface
                            .configure(&state.render_state.device, &state.surface_configuration);

                        // Also trigger a rerender immediately
                        *self.time_lock.write() = 0.;
                    }
                }
            }

            // Render only if sufficient time has passed since the last render
            if performance(&worker).unwrap().now() / 1000. >= *self.time_lock.read() {
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
                    time: Some(performance(&worker).unwrap().now() / 1000.),
                    ..Default::default()
                };
                let output = self.context.run(input, |_| {
                    state.app.custom_update(
                        &self.context,
                        &mut CustomFrame {
                            info: &self.integration_info,
                            storage: None,
                        },
                    )
                });
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

                *self.time_lock.write() = performance(&worker).unwrap().now() / 1000.
                    + output.repaint_after.as_secs_f64();
            }

            self.clone().setup_render_hook();
        });

        let _ = get_worker().request_animation_frame(callback.as_ref().unchecked_ref());
        callback.forget();
    }
}