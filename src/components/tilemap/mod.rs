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
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        map_id: i32,
        cursor_pos: &mut egui::Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
        dragging_event: bool,
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

        let tiles = self.tiles.clone();
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
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

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        let mut scale = self.tiles.uniform.scale();
        let mut pan = self.tiles.uniform.pan();

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = scale;
            let delta = ui.input(|i| i.scroll_delta.y * 5.0);

            // Apply scroll and cap max zoom to 15%
            scale += delta / 30.;
            scale = 15.0_f32.max(scale);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            pan = pos - canvas_center - pos_norm * scale;

            // Figure out the tile the cursor is hovering over
            let tile_size = (scale / 100.) * 32.;
            let mut pos_tile = (pos - pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0.0, map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0.0, map.height as f32 - 1.);
            // Handle input
            if selected_layer < map.data.zsize() || dragging_event || response.clicked() {
                *cursor_pos = pos_tile.to_pos2();
            }
        }

        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input(|i| {
                i.modifiers.command && response.dragged_by(egui::PointerButton::Primary)
            }));
        if panning_map_view {
            pan += response.drag_delta();
        }

        // Handle cursor icon
        if panning_map_view {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        self.tiles.uniform.set_scale(scale);
        self.tiles.uniform.set_pan(pan);

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
