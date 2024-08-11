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

use luminol_components::UiExt;
use luminol_core::prelude::frame::{FRAME_HEIGHT, FRAME_WIDTH};
use luminol_data::BlendMode;

pub struct Modal {
    state: State,
    id_source: egui::Id,
    pub mode: Mode,
    pub frames_len: usize,
    pub start_frame: usize,
    pub end_frame: usize,
    pub num_patterns: u32,

    pub set_pattern_enabled: bool,
    pub set_x_enabled: bool,
    pub set_y_enabled: bool,
    pub set_scale_enabled: bool,
    pub set_rotation_enabled: bool,
    pub set_flip_enabled: bool,
    pub set_opacity_enabled: bool,
    pub set_blending_enabled: bool,

    pub set_pattern: i16,
    pub set_x: i16,
    pub set_y: i16,
    pub set_scale: i16,
    pub set_rotation: i16,
    pub set_flip: i16,
    pub set_opacity: i16,
    pub set_blending: i16,

    pub add_pattern: i16,
    pub add_x: i16,
    pub add_y: i16,
    pub add_scale: i16,
    pub add_rotation: i16,
    pub add_flip: bool,
    pub add_opacity: i16,
    pub add_blending: i16,

    pub mul_pattern: f64,
    pub mul_x: f64,
    pub mul_y: f64,
    pub mul_scale: f64,
    pub mul_rotation: f64,
    pub mul_opacity: f64,
}

enum State {
    Closed,
    Open,
}

#[derive(PartialEq, Eq)]
pub enum Mode {
    Set,
    Add,
    Mul,
}

