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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

pub struct Modal {
    state: State,
    id_source: egui::Id,
    pub frames_len: usize,
    pub new_frames_len: usize,
}

enum State {
    Closed,
    Open,
}

impl Modal {
    pub fn new(id_source: impl Into<egui::Id>) -> Self {
        Self {
            state: State::Closed,
            id_source: id_source.into(),
            frames_len: 1,
            new_frames_len: 1,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data<'m> = ();

    fn button<'m>(
        &'m mut self,
        _data: Self::Data<'m>,
        _update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| {
            let response = ui.button("Change frame count");
            if response.clicked() {
                self.state = State::Open;
            }
            response
        }
    }

    fn reset(&mut self, _: &mut luminol_core::UpdateState<'_>, _data: Self::Data<'_>) {
        self.close_window();
    }
}

impl Modal {
    pub fn close_window(&mut self) {
        self.state = State::Closed;
    }

    pub fn show_window(&mut self, ctx: &egui::Context, frames_len: usize) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        if !matches!(self.state, State::Open) {
            self.frames_len = frames_len;
            self.new_frames_len = frames_len;
            return false;
        }

        egui::Window::new("Change Frame Count")
            .open(&mut win_open)
            .id(self.id_source.with("change_frame_count_tool"))
            .show(ctx, |ui| {
                ui.add(luminol_components::Field::new(
                    "Frame Count",
                    egui::DragValue::new(&mut self.new_frames_len).range(1..=usize::MAX),
                ));

                if self.frames_len <= 999 && self.new_frames_len > 999 {
                    egui::Frame::none().show(ui, |ui| {
                        ui.style_mut()
                            .visuals
                            .widgets
                            .noninteractive
                            .bg_stroke
                            .color = ui.style().visuals.warn_fg_color;
                        egui::Frame::group(ui.style())
                            .fill(ui.visuals().gray_out(ui.visuals().gray_out(
                                ui.visuals().gray_out(ui.style().visuals.warn_fg_color),
                            )))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(egui::RichText::new("Setting the frame count above 999 may introduce performance issues and instability").color(ui.style().visuals.warn_fg_color));
                            });
                    });
                }

                ui.label(format!("Change the number of frames in this animation from {} to {}", self.frames_len, self.new_frames_len));

                luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
            });

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }
        needs_save
    }
}
