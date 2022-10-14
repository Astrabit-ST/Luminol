// Copyright (C) 2022 Lily Lyons
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

/*
This file serves as a baseline for how to handle the tilemap.
It's slow and should only be used as a reference for how the tilemap works.
*/

use egui_extras::RetainedImage;
use std::collections::HashMap;
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use wasm_timer::Instant;

use egui::{Pos2, Response, Vec2};
use ndarray::Axis;

use crate::data::rmxp_structs::rpg;
use crate::{load_image_software, UpdateInfo};

use super::TilemapDef;

/// A generic tilemap, implements TilemapDef.
/// Does not use any special optimizations.
#[allow(dead_code)]
pub struct Tilemap {
    /// The tilemap pan.
    pub pan: Vec2,
    /// The scale of the tilemap.
    pub scale: f32,
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    ani_idx: i32,
    ani_instant: Instant,
    load_promise: poll_promise::Promise<Result<Textures, String>>,
}

struct Textures {
    tileset_tex: RetainedImage,
    autotile_texs: Vec<Option<RetainedImage>>,
    event_texs: HashMap<(String, i32), Option<RetainedImage>>,
    fog_tex: Option<RetainedImage>,
    fog_zoom: i32,
    pano_tex: Option<RetainedImage>,
}

/// Hardcoded list of tiles from r48 and old python Luminol.
/// There seems to be very little pattern in autotile IDs so this is sadly
/// the best we can do.
const AUTOTILES: [[i32; 4]; 48] = [
    [26, 27, 32, 33],
    [4, 27, 32, 33],
    [26, 5, 32, 33],
    [4, 5, 32, 33],
    [26, 27, 32, 11],
    [4, 27, 32, 11],
    [26, 5, 32, 11],
    [4, 5, 32, 11],
    [26, 27, 10, 33],
    [4, 27, 10, 33],
    [26, 5, 10, 33],
    [4, 5, 10, 33],
    [26, 27, 10, 11],
    [4, 27, 10, 11],
    [26, 5, 10, 11],
    [4, 5, 10, 11],
    [24, 25, 30, 31],
    [24, 5, 30, 31],
    [24, 25, 30, 11],
    [24, 5, 30, 11],
    [14, 15, 20, 21],
    [14, 15, 20, 11],
    [14, 15, 10, 21],
    [14, 15, 10, 11],
    [28, 29, 34, 35],
    [28, 29, 10, 35],
    [4, 29, 34, 35],
    [4, 29, 10, 35],
    [38, 39, 44, 45],
    [4, 39, 44, 45],
    [38, 5, 44, 45],
    [4, 5, 44, 45],
    [24, 29, 30, 35],
    [14, 15, 44, 45],
    [12, 13, 18, 19],
    [12, 13, 18, 11],
    [16, 17, 22, 23],
    [16, 17, 10, 23],
    [40, 41, 46, 47],
    [4, 41, 46, 47],
    [36, 37, 42, 43],
    [36, 5, 42, 43],
    [12, 17, 18, 23],
    [12, 13, 42, 43],
    [36, 41, 42, 47],
    [16, 17, 46, 47],
    [12, 17, 42, 47],
    [0, 1, 6, 7],
];

