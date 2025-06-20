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

use luminol_graphics::troop::{TROOP_HEIGHT, TROOP_WIDTH};

pub struct TroopView {
    pub troop: luminol_graphics::Troop,

    pub selected_member_index: Option<usize>,
    pub hovered_member_index: Option<usize>,
    pub hovered_member_drag_pos: Option<(i32, i32)>,
    pub hovered_member_drag_offset: Option<egui::Vec2>,

    pub pan: egui::Vec2,

    pub scale: f32,
    previous_scale: f32,

    pub data_id: egui::Id,
}

impl TroopView {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let data_id = egui::Id::new("luminol_troop_view").with(
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

        Self {
            troop: luminol_graphics::Troop::new(&update_state.graphics),
            selected_member_index: None,
            hovered_member_index: None,
            hovered_member_drag_pos: None,
            hovered_member_drag_offset: None,
            pan,
            scale,
            previous_scale: scale,
            data_id,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &luminol_core::UpdateState<'_>,
        clip_rect: egui::Rect,
    ) -> egui::InnerResponse<Option<(i32, i32)>> {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect.intersect(clip_rect));

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Take focus when this view is interacted with so the map editor doesn't receive
        // keypresses if it's also open at the same time
        if response.clicked() || response.double_clicked() || response.dragged() {
            response.request_focus();
        }

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
            self.scale = self.scale.clamp(15., 300.);

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

        self.troop.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(canvas_rect.width(), canvas_rect.height()),
            glam::vec2(
                canvas_rect.width() / 2. + self.pan.x + clip_offset.x,
                canvas_rect.height() / 2. + self.pan.y + clip_offset.y,
            ),
            glam::Vec2::splat(scale),
        );

        let painter = luminol_graphics::Painter::new(self.troop.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                canvas_rect,
                painter,
            ));

        let offset = canvas_center.to_vec2() + self.pan;

        // Draw the grid lines and the border of the view
        ui.painter().line_segment(
            [
                egui::pos2(-(TROOP_WIDTH as f32 / 2.), 0.) * scale + offset,
                egui::pos2(TROOP_WIDTH as f32 / 2., 0.) * scale + offset,
            ],
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );
        ui.painter().line_segment(
            [
                egui::pos2(0., -(TROOP_HEIGHT as f32 / 2.)) * scale + offset,
                egui::pos2(0., TROOP_HEIGHT as f32 / 2.) * scale + offset,
            ],
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );
        ui.painter().rect_stroke(
            egui::Rect::from_center_size(
                offset.to_pos2(),
                egui::vec2(TROOP_WIDTH as f32, TROOP_HEIGHT as f32) * scale,
            ),
            5.,
            egui::Stroke::new(1., egui::Color32::DARK_GRAY),
        );

        // Find the troop member that the cursor is hovering over; if multiple are hovered we
        // prioritize the one with the lowest index
        if response.clicked() {
            self.selected_member_index = None;
        }
        if self.hovered_member_drag_offset.is_none() {
            self.hovered_member_index = ui
                .input(|i| !i.modifiers.shift)
                .then(|| {
                    self.troop
                        .members()
                        .iter()
                        .map(|(i, member)| (i, (member.rect * scale).translate(offset)))
                        .find_map(|(i, rect)| {
                            (response.hovered() && ui.rect_contains_pointer(rect)).then(|| {
                                if response.clicked() {
                                    // If the hovered member was clicked, make it the selected
                                    // member
                                    self.selected_member_index = Some(i);
                                }
                                i
                            })
                        })
                })
                .flatten();
        }

        let hover_pos_in_troop_coords = response.hover_pos().map(|pos| (pos - offset) / scale);

        if !response.is_pointer_button_down_on()
            || ui.input(|i| {
                !i.pointer.button_down(egui::PointerButton::Primary) || i.modifiers.shift
            })
        {
            self.hovered_member_drag_offset = None;
        } else if let (Some(member), None, true) = (
            self.hovered_member_index
                .and_then(|i| self.troop.members().get(i)),
            self.hovered_member_drag_offset,
            response.drag_started_by(egui::PointerButton::Primary),
        ) {
            self.hovered_member_drag_offset = Some(
                member.rect.center_bottom()
                    + egui::vec2(TROOP_WIDTH as f32 / 2., TROOP_HEIGHT as f32 / 2.)
                    - hover_pos_in_troop_coords.unwrap(),
            );
        }

        if let Some(drag_offset) = self.hovered_member_drag_offset {
            let pos = hover_pos_in_troop_coords.unwrap() + drag_offset;
            self.hovered_member_drag_pos = Some((
                pos.x.clamp(0., TROOP_WIDTH as f32).round_ties_even() as i32,
                pos.y.clamp(0., TROOP_HEIGHT as f32).round_ties_even() as i32,
            ));
        } else {
            self.hovered_member_drag_pos = None;
        }

        // Draw a rectangle on the border of every troop member
        for rect in self
            .troop
            .members()
            .iter()
            .map(|(_, member)| (member.rect * scale).translate(offset))
        {
            ui.painter().rect_stroke(
                rect,
                5.,
                egui::Stroke::new(
                    1.,
                    if ui.input(|i| i.modifiers.shift) {
                        egui::Color32::DARK_GRAY
                    } else {
                        egui::Color32::WHITE
                    },
                ),
            );
        }

        // Draw a yellow rectangle on the border of the hovered member
        if let Some(member) = self
            .hovered_member_index
            .and_then(|i| self.troop.members().get(i))
        {
            let rect = (member.rect * scale).translate(offset);
            ui.painter()
                .rect_stroke(rect, 5., egui::Stroke::new(3., egui::Color32::YELLOW));
        }

        // Draw a magenta rectangle on the border of the selected member
        if let Some(member) = self
            .selected_member_index
            .and_then(|i| self.troop.members().get(i))
        {
            let rect = (member.rect * scale).translate(offset);
            ui.painter().rect_stroke(
                rect,
                5.,
                egui::Stroke::new(3., egui::Color32::from_rgb(255, 0, 255)),
            );
        }

        ui.ctx().data_mut(|d| {
            d.insert_persisted(self.data_id, (self.pan, self.scale));
        });

        egui::InnerResponse::new(
            hover_pos_in_troop_coords.map(|pos| {
                let pos = pos + egui::vec2(TROOP_WIDTH as f32 / 2., TROOP_HEIGHT as f32 / 2.);
                (
                    pos.x.clamp(0., TROOP_WIDTH as f32).round_ties_even() as i32,
                    pos.y.clamp(0., TROOP_HEIGHT as f32).round_ties_even() as i32,
                )
            }),
            response,
        )
    }
}
