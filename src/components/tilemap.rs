use std::cell::RefMut;

use egui::{Pos2, Vec2};
use egui_extras::RetainedImage;
use ndarray::Axis;

use crate::data::rmxp_structs::rpg;

pub struct Tilemap {
    pan: Vec2,
    pub scale: f32,
    pub visible_display: bool,
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            scale: 100.,
            visible_display: false,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &mut rpg::Map,
        map_id: i32,
        tileset_tex: RefMut<'_, RetainedImage>,
    ) {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let response = ui.interact(
            canvas_rect,
            egui::Id::new(format!("map_canvas_{}", map_id)),
            egui::Sense::click_and_drag(),
        );

        // Handle zoom
        if response.hovered() {
            self.scale *= 10.0;
            self.scale += ui.input().scroll_delta.y * 5.0;
            self.scale /= 10.0;
            self.scale = 0.1_f32.max(self.scale);
        }

        // Handle pan
        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input().modifiers.command && response.dragged_by(egui::PointerButton::Primary));
        if panning_map_view {
            self.pan += response.drag_delta();
        }

        // Handle cursor
        if panning_map_view {
            response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else {
            response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        let scale = self.scale / 100.;
        let tile_size = 32. * scale;
        let canvas_pos = canvas_center + self.pan;

        let xsize = map.data.len_of(Axis(2));
        let ysize = map.data.len_of(Axis(1));

        let tile_width = 32. / tileset_tex.width() as f32;
        let tile_height = 32. / tileset_tex.height() as f32;

        let width2 = (map.width / 2) as f32;
        let height2 = (map.height / 2) as f32 + 0.5;

        let pos = egui::Vec2::new(width2 * tile_size, height2 as f32 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        // Iterate through all tiles.
        for (idx, ele) in map.data.iter().enumerate() {
            // We grab the x and y through some simple means.
            let (x, y) = (
                // We reset the x every xsize elements.
                idx % xsize,
                // We reset the y every ysize elements, but only increment it every xsize elements.
                (idx / xsize) % ysize,
            );

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
            }
        }

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(3.0, egui::Color32::DARK_GRAY),
        );

        if self.visible_display {
            let width2: f32 = 20. / 2.;
            let height2: f32 = 14. / 2.;

            let pos = egui::Vec2::new(width2 * tile_size, height2 * tile_size);
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
    }
}