#[allow(dead_code)]
impl TilemapDef for Tilemap {
    fn new(info: &'static UpdateInfo, id: i32) -> Self {
        Self {
            pan: Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            ani_idx: 0,
            ani_instant: Instant::now(),
            load_promise: poll_promise::Promise::spawn_local(async move {
                Self::load_data(info, id).await
            }),
        }
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
    ) -> Response {
        let textures = self.load_promise.ready().unwrap().as_ref().unwrap();

        // Every 16 frames update autotile animation index
        if self.ani_instant.elapsed() >= Duration::from_secs_f32((1. / 60.) * 16.) {
            self.ani_instant = Instant::now();
            self.ani_idx += 1;
        }

        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = self.scale;
            let delta = ui.input().scroll_delta.y * 5.0;

            // Apply scroll and cap max zoom to 15%
            self.scale += delta / 30.;
            self.scale = 15.0_f32.max(self.scale);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * self.scale;

            // Figure out the tile the cursor is hovering over
            let tile_size = (self.scale / 100.) * 32.;
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0.0, map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0.0, map.height as f32 - 1.);
            // Handle input
            if selected_layer < map.data.len_of(Axis(0)) || response.clicked() {
                *cursor_pos = pos_tile.to_pos2();
            }
        }

        // Handle pan
        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary));
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
        let scale = self.scale / 100.;
        let tile_size = 32. * scale;
        let at_tile_size = 16. * scale;
        let canvas_pos = canvas_center + self.pan;

        let xsize = map.data.len_of(Axis(2));
        let ysize = map.data.len_of(Axis(1));

        let tile_width = 32. / textures.tileset_tex.width() as f32;
        let tile_height = 32. / textures.tileset_tex.height() as f32;

        let width2 = map.width as f32 / 2.;
        let height2 = map.height as f32 / 2.;

        let pos = egui::Vec2::new(width2 * tile_size, height2 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        // Do we need to render a panorama?
        if let Some(pano_tex) = &textures.pano_tex {
            // Find the minimum number of panoramas we can fit in the map size (should fit the screen)
            let mut pano_repeat = map_rect.size() / (pano_tex.size_vec2() * scale);
            // We want to display more not less than we possibly can
            pano_repeat.x = pano_repeat.x.ceil();
            pano_repeat.y = pano_repeat.y.ceil();

            // Iterate through ranges
            for y in 0..(pano_repeat.y as usize) {
                for x in 0..(pano_repeat.x as usize) {
                    // Display the panorama
                    let pano_rect = egui::Rect::from_min_size(
                        map_rect.min
                            + egui::vec2(
                                (pano_tex.width() * x) as f32 * scale,
                                (pano_tex.height() * y) as f32 * scale,
                            ),
                        pano_tex.size_vec2() * scale,
                    );

                    egui::Image::new(pano_tex.texture_id(ui.ctx()), pano_tex.size_vec2() * scale)
                        .paint_at(ui, pano_rect);
                }
            }
        }

        // Iterate through all tiles.
        for (idx, ele) in map.data.iter().enumerate() {
            if *ele < 48 {
                continue;
            }

            // We grab the x and y through some simple means.
            let (x, y, z) = (
                // We reset the x every xsize elements.
                idx % xsize,
                // We reset the y every ysize elements, but only increment it every xsize elements.
                (idx / xsize) % ysize,
                // We change the z every xsize * ysize elements.
                idx / (xsize * ysize),
            );

            // Is the layer toggled off?
            if !toggled_layers[z] {
                continue;
            }

            // Find tile bounds
            let tile_rect = egui::Rect::from_min_size(
                map_rect.min + egui::Vec2::new(x as f32 * tile_size, y as f32 * tile_size),
                egui::Vec2::splat(tile_size),
            );

            // Do we draw an autotile or regular tile?
            if *ele >= 384 {
                // Normalize element
                let ele = ele - 384;

                // Calculate UV coordinates
                let tile_x =
                    (ele as usize % (textures.tileset_tex.width() / 32)) as f32 * tile_width;
                let tile_y =
                    (ele as usize / (textures.tileset_tex.width() / 32)) as f32 * tile_height;

                let uv = egui::Rect::from_min_size(
                    Pos2::new(tile_x, tile_y),
                    egui::vec2(tile_width, tile_height),
                );

                // Display tile
                egui::Image::new(
                    textures.tileset_tex.texture_id(ui.ctx()),
                    textures.tileset_tex.size_vec2(),
                )
                .uv(uv)
                .paint_at(ui, tile_rect);
            } else {
                // holy shit
                // Find what autotile we're displaying
                let autotile_id = (ele / 48) - 1;

                if let Some(autotile_tex) = &textures.autotile_texs[autotile_id as usize] {
                    // Get the relative tile size
                    let tile_width = 16. / autotile_tex.width() as f32;
                    let tile_height = 16. / autotile_tex.height() as f32;

                    // Display each autotile corner (taken from r48)
                    for s_a in 0..2 {
                        for s_b in 0..2 {
                            // Find tile display rectangle
                            let tile_rect = egui::Rect::from_min_size(
                                map_rect.min
                                    + egui::Vec2::new(
                                        (x as f32 * tile_size) + (s_a as f32 * at_tile_size),
                                        (y as f32 * tile_size) + (s_b as f32 * at_tile_size),
                                    ),
                                egui::Vec2::splat(at_tile_size),
                            );

                            // Calculate tile index
                            let ti = AUTOTILES[*ele as usize % 48][s_a + (s_b * 2)];

                            // Calculate tile x
                            let tx = ti % 6;
                            // Offset by animation amount
                            let tx_off = (self.ani_idx as usize % (autotile_tex.width() / 96)) * 6;
                            let tx = (tx + tx_off as i32) as f32 * tile_width;
                            // Calculate tile y
                            let ty = (ti / 6) as f32 * tile_height;

                            // Find uv
                            let uv = egui::Rect::from_min_size(
                                Pos2::new(tx, ty),
                                egui::vec2(tile_width, tile_height),
                            );

                            // Display corner
                            egui::Image::new(
                                autotile_tex.texture_id(ui.ctx()),
                                autotile_tex.size_vec2(),
                            )
                            .uv(uv)
                            .paint_at(ui, tile_rect);
                        }
                    }
                }
            }
        }

        // Do we display events?
        if *toggled_layers.last().unwrap() {
            for (_, event) in map.events.iter() {
                // aaaaaaaa
                // Get graphic and texture
                let graphic = &event.pages[0].graphic;
                let tex = textures
                    .event_texs
                    .get(&(graphic.character_name.clone(), graphic.character_hue))
                    .unwrap();
                if let Some(tex) = tex {
                    // FInd character width and height
                    let cw = (tex.width() / 4) as f32;
                    let ch = (tex.height() / 4) as f32;

                    // The math here display the character correctly.
                    // Why it works? Dunno.
                    let c_rect = egui::Rect::from_min_size(
                        map_rect.min
                            + egui::Vec2::new(
                                (event.x as f32 * tile_size) + ((16. - (cw / 2.)) * scale),
                                (event.y as f32 * tile_size) + ((32. - ch) * scale),
                            ),
                        egui::vec2(cw * scale, ch * scale),
                    );

                    // Find UV coords.
                    let cx = (graphic.pattern as f32 * cw) / tex.width() as f32;
                    let cy = (((graphic.direction - 2) / 2) as f32 * ch) / tex.height() as f32;

                    let uv = egui::Rect::from_min_size(
                        Pos2::new(cx, cy),
                        egui::vec2(cw / tex.width() as f32, ch / tex.height() as f32),
                    );

                    // Display the character.
                    egui::Image::new(tex.texture_id(ui.ctx()), tex.size_vec2())
                        .uv(uv)
                        .paint_at(ui, c_rect);
                // Do we need to display a tile instead?
                } else if graphic.tile_id.is_positive() {
                    // Same logic for tiles. See above.
                    let tile_rect = egui::Rect::from_min_size(
                        map_rect.min
                            + egui::Vec2::new(
                                event.x as f32 * tile_size,
                                event.y as f32 * tile_size,
                            ),
                        egui::Vec2::splat(tile_size),
                    );

                    let tile_x = ((graphic.tile_id - 384) as usize
                        % (textures.tileset_tex.width() / 32))
                        as f32
                        * tile_width;
                    let tile_y = ((graphic.tile_id - 384) as usize
                        / (textures.tileset_tex.width() / 32))
                        as f32
                        * tile_height;

                    let uv = egui::Rect::from_min_size(
                        Pos2::new(tile_x, tile_y),
                        egui::vec2(tile_width, tile_height),
                    );

                    egui::Image::new(
                        textures.tileset_tex.texture_id(ui.ctx()),
                        textures.tileset_tex.size_vec2(),
                    )
                    .uv(uv)
                    .paint_at(ui, tile_rect);
                }

                // Display the event box.
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

        // Display the fog if we should.
        // Uses an almost identical method to panoramas with an added scale.
        if let Some(fog_tex) = &textures.fog_tex {
            let zoom = (textures.fog_zoom as f32 / 100.) * scale;
            let mut fox_repeat = map_rect.size() / (fog_tex.size_vec2() * zoom);
            fox_repeat.x = fox_repeat.x.ceil();
            fox_repeat.y = fox_repeat.y.ceil();

            for y in 0..(fox_repeat.y as usize) {
                for x in 0..(fox_repeat.x as usize) {
                    let fog_rect = egui::Rect::from_min_size(
                        map_rect.min
                            + egui::vec2(
                                (fog_tex.width() * x) as f32 * zoom,
                                (fog_tex.height() * y) as f32 * zoom,
                            ),
                        fog_tex.size_vec2() * zoom,
                    );

                    egui::Image::new(fog_tex.texture_id(ui.ctx()), fog_tex.size_vec2() * zoom)
                        .paint_at(ui, fog_rect);
                }
            }
        }

        // Outline the map.
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
            Vec2::splat(tile_size),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::YELLOW),
        );

        // Every 16 frames request a repaint. This is for autotile animations.
        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32((1. / 60.) * 16.));

        // Return response.
        response
    }

    fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let textures = self.load_promise.ready().unwrap().as_ref().unwrap();

        let (rect, response) =
            ui.allocate_exact_size(textures.tileset_tex.size_vec2(), egui::Sense::click());

        egui::Image::new(textures.tileset_tex.texture_id(ui.ctx()), rect.size()).paint_at(ui, rect);

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mut pos = (pos - rect.min) / 32.;
                pos.x = pos.x.floor();
                pos.y = pos.y.floor();
                *selected_tile = (pos.x + pos.y * 8.) as i16;
            }
        }
        let cursor_x = *selected_tile % 8 * 32;
        let cursor_y = *selected_tile / 8 * 32;
        ui.painter().rect_stroke(
            egui::Rect::from_min_size(
                rect.min + egui::vec2(cursor_x as f32, cursor_y as f32),
                egui::Vec2::splat(32.),
            ),
            5.0,
            egui::Stroke::new(1.0, egui::Color32::WHITE),
        );
    }

    fn textures_loaded(&self) -> bool {
        self.load_promise.ready().is_some()
    }

    fn load_result(&self) -> Result<(), String> {
        self.load_promise
            .ready()
            .unwrap()
            .as_ref()
            .map(|_| ())
            .map_err(|e| e.clone())
    }
}

