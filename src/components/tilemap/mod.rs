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
mod quad;
mod tiles;
mod vertex;
mod viewport;

use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::prelude::*;

#[derive(Debug)]
pub struct Tilemap {
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    pub pan: egui::Vec2,

    resources: Arc<Resources>,
    ani_instant: Instant,
}

#[derive(Debug)]
struct Resources {
    tiles: tiles::Tiles,
    events: events::Events,
    viewport: viewport::Viewport,
    panorama: Option<plane::Plane>,
    fog: Option<plane::Plane>,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub enum BlendMode {
    Normal = 0,
    Add = 1,
    Subtract = 2,
}

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        // Load the map.
        let map = state!().data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state!().data_cache.tilesets();
        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let tiles = tiles::Tiles::new(tileset, &map)?;
        let events = events::Events::new(&map, &tiles.atlas.atlas_texture)?;

        let panorama = if tileset.panorama_name.is_empty() {
            None
        } else {
            Some(plane::Plane::new(
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Panoramas", &tileset.panorama_name)?,
                tileset.panorama_hue,
                1,
                BlendMode::Normal,
                1,
            ))
        };
        let fog = if tileset.panorama_name.is_empty() {
            None
        } else {
            Some(plane::Plane::new(
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Fogs", &tileset.fog_name)?,
                tileset.fog_hue,
                tileset.fog_zoom,
                match tileset.fog_blend_type {
                    0 => BlendMode::Normal,
                    1 => BlendMode::Add,
                    2 => BlendMode::Subtract,
                    mode => return Err(format!("unexpected blend mode {mode}")),
                },
                tileset.fog_opacity,
            ))
        };

        let viewport = viewport::Viewport::new();

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
            }),
            ani_instant: Instant::now(),
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        map_id: i32,
        cursor_pos: &mut egui::Pos2,
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
            let mut scale = self.scale();
            // We need to store the old scale before applying any transformations
            let old_scale = scale;
            let delta = ui.input(|i| i.scroll_delta.y * 5.0);

            // Apply scroll and cap max zoom to 15%
            scale += delta / 30.;
            scale = 15.0_f32.max(scale);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * scale;

            // Figure out the tile the cursor is hovering over
            let tile_size = (scale / 100.) * 32.;
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0.0, map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0.0, map.height as f32 - 1.);
            // Handle input

            if scale != self.scale() {
                self.set_scale(scale);
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
        let scale = self.scale() / (ui.ctx().pixels_per_point() * 100.);
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
        ui.painter().add(egui::PaintCallback {
            rect: map_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |_device, _queue, _encoder, paint_callback_resources| {
                        //
                        let res_hash: &mut HashMap<i32, Arc<Resources>> = paint_callback_resources
                            .entry()
                            .or_insert_with(Default::default);
                        res_hash.insert(map_id, resources.clone());

                        vec![]
                    })
                    .paint(move |info, render_pass, paint_callback_resources| {
                        //
                        let res_hash: &HashMap<i32, Arc<Resources>> = paint_callback_resources
                            .get()
                            .expect("failed to get tilemap resources");
                        let Resources {
                            tiles,
                            events,
                            viewport,
                            panorama,
                            fog,
                        } = res_hash[&map_id].as_ref();

                        let proj = cgmath::ortho(
                            0.0,
                            info.viewport_in_pixels().width_px,
                            info.viewport_in_pixels().height_px,
                            0.0,
                            -1.0,
                            1.0,
                        );
                        viewport.set_proj(proj);
                        viewport.bind(render_pass);

                        if let Some(panorama) = panorama {
                            panorama.draw(render_pass);
                        }
                        tiles.draw(render_pass);
                        events.draw(render_pass);
                        if let Some(fog) = fog {
                            fog.draw(render_pass);
                        }
                    }),
            ),
        });

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::DARK_GRAY),
        );

        // TODO: draw event bounds instead?
        for (_, event) in map.events.iter() {
            let box_rect = egui::Rect::from_min_size(
                map_rect.min
                    + egui::Vec2::new(event.x as f32 * tile_size, event.y as f32 * tile_size),
                egui::Vec2::splat(tile_size),
            );

            ui.painter()
                .rect_stroke(box_rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
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
            map_rect.min + (cursor_pos.to_vec2() * tile_size),
            egui::Vec2::splat(tile_size),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::YELLOW),
        );

        /*
        self.tiles.uniform.set_pan(pan);
        */

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(
                tiles::TILESET_WIDTH as f32,
                self.resources.tiles.atlas.tileset_height as f32,
            ),
            egui::Sense::click(),
        );

        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(|device, queue, _encoder, paint_callback_resources| {
                        //
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                    }),
            ),
        });
    }

    pub fn scale(&mut self) -> f32 {
        self.resources.viewport.scale()
    }

    pub fn set_scale(&self, scale: f32) {
        self.resources.viewport.set_scale(scale);
    }

    /*
    #[allow(unused_variables, unused_assignments)]
    fn load_data(map: &rpg::Map, tileset: &rpg::Tileset) {
        let state = state!();

        let event_texs = map
            .events
            .iter()
            .filter_map(|(_, e)| e.pages.first().map(|p| p.graphic.character_name.clone()))
            .filter(|s| !s.is_empty())
            .dedup()
            .map(|char_name| {
                //
                state
                    .image_cache
                    .load_wgpu_image("Graphics/Characters", &char_name)
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Fogs", &tileset.fog_name)
            .ok();

        let pano_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Panoramas", &tileset.panorama_name)
            .ok();

        // Finally create and return the struct.
    }
    */
}
