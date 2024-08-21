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

use crate::components::Field;

pub struct Modal {
    state: State,
    id_source: egui::Id,
    pub frames_len: usize,
    pub src_frame: usize,
    pub dst_frame: usize,
    pub frame_count: usize,
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
            src_frame: 0,
            dst_frame: 0,
            frame_count: 1,
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
            let response = ui.button("Copy frames");
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

    pub fn show_window(
        &mut self,
        ctx: &egui::Context,
        current_frame: usize,
        frames_len: usize,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        if !matches!(self.state, State::Open) {
            self.frames_len = frames_len;
            self.src_frame = current_frame;
            self.dst_frame = current_frame;
            self.frame_count = 1;
            return false;
        }

        egui::Window::new("Copy Frames")
            .open(&mut win_open)
            .id(self.id_source.with("copy_frames_tool"))
            .show(ctx, |ui| {
                ui.columns(3, |columns| {
                    self.src_frame += 1;
                    columns[0].add(Field::new(
                        "Source Frame",
                        egui::DragValue::new(&mut self.src_frame).range(1..=self.frames_len),
                    ));
                    self.src_frame -= 1;

                    self.dst_frame += 1;
                    columns[1].add(Field::new(
                        "Destination Frame",
                        egui::DragValue::new(&mut self.dst_frame).range(1..=self.frames_len),
                    ));
                    self.dst_frame -= 1;

                    columns[2].add(Field::new(
                        "Frame Count",
                        egui::DragValue::new(&mut self.frame_count)
                            .range(1..=self.frames_len - self.src_frame.max(self.dst_frame)),
                    ));
                });

                ui.label(if self.frame_count == 1 {
                    format!(
                        "Copy frame {} to frame {}",
                        self.src_frame + 1,
                        self.dst_frame + 1,
                    )
                } else {
                    format!(
                        "Copy frames {}–{} to frames {}–{}",
                        self.src_frame + 1,
                        self.src_frame + self.frame_count,
                        self.dst_frame + 1,
                        self.dst_frame + self.frame_count,
                    )
                });

                crate::components::close_options_ui(ui, &mut keep_open, &mut needs_save);
            });

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }
        needs_save
    }
}
