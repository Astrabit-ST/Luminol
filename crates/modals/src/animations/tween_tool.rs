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
    pub start_frame: usize,
    pub end_frame: usize,
    pub start_cell: usize,
    pub end_cell: usize,
    pub tween_pattern: bool,
    pub tween_position: bool,
    pub tween_shading: bool,
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
            start_frame: 0,
            end_frame: 0,
            start_cell: 0,
            end_cell: 15,
            tween_pattern: true,
            tween_position: true,
            tween_shading: true,
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
            let response = ui.button("Tween");
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
            self.start_frame = frames_len
                .saturating_sub(2)
                .min(current_frame.saturating_sub(1));
            self.end_frame = 2.max(current_frame + 1);
            return false;
        }

        egui::Window::new("Tween")
            .open(&mut win_open)
            .id(self.id_source.with("tween_tool"))
            .show(ctx, |ui| {
                ui.columns(2, |columns| {
                    self.start_frame += 1;
                    columns[0].add(luminol_components::Field::new(
                        "Starting Frame",
                        egui::DragValue::new(&mut self.start_frame)
                            .range(1..=self.frames_len.saturating_sub(2)),
                    ));
                    self.start_frame -= 1;

                    if self.start_frame + 2 > self.end_frame {
                        self.end_frame = self.start_frame + 2;
                    }

                    self.end_frame += 1;
                    columns[1].add(luminol_components::Field::new(
                        "Ending Frame",
                        egui::DragValue::new(&mut self.end_frame).range(3..=self.frames_len),
                    ));
                    self.end_frame -= 1;

                    if self.end_frame - 2 < self.start_frame {
                        self.start_frame = self.end_frame - 2;
                    }
                });

                ui.columns(2, |columns| {
                    self.start_cell += 1;
                    columns[0].add(luminol_components::Field::new(
                        "Starting Cell",
                        egui::DragValue::new(&mut self.start_cell).range(1..=i16::MAX as usize + 1),
                    ));
                    self.start_cell -= 1;

                    if self.start_cell > self.end_cell {
                        self.end_cell = self.start_cell;
                    }

                    self.end_cell += 1;
                    columns[1].add(luminol_components::Field::new(
                        "Ending Cell",
                        egui::DragValue::new(&mut self.end_cell).range(1..=i16::MAX as usize + 1),
                    ));
                    self.end_cell -= 1;

                    if self.end_cell < self.start_cell {
                        self.start_cell = self.end_cell;
                    }
                });

                ui.checkbox(&mut self.tween_pattern, "Pattern");
                ui.checkbox(&mut self.tween_position, "Position, scale and rotation");
                ui.checkbox(&mut self.tween_shading, "Opacity and blending");

                let mut vec = Vec::with_capacity(3);
                if self.tween_pattern {
                    vec.push("pattern");
                }
                if self.tween_position {
                    vec.push("position, scale, rotation");
                }
                if self.tween_shading {
                    vec.push("opacity, blending");
                }
                ui.label(if vec.is_empty() {
                    "Do nothing".to_string()
                } else if self.start_cell == self.end_cell {
                    format!(
                        "Linearly interpolate cell {} for cell {} from frame {} to frame {}",
                        vec.join(", "),
                        self.start_cell + 1,
                        self.start_frame + 1,
                        self.end_frame + 1,
                    )
                } else {
                    format!(
                        "Linearly interpolate cell {} for cells {}–{} from frame {} to frame {}",
                        vec.join(", "),
                        self.start_cell + 1,
                        self.end_cell + 1,
                        self.start_frame + 1,
                        self.end_frame + 1,
                    )
                });

                luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
            });

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }
        needs_save
    }
}
