// Copyright (C) 2024 Melody Madeline Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use wasm_bindgen::JsCast;

pub(super) fn runner_worker(state_cell: std::rc::Rc<std::cell::RefCell<super::WorkerState>>) {
    let mut state = state_cell.borrow_mut();
    let state_cell = state_cell.clone();

    state.runner_worker_closure = Some(wasm_bindgen::closure::Closure::once(move || {
        let mut state_ref = state_cell.borrow_mut();
        let state = &mut *state_ref;

        // Unsubscribe all event handlers on panic
        if super::has_panicked() {
            if let Some(panic_tx) = state.channels.panic_tx.take() {
                let _ = panic_tx.send(());
            }
            return;
        }

        state.input.events.clear();

        let old_native_pixels_per_point = state.native_pixels_per_point;
        let mut needs_repaint = false;
        let mut needs_save = false;

        // Handle each event that was sent from the main thread
        for event in state.channels.event_rx.try_iter() {
            match event {
                super::Event::Egui(event) => {
                    match event {
                        egui::Event::WindowFocused(new_has_focus) => {
                            state.input.focused = new_has_focus;
                            state.touch = None;
                            state.input.events.push(event);
                        }

                        egui::Event::MouseWheel {
                            delta: wheel_delta,
                            modifiers: wheel_modifiers,
                            ..
                        } => {
                            if wheel_modifiers.ctrl && !state.input.modifiers.ctrl {
                                // The browser is saying the ctrl key is down, but it isn't _really_.
                                // This happens on pinch-to-zoom on a Mac trackpad.
                                // egui will treat ctrl+scroll as zoom, so it all works.
                                // However, we explicitly handle it here in order to better match the pinch-to-zoom
                                // speed of a native app, without being sensitive to egui's `scroll_zoom_speed` setting.
                                let pinch_to_zoom_sensitivity = 0.01; // Feels good on a Mac trackpad in 2024
                                let zoom_factor = (pinch_to_zoom_sensitivity * wheel_delta.y).exp();
                                state.input.events.push(egui::Event::Zoom(zoom_factor));
                            } else {
                                state.input.events.push(event);
                            }
                        }

                        _ => state.input.events.push(event),
                    }

                    needs_repaint = true;
                }

                super::Event::ScreenResize {
                    inner_width,
                    inner_height,
                    device_pixel_ratio,
                } => {
                    state.width = inner_width;
                    state.height = inner_height;
                    state.native_pixels_per_point = device_pixel_ratio;
                }

                super::Event::Modifiers(new_modifiers) => {
                    state.input.modifiers = new_modifiers;
                }

                super::Event::Touch(touch_id) => {
                    state.touch = touch_id;
                    needs_repaint = true;
                }

                super::Event::Save => {
                    needs_save = true;
                }

                super::Event::ReleaseAllKeys => {
                    state.context.input(|i| {
                        for key in i.keys_down.iter().copied() {
                            state.input.events.push(egui::Event::Key {
                                key,
                                physical_key: None,
                                pressed: false,
                                repeat: false,
                                modifiers: state.input.modifiers,
                            });
                        }
                    });
                    needs_repaint = true;
                }
            }
        }

        let now = state.worker.performance().unwrap().now();

        // If the screen size or pixel ratio has changed, trigger a rerender
        if state.native_pixels_per_point != old_native_pixels_per_point || {
            let pixels_per_point = state.context.zoom_factor() * state.native_pixels_per_point;
            let width_pixels = (state.width as f32 * pixels_per_point).round() as u32;
            let height_pixels = (state.height as f32 * pixels_per_point).round() as u32;
            state.surface_configuration.width != width_pixels
                || state.surface_configuration.height != height_pixels
        } {
            needs_repaint = true;
        }

        // Render if needed, or if enough time has passed since the previous render
        if needs_repaint || now >= state.repaint_time.load(portable_atomic::Ordering::SeqCst) {
            // Cancel all pending repaints, since we're doing that right now
            state
                .repaint_time
                .store(f64::INFINITY, portable_atomic::Ordering::SeqCst);

            // Ask the app to paint the next frame
            state.input.screen_rect = Some(egui::Rect::from_min_max(
                egui::Pos2::ZERO,
                egui::pos2(state.width as f32, state.height as f32),
            ));
            state.input.time = Some(now / 1000.);
            state.input.max_texture_side =
                Some(state.render_state.device.limits().max_texture_dimension_2d as usize);
            state
                .input
                .viewports
                .entry(egui::ViewportId::ROOT)
                .or_default()
                .native_pixels_per_point = Some(state.native_pixels_per_point);
            state.app.raw_input_hook(&state.context, &mut state.input);
            let output = state.context.run(state.input.clone(), |context| {
                crate::app::AppTrait::update(&mut *state.app, context)
            });
            state
                .channels
                .output_tx
                .send(super::Output::Egui {
                    output: output.platform_output,
                    zoom_factor: state.context.zoom_factor(),
                    screen_reader_enabled: state.context.options(|o| o.screen_reader),
                })
                .unwrap();
            let clear_color = state.app.clear_color(&state.context.style().visuals);
            let paint_jobs = state
                .context
                .tessellate(output.shapes, output.pixels_per_point);

            // Resize the screen if needed
            let width_pixels = (state.width as f32 * output.pixels_per_point).round() as u32;
            let height_pixels = (state.height as f32 * output.pixels_per_point).round() as u32;
            if state.surface_configuration.width != width_pixels
                || state.surface_configuration.height != height_pixels
            {
                state.surface_configuration.width = width_pixels;
                state.surface_configuration.height = height_pixels;
                state.canvas.set_width(width_pixels);
                state.canvas.set_height(height_pixels);
                state
                    .surface
                    .configure(&state.render_state.device, &state.surface_configuration);
            }

            let mut encoder =
                state
                    .render_state
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Luminol Web Runner Encoder"),
                    });
            let screen_descriptor = egui_wgpu::ScreenDescriptor {
                size_in_pixels: [
                    state.surface_configuration.width,
                    state.surface_configuration.height,
                ],
                pixels_per_point: output.pixels_per_point,
            };

            // Upload textures to GPU that are changed or newly created in the current frame
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

            // Create texture to render onto
            // Note: This variable needs to live for the entire remaining duration we use
            // `state.render_state` or WebGL will break
            let render_texture = state.surface.get_current_texture().unwrap();

            // Execute egui's render pass
            {
                let renderer = state.render_state.renderer.read();
                let view = render_texture.texture.create_view(&Default::default());
                let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Luminol Web Runner Renderer"),
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
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                renderer.render(
                    &mut render_pass.forget_lifetime(),
                    &paint_jobs[..],
                    &screen_descriptor,
                );
            }

            // Copy from the internal drawing buffer onto the HTML canvas
            state.render_state.queue.submit(
                command_buffers
                    .into_iter()
                    .chain(std::iter::once(encoder.finish())),
            );
            render_texture.present();

            // Remove textures that are no longer needed after this frame
            {
                let mut renderer = state.render_state.renderer.write();
                for id in output.textures_delta.free.iter() {
                    renderer.free_texture(id);
                }
            }
        }

        // Save if requested
        if needs_save || now >= state.save_time + state.app.auto_save_interval().as_secs_f64() {
            state.save_time = now;
            {
                let mut storage = state.storage.clone();
                state.app.save(&mut storage);
            }
            match state.context.memory(ron::to_string) {
                Ok(value) => {
                    let (oneshot_tx, oneshot_rx) = oneshot::channel();
                    state
                        .channels
                        .output_tx
                        .send(super::Output::StorageSet {
                            key: state.app_id.to_string(),
                            value,
                            oneshot_tx,
                        })
                        .unwrap();
                    if !oneshot_rx.recv().unwrap() {
                        tracing::warn!("Failed to save memory for {}", state.app_id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to serialize memory for {}: {e}", state.app_id)
                }
            }
        }

        // Request this closure to be called again the next frame
        drop(state_ref);
        runner_worker(state_cell);
    }));

    state
        .worker
        .request_animation_frame(
            state
                .runner_worker_closure
                .as_ref()
                .unwrap()
                .as_ref()
                .unchecked_ref(),
        )
        .expect("failed to request an animation frame");
}