impl Tilemap {
    #[allow(unused_variables, unused_assignments)]
    async fn load_data(info: &'static UpdateInfo, id: i32) -> Result<Textures, String> {
        // Load the map.
        let tileset_name;
        let autotile_names;

        let event_names: Vec<_>;

        let fog_name;
        let fog_hue;
        let fog_zoom;

        let pano_name;
        let pano_hue;

        // We get all the variables we need from here so we don't borrow the refcell across an await.
        // This could be done with a RwLock or a Mutex but this is more practical.
        // We do have to clone variables but this is negligble compared to the alternative.
        {
            let map = info.data_cache.load_map(&info.filesystem, id).await?;
            // Get tilesets.
            let tilesets = info.data_cache.tilesets();

            // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
            let tileset = &tilesets
                .as_ref()
                .ok_or_else(|| "Tilesets not loaded".to_string())?[map.tileset_id as usize - 1];

            tileset_name = tileset.tileset_name.clone();
            autotile_names = tileset.autotile_names.clone();

            event_names = map
                .events
                .values()
                .map(|e| {
                    (
                        e.pages[0].graphic.character_name.clone(),
                        e.pages[0].graphic.character_hue,
                    )
                })
                .collect();

            fog_name = tileset.fog_name.clone();
            fog_hue = tileset.fog_hue;
            fog_zoom = tileset.fog_zoom;

            pano_name = tileset.panorama_name.clone();
            pano_hue = tileset.panorama_hue;
        }

        // Load tileset textures.
        let tileset_tex =
            load_image_software(format!("Graphics/Tilesets/{}", tileset_name), info).await?;

        // Create an async iter over the autotile textures.
        let autotile_texs_iter = autotile_names.iter().map(|str| async move {
            load_image_software(format!("Graphics/Autotiles/{}", str), info)
                .await
                .ok()
        });

        // Await all the futures.
        let autotile_texs = futures::future::join_all(autotile_texs_iter).await;

        // Similar deal as to the autotiles.
        let event_texs_iter = event_names.iter().map(|(char_name, hue)| async move {
            (
                (char_name.clone(), *hue),
                load_image_software(format!("Graphics/Characters/{}", char_name), info)
                    .await
                    .ok(),
            )
        });

        // Unfortunately since join_all produces a vec, we need to convert it to a hashmap.
        let event_texs: HashMap<_, _> = futures::future::join_all(event_texs_iter)
            .await
            .into_iter()
            .collect();

        // These two are pretty simple.
        let fog_tex = load_image_software(format!("Graphics/Fogs/{}", fog_name), info)
            .await
            .ok();

        let pano_tex = load_image_software(format!("Graphics/Panoramas/{}", pano_name), info)
            .await
            .ok();

        // Finally create and return the struct.
        Ok(Textures {
            autotile_texs,
            tileset_tex,
            event_texs,
            fog_tex,
            fog_zoom,
            pano_tex,
        })
    }
}
