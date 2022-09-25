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

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use egui::{Pos2, Response, Vec2};
use egui_extras::RetainedImage;
use ndarray::Axis;

use crate::data::rmxp_structs::rpg;

#[allow(dead_code)]
pub struct Tilemap {
    pub pan: Vec2,
    pub scale: f32,
    pub visible_display: bool,
    ani_idx: i32,
    ani_instant: Instant,
}

#[allow(dead_code)]
impl Tilemap {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            ani_idx: 0,
            ani_instant: Instant::now(),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut Pos2,
        tileset_tex: &RetainedImage,
        autotile_texs: &[Option<RetainedImage>],
        event_texs: &HashMap<String, Option<RetainedImage>>,
        toggled_layers: &Vec<bool>,
        selected_layer: usize,
    ) -> Response {
        if self.ani_instant.elapsed() >= Duration::from_secs_f32((1. / 60.) * 16.) {
            self.ani_instant = Instant::now();
            self.ani_idx += 1;
        }

        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Handle zoom
        if response.hovered() {
            let delta = ui.input().scroll_delta.y * 5.0;

            if let Some(pos) = response.hover_pos() {
                let old_scale = self.scale;

                self.scale += delta / 30.;
                self.scale = 15.0_f32.max(self.scale);

                let pos_norm = (pos - self.pan - canvas_center) / old_scale;
                self.pan = pos - canvas_center - pos_norm * self.scale;

                let tile_size = (self.scale / 100.) * 32.;
                let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                    + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
                pos_tile.x = pos_tile.x.floor().max(0.0).min(map.width as f32 - 1.);
                pos_tile.y = pos_tile.y.floor().max(0.0).min(map.height as f32 - 1.);
                if selected_layer < map.data.len_of(Axis(0)) {
                    *cursor_pos = pos_tile.to_pos2();
                } else if response.clicked() {
                    *cursor_pos = pos_tile.to_pos2();
                }
            }
        }

        // Handle pan
        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary));
        if panning_map_view {
            self.pan += response.drag_delta();
        }

        // Handle cursor
        if panning_map_view {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        let scale = self.scale / 100.;
        let tile_size = 32. * scale;
        let at_tile_size = 16. * scale;
        let canvas_pos = canvas_center + self.pan;

        let xsize = map.data.len_of(Axis(2));
        let ysize = map.data.len_of(Axis(1));

        let tile_width = 32. / tileset_tex.width() as f32;
        let tile_height = 32. / tileset_tex.height() as f32;

        let width2 = map.width as f32 / 2.;
        let height2 = map.height as f32 / 2.;

        let pos = egui::Vec2::new(width2 * tile_size, height2 as f32 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

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

            if !toggled_layers[z] {
                continue;
            }

            let tile_rect = egui::Rect::from_min_size(
                map_rect.min + egui::Vec2::new(x as f32 * tile_size, y as f32 * tile_size),
                egui::Vec2::splat(tile_size),
            );

            if *ele >= 384 {
                let ele = ele - 384;

                let tile_x = (ele as usize % (tileset_tex.width() / 32)) as f32 * tile_width;
                let tile_y = (ele as usize / (tileset_tex.width() / 32)) as f32 * tile_height;

                let uv = egui::Rect::from_min_size(
                    Pos2::new(tile_x, tile_y),
                    egui::vec2(tile_width, tile_height),
                );

                egui::Image::new(tileset_tex.texture_id(ui.ctx()), tileset_tex.size_vec2())
                    .uv(uv)
                    .paint_at(ui, tile_rect);
            } else {
                // holy shit
                let autotile_id = (ele / 48) - 1;

                if let Some(autotile_tex) = &autotile_texs[autotile_id as usize] {
                    let tile_width = 16. / autotile_tex.width() as f32;
                    let tile_height = 16. / autotile_tex.height() as f32;

                    for s_a in 0..2 {
                        for s_b in 0..2 {
                            let tile_rect = egui::Rect::from_min_size(
                                map_rect.min
                                    + egui::Vec2::new(
                                        (x as f32 * tile_size) + (s_a as f32 * at_tile_size),
                                        (y as f32 * tile_size) + (s_b as f32 * at_tile_size),
                                    ),
                                egui::Vec2::splat(at_tile_size),
                            );

                            let ti = AUTOTILES[*ele as usize % 48][s_a + (s_b * 2)];

                            let tx = ti % 6;
                            let tx_off = (self.ani_idx as usize % (autotile_tex.width() / 96)) * 6;
                            let tx = (tx + tx_off as i32) as f32 * tile_width;

                            let ty = (ti / 6) as f32 * tile_height;

                            let uv = egui::Rect::from_min_size(
                                Pos2::new(tx, ty),
                                egui::vec2(tile_width, tile_height),
                            );

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

        if *toggled_layers.last().unwrap() {
            for (_, event) in map.events.iter() {
                // aaaaaaaa
                let graphic = &event.pages[0].graphic;
                let tex = event_texs.get(&graphic.character_name).unwrap();
                if let Some(tex) = tex {
                    let cw = (tex.width() / 4) as f32;
                    let ch = (tex.height() / 4) as f32;

                    let c_rect = egui::Rect::from_min_size(
                        map_rect.min
                            + egui::Vec2::new(
                                (event.x as f32 * tile_size) + ((16. - (cw / 2.)) * scale),
                                (event.y as f32 * tile_size) + ((32. - ch) * scale),
                            ),
                        egui::vec2(cw * scale, ch * scale),
                    );

                    let cx = (graphic.pattern as f32 * cw) / tex.width() as f32;
                    let cy = (((graphic.direction - 2) / 2) as f32 * ch) / tex.height() as f32;

                    let uv = egui::Rect::from_min_size(
                        Pos2::new(cx, cy),
                        egui::vec2(cw / tex.width() as f32, ch / tex.height() as f32),
                    );

                    egui::Image::new(tex.texture_id(ui.ctx()), tex.size_vec2())
                        .uv(uv)
                        .paint_at(ui, c_rect);
                } else if graphic.tile_id.is_positive() {
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
                        * tile_width;
                    let tile_y = ((graphic.tile_id - 384) as usize / (tileset_tex.width() / 32))
                        as f32
                        * tile_height;

                    let uv = egui::Rect::from_min_size(
                        Pos2::new(tile_x, tile_y),
                        egui::vec2(tile_width, tile_height),
                    );

                    egui::Image::new(tileset_tex.texture_id(ui.ctx()), tileset_tex.size_vec2())
                        .uv(uv)
                        .paint_at(ui, tile_rect);
                }

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

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::DARK_GRAY),
        );

        if self.visible_display {
            let width2: f32 = (640. / 2.) * scale;
            let height2: f32 = (480. / 2.) * scale;

            let pos = egui::Vec2::new(width2, height2);
            let visible_rect = egui::Rect {
                min: canvas_center - pos,
                max: canvas_center + pos,
            };

            ui.painter().rect_stroke(
                visible_rect,
                5.0,
                egui::Stroke::new(1.0, egui::Color32::YELLOW),
            );
        }

        let cursor_rect = egui::Rect::from_min_size(
            map_rect.min + (cursor_pos.to_vec2() * tile_size),
            Vec2::splat(tile_size),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::YELLOW),
        );

        ui.ctx()
            .request_repaint_after(Duration::from_secs_f32((1. / 60.) * 16.));

        response
    }
}

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
