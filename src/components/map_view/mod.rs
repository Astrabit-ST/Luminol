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
mod events;
mod plane;

use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::prelude::*;

#[derive(Debug)]
pub struct MapView {
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    pub pan: egui::Vec2,

    map_id: usize,

    resources: Arc<Resources>,
    ani_instant: Instant,

    pub selected_layer: SelectedLayer,
    pub cursor_pos: egui::Pos2,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub event_enabled: bool,
    pub enabled_layers: Vec<bool>,
    pub scale: f32,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum SelectedLayer {
    #[default]
    Events,
    Tiles(usize),
}

#[derive(Debug)]
struct Resources {
    tiles: Tiles,
    events: events::Events,
    viewport: Viewport,
    panorama: Option<plane::Plane>,
    fog: Option<plane::Plane>,
    atlas: Atlas,
}

type ResourcesHash = HashMap<usize, Arc<Resources>>;

impl MapView {
    pub fn new(map_id: usize, map: &rpg::Map, tileset: &rpg::Tileset) -> Result<MapView, String> {
        // Get tilesets.
        let atlas = state!().atlas_cache.load_atlas(tileset)?;

        let tiles = Tiles::new(atlas.clone(), &map.data);

        let events = events::Events::new(map, &atlas)?;

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            Some(plane::Plane::new(
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Panoramas", panorama_name)?,
                tileset.panorama_hue,
                100,
                BlendMode::Normal,
                255,
                map.width,
                map.height,
            ))
        } else {
            None
        };
        let fog = if let Some(ref fog_name) = tileset.fog_name {
            Some(plane::Plane::new(
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Fogs", fog_name)?,
                tileset.fog_hue,
                tileset.fog_zoom,
                tileset.fog_blend_type,
                tileset.fog_opacity,
                map.width,
                map.height,
            ))
        } else {
            None
        };
        let viewport = Viewport::new();
        viewport.set_proj(cgmath::ortho(
            0.0,
            map.width as f32 * 32.,
            map.height as f32 * 32.,
            0.0,
            -1.0,
            1.0,
        ));

        Ok(Self {
            visible_display: false,
            move_preview: false,
            pan: egui::Vec2::ZERO,

            resources: Arc::new(Resources {
                tiles,
                events,
                viewport,
                panorama,
                fog,
                atlas,
            }),

            ani_instant: Instant::now(),
            map_id,

            selected_layer: SelectedLayer::default(),
            cursor_pos: egui::Pos2::ZERO,

            fog_enabled: true,
            pano_enabled: true,
            event_enabled: true,
            enabled_layers: vec![true; map.data.zsize()],
            scale: 100.,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        dragging_event: bool,
    ) -> egui::Response {
        if self.ani_instant.elapsed() >= Duration::from_secs_f32((1. / 60.) * 16.) {
            self.ani_instant = Instant::now();
            self.resources.tiles.autotiles.inc_ani_index();
        }
        ui.ctx().request_repaint_after(Duration::from_millis(16));

        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = self.scale;
            let delta = ui.input(|i| i.scroll_delta.y * 5.0);

            // Apply scroll and cap max zoom to 15%
            self.scale += delta / 30.;
            self.scale = 15.0_f32.max(self.scale);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * self.scale;

            // Figure out the tile the cursor is hovering over
            let tile_size = (self.scale / (ui.ctx().pixels_per_point() * 100.)) * 32.;
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0.0, map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0.0, map.height as f32 - 1.);
            // Handle input
            if matches!(self.selected_layer, SelectedLayer::Tiles(_))
                || dragging_event
                || response.clicked()
            {
                self.cursor_pos = pos_tile.to_pos2();
            }
        }

        // Handle pan
        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input(|i| {
                i.modifiers.command && response.dragged_by(egui::PointerButton::Primary)
            }));
        if panning_map_view {
            self.pan += response.drag_delta();
            ui.ctx().request_repaint();
        }

