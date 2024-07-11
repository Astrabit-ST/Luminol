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

use luminol_graphics::frame::{FRAME_HEIGHT, FRAME_WIDTH};

pub struct AnimationFrameView {
    pub frame: luminol_graphics::Frame,

    pub pan: egui::Vec2,

    pub scale: f32,
    pub previous_scale: f32,

    pub data_id: egui::Id,
}

impl AnimationFrameView {
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        animation: &luminol_data::rpg::Animation,
        frame_index: usize,
    ) -> color_eyre::Result<AnimationFrameView> {
        let data_id = egui::Id::new("luminol_animation_frame_view").with(
            update_state
                .project_config
                .as_ref()
                .expect("project not loaded")
                .project
                .persistence_id,
        );
        let (pan, scale) = update_state
            .ctx
            .data_mut(|d| *d.get_persisted_mut_or_insert_with(data_id, || (egui::Vec2::ZERO, 50.)));

        let frame = luminol_graphics::Frame::new(
            &update_state.graphics,
            update_state.graphics.atlas_loader.load_animation_atlas(
                &update_state.graphics,
                update_state.filesystem,
                animation,
            )?,
            animation,
            frame_index,
        );

        Ok(Self {
            frame,
            pan,
            scale,
            previous_scale: scale,
            data_id,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &luminol_core::UpdateState<'_>,
        clip_rect: egui::Rect,
    ) -> egui::Response {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect.intersect(clip_rect));

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        let min_clip = (ui.ctx().screen_rect().min - canvas_rect.min).max(Default::default());
        let max_clip = (canvas_rect.max - ui.ctx().screen_rect().max).max(Default::default());
        let clip_offset = (max_clip - min_clip) / 2.;
        let canvas_rect = ui.ctx().screen_rect().intersect(canvas_rect);

        // If the user changed the scale using the scale slider, pan the map so that the scale uses
        // the center of the visible part of the map as the scale center
        if self.scale != self.previous_scale {
            self.pan = self.pan * self.scale / self.previous_scale;
        }

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = self.scale;
            let delta = ui.input(|i| i.smooth_scroll_delta.y);

            // Apply scroll and cap max zoom to 15%
            self.scale *= (delta / 9.0f32.exp2()).exp2();
            self.scale = self.scale.max(15.).min(300.);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * self.scale;
        }

        self.previous_scale = self.scale;

        let ctrl_drag =
            ui.input(|i| i.modifiers.command) && response.dragged_by(egui::PointerButton::Primary);

        let panning_map_view = response.dragged_by(egui::PointerButton::Middle) || ctrl_drag;

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
        let scale = self.scale / (ui.ctx().pixels_per_point() * 100.);

        // no idea why this math works (could probably be simplified)
        let proj_center_x = -(self.pan.x + clip_offset.x) / scale;
        let proj_center_y = -(self.pan.y + clip_offset.y) / scale;
        let proj_width2 = canvas_rect.width() / scale / 2.;
        let proj_height2 = canvas_rect.height() / scale / 2.;
        self.frame.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(canvas_rect.width(), canvas_rect.height()),
            glam::vec2(proj_width2 - proj_center_x, proj_height2 - proj_center_y) * scale,
            glam::Vec2::splat(scale),
        );

        let painter = luminol_graphics::Painter::new(self.frame.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                painter,
            ));

        let offset = canvas_center.to_vec2() + self.pan;

        // Draw the grid lines and the border of the animation frame
        ui.painter().line_segment(
            [
                egui::pos2(-(FRAME_WIDTH as f32 / 2.), 0.) * scale + offset,
                egui::pos2(FRAME_WIDTH as f32 / 2., 0.) * scale + offset,
            ],
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );
        ui.painter().line_segment(
            [
                egui::pos2(0., -(FRAME_HEIGHT as f32 / 2.)) * scale + offset,
                egui::pos2(0., FRAME_HEIGHT as f32 / 2.) * scale + offset,
            ],
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );
        ui.painter().rect_stroke(
            egui::Rect::from_center_size(
                offset.to_pos2(),
                egui::vec2(FRAME_WIDTH as f32, FRAME_HEIGHT as f32) * scale,
            ),
            5.,
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );

        // Draw a white rectangle on the border of every cell
        for (_, (_, cell_rect)) in self.frame.sprites.iter() {
            ui.painter().rect_stroke(
                (*cell_rect * scale).translate(offset),
                5.,
                egui::Stroke::new(1., egui::Color32::WHITE),
            );
        }

        ui.ctx().data_mut(|d| {
            d.insert_persisted(self.data_id, (self.pan, self.scale));
        });

        response
    }
}
