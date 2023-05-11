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
mod planes;
mod tiles;

use std::sync::Arc;
use std::time::Instant;

pub use crate::prelude::*;

pub struct Tilemap {
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    tiles: Arc<tiles::Tiles>,
    ani_instant: Instant,

    pub pan: egui::Vec2,
}

#[derive(Default, Debug)]
struct Resources(HashMap<i32, Arc<tiles::Tiles>>);

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        // Load the map.
        let map = state!().data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state!().data_cache.tilesets();
        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let tiles = tiles::Tiles::new(tileset, &map)?;
        let tiles = Arc::new(tiles);

        Ok(Self {
            visible_display: false,
            move_preview: false,

            tiles,
            ani_instant: Instant::now(),

            pan: egui::Vec2::ZERO,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        map_id: i32,
        cursor_pos: &mut egui::Pos2,
    ) -> egui::Response {
        if Instant::now().duration_since(self.ani_instant).as_millis() > 16 {
            self.ani_instant = Instant::now();
            self.tiles.uniform.inc_ani_index();
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }

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

        let tiles = self.tiles.clone();
        ui.painter().add(egui::PaintCallback {
            rect: map_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |_device, _queue, _encoder, paint_callback_resources| {
                        //
                        let resources: &mut Resources = paint_callback_resources
                            .entry()
                            .or_insert_with(Default::default);
                        resources.0.insert(map_id, tiles.clone());

                        vec![]
                    })
                    .paint(move |info, render_pass, paint_callback_resources| {
                        //
                        let resources: &Resources = paint_callback_resources
                            .get()
                            .expect("failed to get tilemap resources");
                        let tiles = resources.0[&map_id].as_ref();

                        tiles.uniform.set_proj(cgmath::ortho(
                            0.0,
                            info.viewport_in_pixels().width_px,
                            info.viewport_in_pixels().height_px,
                            0.0,
                            -1.0,
                            1.0,
                        ));

                        tiles.draw(render_pass);
                    }),
            ),
        });

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::DARK_GRAY),
        );

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
                self.tiles.atlas.tileset_height as f32,
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
        self.tiles.uniform.scale()
    }

    pub fn set_scale(&self, scale: f32) {
        self.tiles.uniform.set_scale(scale);
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
