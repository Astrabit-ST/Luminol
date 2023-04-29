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

use egui::Color32;
use egui_extras::RetainedImage;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use egui::{Pos2, Response, Vec2};

use crate::state;
use rmxp_types::rpg;

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
    /// Toggle move route preview
    pub move_preview: bool,
    ani_idx: i32,
    ani_instant: Instant,
    textures: Textures,
}

pub struct Textures {
    tileset_tex: Option<Arc<RetainedImage>>,
    autotile_texs: Vec<Option<Arc<RetainedImage>>>,
    event_texs: HashMap<(String, i32), Option<Arc<RetainedImage>>>,
    fog_tex: Option<Arc<RetainedImage>>,
    fog_zoom: i32,
    pano_tex: Option<Arc<RetainedImage>>,
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
impl Tilemap {
    fn new(id: i32) -> Result<Tilemap, String> {
        let textures = Self::load_data(id)?;
        Ok(Self {
            pan: Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,
            ani_idx: 0,
            ani_instant: Instant::now(),
            textures,
        })
    }

    fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
        dragging_event: bool,
    ) -> Response {
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
            let tile_size = (self.scale / 100.) * 32.;
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0.0, map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0.0, map.height as f32 - 1.);
            // Handle input
            if selected_layer < map.data.zsize() || dragging_event || response.clicked() {
                *cursor_pos = pos_tile.to_pos2();
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
        let scale = self.scale / 100.;
        let tile_size = 32. * scale;
        let at_tile_size = 16. * scale;
        let canvas_pos = canvas_center + self.pan;

        let xsize = map.data.xsize();
        let ysize = map.data.ysize();

        let tile_width = self
            .textures
            .tileset_tex
            .as_ref()
            .map(|t| 32. / t.width() as f32);
        let tile_height = self
            .textures
            .tileset_tex
            .as_ref()
            .map(|t| 32. / t.height() as f32);

        let width2 = map.width as f32 / 2.;
        let height2 = map.height as f32 / 2.;

        let pos = egui::Vec2::new(width2 * tile_size, height2 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        // Do we need to render a panorama?
        if toggled_layers[map.data.zsize() + 1] {
            if let Some(pano_tex) = &self.textures.pano_tex {
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

                        egui::Image::new(
                            pano_tex.texture_id(ui.ctx()),
                            pano_tex.size_vec2() * scale,
                        )
                        .paint_at(ui, pano_rect);
                    }
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
                if let Some(ref tileset_tex) = self.textures.tileset_tex {
                    // Normalize element
                    let ele = ele - 384;

                    // Calculate UV coordinates
                    let tile_x =
                        (ele as usize % (tileset_tex.width() / 32)) as f32 * tile_width.unwrap();
                    let tile_y =
                        (ele as usize / (tileset_tex.width() / 32)) as f32 * tile_height.unwrap();

                    let uv = egui::Rect::from_min_size(
                        Pos2::new(tile_x, tile_y),
                        egui::vec2(tile_width.unwrap(), tile_height.unwrap()),
                    );

                    // Display tile
                    egui::Image::new(tileset_tex.texture_id(ui.ctx()), tileset_tex.size_vec2())
                        .uv(uv)
                        .paint_at(ui, tile_rect);
                }
            } else {
                // holy shit
                // Find what autotile we're displaying
                let autotile_id = (ele / 48) - 1;

                if let Some(autotile_tex) = &self.textures.autotile_texs[autotile_id as usize] {
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
                let Some(tex) =self. textures
                    .event_texs
                    .get(&(graphic.character_name.clone(), graphic.character_hue))
                    else {
                        continue;
                    };
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
                    if let Some(ref tileset_tex) = self.textures.tileset_tex {
                        // Same logic for tiles. See above.
                        let tile_rect = egui::Rect::from_min_size(
                            map_rect.min
                                + egui::Vec2::new(
                                    event.x as f32 * tile_size,
                                    event.y as f32 * tile_size,
                                ),
                            egui::Vec2::splat(tile_size),
                        );

                        let tile_x = ((graphic.tile_id - 384) as usize % (tileset_tex.width() / 32))
                            as f32
                            * tile_width.unwrap();
                        let tile_y = ((graphic.tile_id - 384) as usize / (tileset_tex.width() / 32))
                            as f32
                            * tile_height.unwrap();

                        let uv = egui::Rect::from_min_size(
                            Pos2::new(tile_x, tile_y),
                            egui::vec2(tile_width.unwrap(), tile_height.unwrap()),
                        );

                        egui::Image::new(tileset_tex.texture_id(ui.ctx()), tileset_tex.size_vec2())
                            .uv(uv)
                            .paint_at(ui, tile_rect);
                    }
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
        if toggled_layers[map.data.zsize() + 2] {
            if let Some(fog_tex) = &self.textures.fog_tex {
                let zoom = (self.textures.fog_zoom as f32 / 100.) * scale;
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
        }

        if self.move_preview {
            for (_id, event) in map.events.iter() {
                for (page_index, page) in event
                    .pages
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.move_type == 3)
                {
                    let _move_route = &page.move_route;

                    let _directions = vec![page.graphic.direction];
                    let mut points = vec![egui::pos2(event.x as f32, event.y as f32)];
                    // process_move_route(move_route, &mut directions, &mut points);

                    points = points
                        .iter_mut()
                        .map(|p| {
                            map_rect.min
                                + (p.to_vec2() * tile_size)
                                + egui::Vec2::splat(tile_size / 2.)
                        })
                        .collect();

                    let stroke = egui::Stroke::new(
                        1.0,
                        match page_index {
                            0 => Color32::YELLOW,
                            1 => Color32::BLUE,
                            2 => Color32::WHITE,
                            3 => Color32::GREEN,
                            _ => Color32::RED,
                        },
                    );

                    let mut iter = points.into_iter().peekable();
                    while let Some(p) = iter.next() {
                        if let Some(p2) = iter.peek() {
                            ui.painter().arrow(p, *p2 - p, stroke)
                        }
                    }
                }
            }
        }

        if let Some((direction, _route)) = &map.preview_move_route {
            let _directions = vec![*direction];
            let mut points = vec![*cursor_pos];
            // process_move_route(route, &mut directions, &mut points);

            points = points
                .iter_mut()
                .map(|p| {
                    map_rect.min + (p.to_vec2() * tile_size) + egui::Vec2::splat(tile_size / 2.)
                })
                .collect();

            let stroke = egui::Stroke::new(1.0, Color32::YELLOW);

            let mut iter = points.into_iter().peekable();
            while let Some(p) = iter.next() {
                if let Some(p2) = iter.peek() {
                    ui.painter().arrow(p, *p2 - p, stroke)
                } else {
                    ui.painter().rect_stroke(
                        egui::Rect::from_min_size(
                            p - egui::Vec2::splat(tile_size / 2.),
                            egui::Vec2::splat(tile_size),
                        ),
                        5.0,
                        stroke,
                    )
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
        if let Some(ref tileset_tex) = self.textures.tileset_tex {
            let (rect, response) =
                ui.allocate_exact_size(tileset_tex.size_vec2(), egui::Sense::click());

            egui::Image::new(tileset_tex.texture_id(ui.ctx()), rect.size()).paint_at(ui, rect);

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
                    rect.min + egui::vec2(f32::from(cursor_x), f32::from(cursor_y)),
                    egui::Vec2::splat(32.),
                ),
                5.0,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        }
    }

    #[allow(unused_variables, unused_assignments)]
    fn load_data(id: i32) -> Result<Textures, String> {
        let state = state!();
        // Load the map.

        let map = state.data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        // Load tileset textures.
        let tileset_tex = state
            .image_cache
            .load_egui_image("Graphics/Tilesets", &tileset.tileset_name)
            .ok();

        // Create an async iter over the autotile textures.
        let autotile_texs = tileset
            .autotile_names
            .iter()
            .map(|path| {
                state
                    .image_cache
                    .load_egui_image("Graphics/Autotiles", path)
                    .ok()
            })
            .collect();

        // Await all the futures.

        let event_texs = map
            .events
            .iter()
            .map(|(_, e)| {
                (
                    e.pages[0].graphic.character_name.clone(),
                    e.pages[0].graphic.character_hue,
                )
            })
            .dedup()
            .map(|(char_name, hue)| {
                let texture = state
                    .image_cache
                    .load_egui_image("Graphics/Characters", &char_name)
                    .ok();
                ((char_name, hue), texture)
            })
            .collect();

        // These two are pretty simple.
        let fog_tex = state
            .image_cache
            .load_egui_image("Graphics/Fogs", &tileset.fog_name)
            .ok();
        let fog_zoom = tileset.fog_zoom;

        let pano_tex = state
            .image_cache
            .load_egui_image("Graphics/Panoramas", &tileset.panorama_name)
            .ok();

        // Finally create and return the struct.
        Ok(Textures {
            tileset_tex,
            autotile_texs,
            event_texs,
            fog_tex,
            fog_zoom,
            pano_tex,
        })
    }
}
