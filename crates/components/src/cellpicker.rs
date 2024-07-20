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

use luminol_graphics::Renderable;

use luminol_graphics::primitives::cells::{Atlas, CELL_SIZE};
use luminol_graphics::{Cells, Transform, Viewport};

pub struct Cellpicker {
    pub selected_cell: u32,
    pub viewport: Viewport,
    pub view: Cells,
}

impl Cellpicker {
    pub fn new(graphics_state: &luminol_graphics::GraphicsState, atlas: Atlas) -> Self {
        let cells = luminol_data::Table2::new_data(
            atlas.num_patterns() as usize,
            1,
            (0..atlas.num_patterns() as i16).collect(),
        );

        let viewport = Viewport::new(
            graphics_state,
            glam::vec2((atlas.num_patterns() * CELL_SIZE) as f32, CELL_SIZE as f32) / 2.,
        );

        let view = Cells::new(
            graphics_state,
            &cells,
            atlas,
            &viewport,
            Transform::unit(graphics_state),
        );

        Self {
            selected_cell: 0,
            viewport,
            view,
        }
    }

    pub fn ui(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        ui: &mut egui::Ui,
        scroll_rect: egui::Rect,
    ) -> egui::Response {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(
                (self.view.atlas.num_patterns() * CELL_SIZE) as f32,
                CELL_SIZE as f32,
            ) / 2.,
            egui::Sense::click_and_drag(),
        );

        let absolute_scroll_rect = ui
            .ctx()
            .screen_rect()
            .intersect(scroll_rect.translate(canvas_rect.min.to_vec2()));
        let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());

        self.view.transform.set_position(
            &update_state.graphics.render_state,
            glam::vec2(-scroll_rect.left() * 2., 0.),
        );
        self.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(scroll_rect.width(), scroll_rect.height()),
            glam::Vec2::ZERO,
            glam::Vec2::splat(0.5),
        );

        let painter = luminol_graphics::Painter::new(self.view.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                absolute_scroll_rect,
                painter,
            ));

        let rect = (egui::Rect::from_min_size(
            egui::pos2((self.selected_cell * CELL_SIZE) as f32, 0.),
            egui::Vec2::splat(CELL_SIZE as f32),
        ) / 2.)
            .translate(canvas_rect.min.to_vec2());
        ui.painter()
            .rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                self.selected_cell =
                    ((pos - canvas_rect.min) / CELL_SIZE as f32 * 2.).x.floor() as u32;
            }
        }

        self.selected_cell = self
            .selected_cell
            .min(self.view.atlas.num_patterns().saturating_sub(1));

        response
    }
}