impl Modal {
    pub fn new(id_source: impl Into<egui::Id>) -> Self {
        Self {
            state: State::Closed,
            id_source: id_source.into(),
            mode: Mode::Set,
            frames_len: 1,
            start_frame: 0,
            end_frame: 0,
            num_patterns: 5,

            set_pattern_enabled: false,
            set_x_enabled: false,
            set_y_enabled: false,
            set_scale_enabled: false,
            set_rotation_enabled: false,
            set_flip_enabled: false,
            set_opacity_enabled: false,
            set_blending_enabled: false,

            set_pattern: 0,
            set_x: 0,
            set_y: 0,
            set_scale: 100,
            set_rotation: 0,
            set_flip: 0,
            set_opacity: 255,
            set_blending: 1,

            add_pattern: 0,
            add_x: 0,
            add_y: 0,
            add_scale: 0,
            add_rotation: 0,
            add_flip: false,
            add_opacity: 0,
            add_blending: 0,

            mul_pattern: 1.,
            mul_x: 1.,
            mul_y: 1.,
            mul_scale: 1.,
            mul_rotation: 1.,
            mul_opacity: 1.,
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
            let response = ui.button("Batch Edit");
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
        num_patterns: u32,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        if !matches!(self.state, State::Open) {
            self.frames_len = frames_len;
            self.start_frame = current_frame;
            self.end_frame = current_frame;
            self.num_patterns = num_patterns;
            return false;
        }

        egui::Window::new("Batch Edit")
            .open(&mut win_open)
            .id(self.id_source.with("batch_edit_tool"))
            .show(ctx, |ui| {
                ui.with_padded_stripe(false, |ui| {
                    ui.columns(3, |columns| {
                        columns[0].selectable_value(&mut self.mode, Mode::Set, "Set value");
                        columns[1].selectable_value(&mut self.mode, Mode::Add, "Add");
                        columns[2].selectable_value(&mut self.mode, Mode::Mul, "Multiply");
                    });
                });

                ui.with_padded_stripe(true, |ui| {
                    ui.columns(2, |columns| {
                        self.start_frame += 1;
                        columns[0].add(luminol_components::Field::new(
                            "Starting Frame",
                            egui::DragValue::new(&mut self.start_frame).range(1..=self.frames_len),
                        ));
                        self.start_frame -= 1;

                        if self.start_frame > self.end_frame {
                            self.end_frame = self.start_frame;
                        }

                        self.end_frame += 1;
                        columns[1].add(luminol_components::Field::new(
                            "Ending Frame",
                            egui::DragValue::new(&mut self.end_frame).range(1..=self.frames_len),
                        ));
                        self.end_frame -= 1;

                        if self.end_frame < self.start_frame {
                            self.start_frame = self.end_frame;
                        }
                    });
                });

                match self.mode {
                    Mode::Set => {
                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                self.set_pattern += 1;
                                columns[0].add(luminol_components::FieldWithCheckbox::new(
                                    "Pattern",
                                    &mut self.set_pattern_enabled,
                                    egui::DragValue::new(&mut self.set_pattern)
                                        .range(1..=num_patterns as i16),
                                ));
                                self.set_pattern -= 1;

                                columns[1].add(luminol_components::FieldWithCheckbox::new(
                                    "X",
                                    &mut self.set_x_enabled,
                                    egui::DragValue::new(&mut self.set_x)
                                        .range(-(FRAME_WIDTH as i16 / 2)..=FRAME_WIDTH as i16 / 2),
                                ));

                                columns[2].add(luminol_components::FieldWithCheckbox::new(
                                    "Y",
                                    &mut self.set_y_enabled,
                                    egui::DragValue::new(&mut self.set_y).range(
                                        -(FRAME_HEIGHT as i16 / 2)..=FRAME_HEIGHT as i16 / 2,
                                    ),
                                ));

                                columns[3].add(luminol_components::FieldWithCheckbox::new(
                                    "Scale",
                                    &mut self.set_scale_enabled,
                                    egui::DragValue::new(&mut self.set_scale)
                                        .range(1..=i16::MAX)
                                        .suffix("%"),
                                ));
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(4, |columns| {
                                columns[0].add(luminol_components::FieldWithCheckbox::new(
                                    "Rotation",
                                    &mut self.set_rotation_enabled,
                                    egui::DragValue::new(&mut self.set_rotation)
                                        .range(0..=360)
                                        .suffix("°"),
                                ));

                                let mut flip = self.set_flip == 1;
                                columns[1].add(luminol_components::FieldWithCheckbox::new(
                                    "Flip",
                                    &mut self.set_flip_enabled,
                                    egui::Checkbox::without_text(&mut flip),
                                ));
                                self.set_flip = if flip { 1 } else { 0 };

                                columns[2].add(luminol_components::FieldWithCheckbox::new(
                                    "Opacity",
                                    &mut self.set_opacity_enabled,
                                    egui::DragValue::new(&mut self.set_opacity).range(0..=255),
                                ));

                                let mut blend_mode = match self.set_blending {
                                    1 => BlendMode::Add,
                                    2 => BlendMode::Subtract,
                                    _ => BlendMode::Normal,
                                };
                                columns[3].add(luminol_components::FieldWithCheckbox::new(
                                    "Blending",
                                    &mut self.set_blending_enabled,
                                    luminol_components::EnumComboBox::new(
                                        self.id_source.with("set_blending"),
                                        &mut blend_mode,
                                    ),
                                ));
                                self.set_blending = match blend_mode {
                                    BlendMode::Normal => 0,
                                    BlendMode::Add => 1,
                                    BlendMode::Subtract => 2,
                                };
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            if ui.button("Reset values").clicked() {
                                self.set_pattern_enabled = false;
                                self.set_x_enabled = false;
                                self.set_y_enabled = false;
                                self.set_scale_enabled = false;
                                self.set_rotation_enabled = false;
                                self.set_flip_enabled = false;
                                self.set_opacity_enabled = false;
                                self.set_blending_enabled = false;

                                self.set_pattern = 0;
                                self.set_x = 0;
                                self.set_y = 0;
                                self.set_scale = 100;
                                self.set_rotation = 0;
                                self.set_flip = 0;
                                self.set_opacity = 255;
                                self.set_blending = 1;
                            }
                        });
                    }

                    Mode::Add => {
                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                let limit = num_patterns.saturating_sub(1) as i16;
                                columns[0].add(luminol_components::Field::new(
                                    "Pattern",
                                    egui::DragValue::new(&mut self.add_pattern)
                                        .range(-limit..=limit),
                                ));

                                columns[1].add(luminol_components::Field::new(
                                    "X",
                                    egui::DragValue::new(&mut self.add_x)
                                        .range(-(FRAME_WIDTH as i16)..=FRAME_WIDTH as i16),
                                ));

                                columns[2].add(luminol_components::Field::new(
                                    "Y",
                                    egui::DragValue::new(&mut self.add_y)
                                        .range(-(FRAME_HEIGHT as i16)..=FRAME_HEIGHT as i16),
                                ));

                                columns[3].add(luminol_components::Field::new(
                                    "Scale",
                                    egui::DragValue::new(&mut self.add_scale).suffix("%"),
                                ));
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(4, |columns| {
                                columns[0].add(luminol_components::Field::new(
                                    "Rotation",
                                    egui::DragValue::new(&mut self.add_rotation)
                                        .range(-360..=360)
                                        .suffix("°"),
                                ));

                                columns[1].add(luminol_components::Field::new(
                                    "Flip",
                                    egui::Checkbox::without_text(&mut self.add_flip),
                                ));

                                columns[2].add(luminol_components::Field::new(
                                    "Opacity",
                                    egui::DragValue::new(&mut self.add_opacity).range(-255..=255),
                                ));

                                columns[3].add(luminol_components::Field::new(
                                    "Blending",
                                    egui::DragValue::new(&mut self.add_blending).range(-2..=2),
                                ));
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            if ui.button("Reset values").clicked() {
                                self.add_pattern = 0;
                                self.add_x = 0;
                                self.add_y = 0;
                                self.add_scale = 0;
                                self.add_rotation = 0;
                                self.add_flip = false;
                                self.add_opacity = 0;
                                self.add_blending = 0;
                            }
                        });
                    }

                    Mode::Mul => {
                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                self.mul_pattern *= 100.;
                                columns[0].add(luminol_components::Field::new(
                                    "Pattern",
                                    egui::DragValue::new(&mut self.mul_pattern)
                                        .range(
                                            (num_patterns as f64).recip() * 100.0
                                                ..=num_patterns as f64 * 100.0,
                                        )
                                        .suffix("%"),
                                ));
                                self.mul_pattern /= 100.;

                                self.mul_x *= 100.;
                                columns[1].add(luminol_components::Field::new(
                                    "X",
                                    egui::DragValue::new(&mut self.mul_x)
                                        .range(
                                            -(FRAME_WIDTH as f64 / 2.) * 100.0
                                                ..=FRAME_WIDTH as f64 / 2. * 100.0,
                                        )
                                        .suffix("%"),
                                ));
                                self.mul_x /= 100.;

                                self.mul_y *= 100.;
                                columns[2].add(luminol_components::Field::new(
                                    "Y",
                                    egui::DragValue::new(&mut self.mul_y)
                                        .range(
                                            -(FRAME_HEIGHT as f64 / 2.) * 100.0
                                                ..=FRAME_HEIGHT as f64 / 2. * 100.0,
                                        )
                                        .suffix("%"),
                                ));
                                self.mul_y /= 100.;

                                self.mul_scale *= 100.;
                                columns[3].add(luminol_components::Field::new(
                                    "Scale",
                                    egui::DragValue::new(&mut self.mul_scale)
                                        .range(0.0..=i16::MAX as f64 * 100.0)
                                        .suffix("%"),
                                ));
                                self.mul_scale /= 100.;
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                self.mul_rotation *= 100.;
                                columns[0].add(luminol_components::Field::new(
                                    "Rotation",
                                    egui::DragValue::new(&mut self.mul_rotation)
                                        .range(-360. * 100.0..=360.0 * 100.0)
                                        .suffix("%"),
                                ));
                                self.mul_rotation /= 100.;

                                self.mul_opacity *= 100.;
                                columns[1].add(luminol_components::Field::new(
                                    "Opacity",
                                    egui::DragValue::new(&mut self.mul_opacity)
                                        .range(0.0..=255. * 100.0)
                                        .suffix("%"),
                                ));
                                self.mul_opacity /= 100.;
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            if ui.button("Reset values").clicked() {
                                self.mul_pattern = 1.;
                                self.mul_x = 1.;
                                self.mul_y = 1.;
                                self.mul_scale = 1.;
                                self.mul_rotation = 1.;
                                self.mul_opacity = 1.;
                            }
                        });
                    }
                }

                ui.with_padded_stripe(true, |ui| {
                    ui.with_cross_justify(|ui| {
                        let spacing = ui.spacing().item_spacing.y;
                        ui.add_space(spacing);

                        ui.label(match self.mode {
                            Mode::Set if self.start_frame == self.end_frame => {
                                format!(
                                    "Set the above values for all cells in frame {}",
                                    self.start_frame + 1,
                                )
                            }
                            Mode::Set => {
                                format!(
                                    "Set the above values for all cells in frames {}–{}",
                                    self.start_frame + 1,
                                    self.end_frame + 1,
                                )
                            }
                            Mode::Add if self.start_frame == self.end_frame => {
                                format!(
                                    "Add the above values for all cells in frame {}",
                                    self.start_frame + 1,
                                )
                            }
                            Mode::Add => {
                                format!(
                                    "Add the above values for all cells in frames {}–{}",
                                    self.start_frame + 1,
                                    self.end_frame + 1,
                                )
                            }
                            Mode::Mul if self.start_frame == self.end_frame => {
                                format!(
                                    "Multiply by the above values for all cells in frame {}",
                                    self.start_frame + 1,
                                )
                            }
                            Mode::Mul => {
                                format!(
                                    "Multiply by the above values for all cells in frames {}–{}",
                                    self.start_frame + 1,
                                    self.end_frame + 1,
                                )
                            }
                        });

                        luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                        ui.add_space(spacing);
                    });
                });
            });

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }
        needs_save
    }
}
