// Copyright (C) 2023 Lily Lyons
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

use egui::Widget;
use luminol_components::UiExt;
use luminol_core::Modal;

use luminol_modals::sound_picker::Modal as SoundPicker;

/// Database - Animations management window.
pub struct Window {
    selected_animation_name: Option<String>,
    previous_animation: Option<usize>,
    previous_timing_frame: Option<i32>,

    collapsing_view: luminol_components::CollapsingView,
    timing_se_picker: SoundPicker,
    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Self {
            selected_animation_name: None,
            previous_animation: None,
            previous_timing_frame: None,
            collapsing_view: luminol_components::CollapsingView::new(),
            timing_se_picker: SoundPicker::new(
                luminol_audio::Source::SE,
                "animations_timing_se_picker",
            ),
            view: luminol_components::DatabaseView::new(),
        }
    }

    fn show_timing_header(ui: &mut egui::Ui, timing: &luminol_data::rpg::animation::Timing) {
        let mut vec = Vec::with_capacity(3);

        if let Some(path) = &timing.se.name {
            vec.push(format!("play {:?}", path.file_name().unwrap_or_default()));
        };

        match timing.condition {
            luminol_data::rpg::animation::Condition::None => {}
            luminol_data::rpg::animation::Condition::Hit => vec.push("on hit".into()),
            luminol_data::rpg::animation::Condition::Miss => vec.push("on miss".into()),
        }

        match timing.flash_scope {
            luminol_data::rpg::animation::Scope::None => {}
            luminol_data::rpg::animation::Scope::Target => {
                vec.push(format!(
                    "flash target #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                    timing.flash_duration,
                ));
            }
            luminol_data::rpg::animation::Scope::Screen => {
                vec.push(format!(
                    "flash screen #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                    timing.flash_duration,
                ));
            }
            luminol_data::rpg::animation::Scope::HideTarget => {
                vec.push(format!("hide target for {} frames", timing.flash_duration));
            }
        }

        ui.label(format!(
            "Frame {:0>3}: {}",
            timing.frame + 1,
            vec.join(", ")
        ));
    }

    fn show_timing_body(
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        animation_id: usize,
        animation_frame_max: i32,
        timing_se_picker: &mut SoundPicker,
        previous_timing_frame: &mut Option<i32>,
        timing: (usize, &mut luminol_data::rpg::animation::Timing),
    ) -> egui::Response {
        let (timing_index, timing) = timing;
        let mut modified = false;

        let mut response = egui::Frame::none()
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].columns(2, |columns| {
                        modified |= columns[1]
                            .add(luminol_components::Field::new(
                                "Condition",
                                luminol_components::EnumComboBox::new(
                                    (animation_id, timing_index, "condition"),
                                    &mut timing.condition,
                                ),
                            ))
                            .changed();

                        modified |= columns[0]
                            .add(luminol_components::Field::new(
                                "Frame",
                                |ui: &mut egui::Ui| {
                                    let mut frame =
                                        previous_timing_frame.unwrap_or(timing.frame + 1);
                                    let mut response = egui::DragValue::new(&mut frame)
                                        .clamp_range(1..=animation_frame_max)
                                        .update_while_editing(false)
                                        .ui(ui);
                                    response.changed = false;
                                    if response.dragged() {
                                        *previous_timing_frame = Some(frame);
                                    } else {
                                        timing.frame = frame - 1;
                                        *previous_timing_frame = None;
                                        response.changed = true;
                                    }
                                    response
                                },
                            ))
                            .changed();
                    });

                    modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "SE",
                            timing_se_picker.button(&mut timing.se, update_state),
                        ))
                        .changed();
                });

                if timing.flash_scope == luminol_data::rpg::animation::Scope::None {
                    modified |= ui
                        .add(luminol_components::Field::new(
                            "Flash",
                            luminol_components::EnumComboBox::new(
                                (animation_id, timing_index, "flash_scope"),
                                &mut timing.flash_scope,
                            ),
                        ))
                        .changed();
                } else {
                    ui.columns(2, |columns| {
                        modified |= columns[0]
                            .add(luminol_components::Field::new(
                                "Flash",
                                luminol_components::EnumComboBox::new(
                                    (animation_id, timing_index, "flash_scope"),
                                    &mut timing.flash_scope,
                                ),
                            ))
                            .changed();

                        modified |= columns[1]
                            .add(luminol_components::Field::new(
                                "Flash Duration",
                                egui::DragValue::new(&mut timing.flash_duration)
                                    .clamp_range(1..=animation_frame_max),
                            ))
                            .changed();
                    });
                }

                if matches!(
                    timing.flash_scope,
                    luminol_data::rpg::animation::Scope::Target
                        | luminol_data::rpg::animation::Scope::Screen
                ) {
                    modified |= ui
                        .add(luminol_components::Field::new(
                            "Flash Color",
                            |ui: &mut egui::Ui| {
                                let mut color = [
                                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                                ];
                                ui.spacing_mut().interact_size.x = ui.available_width(); // make the color picker button as wide as possible
                                let response = ui.color_edit_button_srgba_unmultiplied(&mut color);
                                if response.changed() {
                                    timing.flash_color.red = color[0] as f64;
                                    timing.flash_color.green = color[1] as f64;
                                    timing.flash_color.blue = color[2] as f64;
                                    timing.flash_color.alpha = color[3] as f64;
                                }
                                response
                            },
                        ))
                        .changed();
                }
            })
            .response;

        if modified {
            response.mark_changed();
        }
        response
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("animation_editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let data = std::mem::take(update_state.data); // take data to avoid borrow checker issues
        let mut animations = data.animations();

        let mut modified = false;

        self.selected_animation_name = None;

        let name = if let Some(name) = &self.selected_animation_name {
            format!("Editing animation {:?}", name)
        } else {
            "Animation Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Animations",
                    &mut animations.data,
                    |animation| format!("{:0>4}: {}", animation.id + 1, animation.name),
                    |ui, animations, id, update_state| {
                        let animation = &mut animations[id];
                        self.selected_animation_name = Some(animation.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut animation.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "SE and Flash",
                                    |ui: &mut egui::Ui| {
                                        if *update_state.modified_during_prev_frame {
                                            self.collapsing_view.request_sort();
                                        }
                                        if self.previous_animation != Some(animation.id) {
                                            self.collapsing_view.clear_animations();
                                            self.timing_se_picker.close_window();
                                        } else if self.collapsing_view.is_animating() {
                                            self.timing_se_picker.close_window();
                                        }
                                        self.collapsing_view.show_with_sort(
                                            ui,
                                            animation.id,
                                            &mut animation.timings,
                                            |ui, _i, timing| Self::show_timing_header(ui, timing),
                                            |ui, i, timing| {
                                                Self::show_timing_body(
                                                    ui,
                                                    update_state,
                                                    animation.id,
                                                    animation.frame_max,
                                                    &mut self.timing_se_picker,
                                                    &mut self.previous_timing_frame,
                                                    (i, timing),
                                                )
                                            },
                                            |a, b| a.frame.cmp(&b.frame),
                                        )
                                    },
                                ))
                                .changed();
                        });

                        self.previous_animation = Some(animation.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            animations.modified = true;
        }

        drop(animations);

        *update_state.data = data; // restore data
    }
}
