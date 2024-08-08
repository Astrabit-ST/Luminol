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
    pub cols: u32,
    pub scale: f32,
}

impl Cellpicker {
    pub fn new(
        graphics_state: &luminol_graphics::GraphicsState,
        atlas: Atlas,
        cols: Option<u32>,
        scale: f32,
    ) -> Self {
        let cols = cols.unwrap_or(atlas.num_patterns());
        let rows = (atlas.num_patterns()).div_ceil(cols);
        let cells = luminol_data::Table2::new_data(
            cols as usize,
            rows as usize,
            (0..(rows * cols) as i16).collect(),
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
            cols,
            scale,
        }
    }

    #[inline]
    pub fn rows(&self) -> u32 {
        self.view.atlas.num_patterns().div_ceil(self.cols)
    }

    pub fn ui(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        ui: &mut egui::Ui,
        scroll_rect: egui::Rect,
    ) -> egui::Response {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(
                (self.cols * CELL_SIZE) as f32,
                (self.rows() * CELL_SIZE) as f32,
            ) * self.scale,
            egui::Sense::click_and_drag(),
        );

        let absolute_scroll_rect = ui
            .ctx()
            .screen_rect()
            .intersect(scroll_rect.translate(canvas_rect.min.to_vec2()));
        let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());

        self.view.transform.set_position(
            &update_state.graphics.render_state,
            glam::vec2(-scroll_rect.left(), -scroll_rect.top()) / self.scale,
        );
        self.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(scroll_rect.width(), scroll_rect.height()),
            glam::Vec2::ZERO,
            glam::Vec2::splat(self.scale),
        );

        let painter = luminol_graphics::Painter::new(self.view.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                absolute_scroll_rect,
                painter,
            ));

        let rect = (egui::Rect::from_min_size(
            egui::pos2(
                ((self.selected_cell % self.cols) * CELL_SIZE) as f32,
                ((self.selected_cell / self.cols) * CELL_SIZE) as f32,
            ),
            egui::Vec2::splat(CELL_SIZE as f32),
        ) * self.scale)
            .translate(canvas_rect.min.to_vec2());
        ui.painter()
            .rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let mapped_pos = (pos - canvas_rect.min) / (CELL_SIZE as f32 * self.scale);
                self.selected_cell = mapped_pos.x as u32 + mapped_pos.y as u32 * self.cols;
            }
        }

        self.selected_cell = self
            .selected_cell
            .min(self.view.atlas.num_patterns().saturating_sub(1));

        response
    }
}