        // Handle cursor icon
        if panning_map_view {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        // Determine some values which are relatively constant
        // If we don't use pixels_per_point then the map is the wrong size.
        // *don't ask me how i know this*.
        // its a *long* story
        let scale = self.scale / (ui.ctx().pixels_per_point() * 100.);
        let tile_size = 32. * scale;
        let canvas_pos = canvas_center + self.pan;

        let width2 = map.width as f32 / 2.;
        let height2 = map.height as f32 / 2.;

        let pos = egui::Vec2::new(width2 * tile_size, height2 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        let resources = self.resources.clone();
        let map_id = self.map_id;

        let fog_enabled = self.fog_enabled;
        let pano_enabled = self.pano_enabled;
        let event_enabled = self.event_enabled;
        let enabled_layers = self.enabled_layers.clone();

        let paint_callback = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, _queue, _, paint_callback_resources| {
                let res_hash: &mut ResourcesHash = paint_callback_resources
                    .entry()
                    .or_insert_with(Default::default);
                res_hash.insert(map_id, resources.clone());

                vec![]
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let res_hash: &ResourcesHash = paint_callback_resources.get().unwrap();
                let resources = &res_hash[&map_id];
                let Resources {
                    tiles: map_tiles,
                    viewport: map_viewport,
                    panorama,
                    fog,
                    events,
                    ..
                } = resources.as_ref();

                map_viewport.bind(render_pass);

                if pano_enabled {
                    if let Some(panorama) = panorama {
                        panorama.draw(render_pass);
                    }
                }

                map_tiles.draw(render_pass, &enabled_layers);

                if event_enabled {
                    events.draw(render_pass);
                }
                if fog_enabled {
                    if let Some(fog) = fog {
                        fog.draw(render_pass);
                    }
                }
            });

        ui.painter().add(egui::PaintCallback {
            rect: map_rect,
            callback: Arc::new(paint_callback),
        });

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::DARK_GRAY),
        );

        // TODO: draw event bounds instead?
        if event_enabled {
            for (_, event) in map.events.iter() {
                let box_rect = egui::Rect::from_min_size(
                    map_rect.min
                        + egui::Vec2::new(event.x as f32 * tile_size, event.y as f32 * tile_size),
                    egui::Vec2::splat(tile_size),
                );

                ui.painter().rect_stroke(
                    box_rect,
                    5.0,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );
            }
        }

        // Do we display the visible region?
        if self.visible_display {
            // Determine the visible region.
            let width2: f32 = (640. / 2.) * scale;
            let height2: f32 = (480. / 2.) * scale;

            let pos = egui::Vec2::new(width2, height2);
            let visible_rect = egui::Rect {
                min: canvas_center - pos,
                max: canvas_center + pos,
            };

            // Show the region.
            ui.painter().rect_stroke(
                visible_rect,
                5.0,
                egui::Stroke::new(1.0, egui::Color32::YELLOW),
            );
        }

        // Display cursor.
        let cursor_rect = egui::Rect::from_min_size(
            map_rect.min + (self.cursor_pos.to_vec2() * tile_size),
            egui::Vec2::splat(tile_size),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::YELLOW),
        );

        response
    }

    /*
    pub fn save_to_disk(&self) {
        let render_state = &state!().render_state;

        let mut encoder =
            render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("tilemap disk image render encoder"),
                });

        let Resources {
            tiles,
            events,
            map_viewport,
            panorama,
            fog,
            ..
        } = self.resources.as_ref();
        let map_id = self.map_id;
        let render_tex: &wgpu::Texture = todo!();
        let render_tex_view = todo!();

        let width = render_tex.width();
        let height = render_tex.height();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: render_tex_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    }),
                    store: true,
                },
            })],
            depth_stencil_attachment: None,
        });

        map_viewport.bind(&mut render_pass);

        if self.pano_enabled {
            if let Some(panorama) = panorama {
                panorama.draw(&mut render_pass);
            }
        }

        tiles.draw_map(&mut render_pass, &self.enabled_layers);

        if self.event_enabled {
            events.draw(&mut render_pass);
        }
        if self.fog_enabled {
            if let Some(fog) = fog {
                fog.draw(&mut render_pass);
            }
        }

        drop(render_pass);

        let bytes_per_row = width * 4;
        let bytes_per_row = bytes_per_row + 256 - (bytes_per_row % 256);
        let buffer_len = bytes_per_row * height;

        let buffer = render_state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&format!("tilemap {map_id} buffer render to disk")),
            size: buffer_len as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        encoder.copy_texture_to_buffer(
            render_tex.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: None,
                },
            },
            render_tex.size(),
        );

        render_state.queue.submit(std::iter::once(encoder.finish()));

        wgpu::util::DownloadBuffer::read_buffer(
            &render_state.device,
            &render_state.queue,
            &buffer.slice(..),
            move |result| {
                let buffer = match result {
                    Ok(b) => b,
                    Err(e) => {
                        state!()
                            .toasts
                            .error(format!("error getting tilemap view into ram: {e:?}"));
                        return;
                    }
                };

                let buffer =
                    image::RgbaImage::from_raw(bytes_per_row / 4, height, buffer.to_vec()).unwrap();

                let result = buffer.save(format!("map_{map_id}.png"));

                match result {
                    Ok(_) => state!()
                        .toasts
                        .info(format!("Sucessfully saved \"map_{map_id}.png\"")),
                    Err(e) => state!()
                        .toasts
                        .error(format!("Error saving tileset view to disk: {e:?}")),
                }
            },
        );
    }
    */
}
