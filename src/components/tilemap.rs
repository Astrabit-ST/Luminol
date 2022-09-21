use egui::Vec2;

use crate::data::rmxp_structs::rpg;

pub struct Tilemap {
    pan: Vec2,
    pub scale: u8,
    pub visible_display: bool,
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            pan: Vec2::ZERO,
            scale: 100,
            visible_display: false,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, map: &mut rpg::Map) {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let response = ui.interact(
            canvas_rect,
            egui::Id::new("map_canvas"),
            egui::Sense::click_and_drag(),
        );

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

        let tile_size = 32.;
        let scale = self.scale as f32 / 100.;
        let canvas_pos = canvas_center + self.pan;

        let width2 = (map.width / 2) as f32;
        let height2 = (map.height / 2) as f32;

        let pos = egui::Vec2::new(
            width2 * tile_size * scale,
            height2 as f32 * tile_size * scale,
        );
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        ui.painter().rect_stroke(
            map_rect,
            5.0,
            egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
        );

        // TODO: Draw map code

        if self.visible_display {
            let width2: f32 = 20. / 2.;
            let height2: f32 = 14. / 2.;

            let pos = egui::Vec2::new(width2 * tile_size * scale, height2 * tile_size * scale);
            let visible_rect = egui::Rect {
                min: canvas_center - pos,
                max: canvas_center + pos,
            };

            ui.painter().rect_stroke(
                visible_rect,
                5.0,
                egui::Stroke::new(1.0, egui::Color32::BLUE),
            );
        }
    }
}
