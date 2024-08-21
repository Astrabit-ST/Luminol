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

use crate::components::{EnumComboBox, Field};
use egui::Widget;
use luminol_core::Modal;

use super::{util::update_flash_maps, TimingEditState};
use luminol_data::rpg::animation::{Condition, Scope, Timing};

pub fn show_timing_header(ui: &mut egui::Ui, timing: &Timing) {
    let mut vec = Vec::with_capacity(3);

    match timing.condition {
        luminol_data::rpg::animation::Condition::None => {}
        luminol_data::rpg::animation::Condition::Hit => vec.push("on hit".into()),
        luminol_data::rpg::animation::Condition::Miss => vec.push("on miss".into()),
    }

    if let Some(path) = &timing.se.name {
        vec.push(format!("play {:?}", path.file_name().unwrap_or_default()));
    };

    match timing.flash_scope {
        Scope::None => {}
        Scope::Target => {
            vec.push(format!(
                "flash target #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                timing.flash_color.red.clamp(0., 255.).round() as u8,
                timing.flash_color.green.clamp(0., 255.).round() as u8,
                timing.flash_color.blue.clamp(0., 255.).round() as u8,
                timing.flash_color.alpha.clamp(0., 255.).round() as u8,
                timing.flash_duration,
            ));
        }
        Scope::Screen => {
            vec.push(format!(
                "flash screen #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                timing.flash_color.red.clamp(0., 255.).round() as u8,
                timing.flash_color.green.clamp(0., 255.).round() as u8,
                timing.flash_color.blue.clamp(0., 255.).round() as u8,
                timing.flash_color.alpha.clamp(0., 255.).round() as u8,
                timing.flash_duration,
            ));
        }
        Scope::HideTarget => {
            vec.push(format!("hide target for {} frames", timing.flash_duration));
        }
    }

    ui.label(format!(
        "Frame {:0>3}: {}",
        timing.frame + 1,
        vec.join(", ")
    ));
}

pub fn show_timing_body(
    ui: &mut egui::Ui,
    update_state: &mut luminol_core::UpdateState<'_>,
    animation: &luminol_data::rpg::Animation,
    flash_maps: &mut super::util::FlashMaps,
    state: &mut TimingEditState,
    timing: (usize, &[Timing], &mut Timing),
) -> egui::Response {
    let (timing_index, previous_timings, timing) = timing;
    let mut modified = false;

    let rank = |condition, frame, scope| {
        previous_timings
            .iter()
            .rev()
            .take_while(|t| t.frame == frame)
            .filter(|t| t.flash_scope == scope && super::util::filter_timing(t, condition))
            .count()
    };

    let mut response = egui::Frame::none()
        .show(ui, |ui| {
            ui.columns(2, |columns| {
                columns[0].columns(2, |columns| {
                    let old_condition = timing.condition;
                    let changed = columns[1]
                        .add(Field::new(
                            "Condition",
                            EnumComboBox::new(
                                (animation.id, timing_index, "condition"),
                                &mut timing.condition,
                            ),
                        ))
                        .changed();
                    if changed {
                        if old_condition != Condition::Miss && timing.condition == Condition::Miss {
                            match timing.flash_scope {
                                Scope::Target => {
                                    flash_maps.target_mut(Condition::Hit).remove(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::Target),
                                    );
                                }
                                Scope::Screen => {
                                    flash_maps.screen_mut(Condition::Hit).remove(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::Screen),
                                    );
                                }
                                Scope::HideTarget => {
                                    flash_maps.hide_mut(Condition::Hit).remove(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::HideTarget),
                                    );
                                }
                                Scope::None => {}
                            }
                        } else if old_condition != Condition::Hit
                            && timing.condition == Condition::Hit
                        {
                            match timing.flash_scope {
                                Scope::Target => {
                                    flash_maps.target_mut(Condition::Miss).remove(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::Target),
                                    );
                                }
                                Scope::Screen => {
                                    flash_maps.screen_mut(Condition::Miss).remove(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::Screen),
                                    );
                                }
                                Scope::HideTarget => {
                                    flash_maps.hide_mut(Condition::Miss).remove(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::HideTarget),
                                    );
                                }
                                Scope::None => {}
                            }
                        }
                        if old_condition == Condition::Miss && timing.condition != Condition::Miss {
                            match timing.flash_scope {
                                Scope::Target => {
                                    flash_maps.target_mut(Condition::Hit).insert(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::Target),
                                        timing.into(),
                                    );
                                }
                                Scope::Screen => {
                                    flash_maps.screen_mut(Condition::Hit).insert(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::Screen),
                                        timing.into(),
                                    );
                                }
                                Scope::HideTarget => {
                                    flash_maps.hide_mut(Condition::Hit).insert(
                                        timing.frame,
                                        rank(Condition::Hit, timing.frame, Scope::HideTarget),
                                        timing.into(),
                                    );
                                }
                                Scope::None => {}
                            }
                        } else if old_condition == Condition::Hit
                            && timing.condition != Condition::Hit
                        {
                            match timing.flash_scope {
                                Scope::Target => {
                                    flash_maps.target_mut(Condition::Miss).insert(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::Target),
                                        timing.into(),
                                    );
                                }
                                Scope::Screen => {
                                    flash_maps.screen_mut(Condition::Miss).insert(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::Screen),
                                        timing.into(),
                                    );
                                }
                                Scope::HideTarget => {
                                    flash_maps.hide_mut(Condition::Miss).insert(
                                        timing.frame,
                                        rank(Condition::Miss, timing.frame, Scope::HideTarget),
                                        timing.into(),
                                    );
                                }
                                Scope::None => {}
                            }
                        }
                        modified = true;
                    }

                    let old_frame = timing.frame;
                    let changed = columns[0]
                        .add(Field::new("Frame", |ui: &mut egui::Ui| {
                            let mut frame = state.previous_frame.unwrap_or(timing.frame + 1);
                            let mut response = egui::DragValue::new(&mut frame)
                                .range(1..=animation.frames.len())
                                .update_while_editing(false)
                                .ui(ui);
                            response.changed = false;
                            if response.dragged() {
                                state.previous_frame = Some(frame);
                            } else if state.previous_frame.is_some() {
                                timing.frame = frame - 1;
                                state.previous_frame = None;
                                response.changed = true;
                            }
                            response
                        }))
                        .changed();
                    if changed {
                        update_flash_maps(timing.condition, |condition| match timing.flash_scope {
                            Scope::Target => {
                                flash_maps.target_mut(condition).set_frame(
                                    old_frame,
                                    rank(condition, old_frame, Scope::Target),
                                    timing.frame,
                                );
                            }
                            Scope::Screen => {
                                flash_maps.screen_mut(condition).set_frame(
                                    old_frame,
                                    rank(condition, old_frame, Scope::Screen),
                                    timing.frame,
                                );
                            }
                            Scope::HideTarget => {
                                flash_maps.hide_mut(condition).set_frame(
                                    old_frame,
                                    rank(condition, old_frame, Scope::HideTarget),
                                    timing.frame,
                                );
                            }
                            Scope::None => {}
                        });
                        modified = true;
                    }
                });

                modified |= columns[1]
                    .add(Field::new(
                        "SE",
                        state.se_picker.button(&mut timing.se, update_state),
                    ))
                    .changed();
            });

            let old_scope = timing.flash_scope;
            let (scope_changed, duration_changed) = if timing.flash_scope == Scope::None {
                (
                    ui.add(Field::new(
                        "Flash",
                        EnumComboBox::new(
                            (animation.id, timing_index, "flash_scope"),
                            &mut timing.flash_scope,
                        ),
                    ))
                    .changed(),
                    false,
                )
            } else {
                ui.columns(2, |columns| {
                    (
                        columns[0]
                            .add(Field::new(
                                "Flash",
                                EnumComboBox::new(
                                    (animation.id, timing_index, "flash_scope"),
                                    &mut timing.flash_scope,
                                ),
                            ))
                            .changed(),
                        columns[1]
                            .add(Field::new(
                                "Flash Duration",
                                egui::DragValue::new(&mut timing.flash_duration)
                                    .range(1..=animation.frames.len()),
                            ))
                            .changed(),
                    )
                })
            };

            if scope_changed {
                update_flash_maps(timing.condition, |condition| {
                    match old_scope {
                        Scope::Target => {
                            flash_maps
                                .target_mut(condition)
                                .remove(timing.frame, rank(condition, timing.frame, Scope::Target));
                        }
                        Scope::Screen => {
                            flash_maps
                                .screen_mut(condition)
                                .remove(timing.frame, rank(condition, timing.frame, Scope::Screen));
                        }
                        Scope::HideTarget => {
                            flash_maps.hide_mut(condition).remove(
                                timing.frame,
                                rank(condition, timing.frame, Scope::HideTarget),
                            );
                        }
                        Scope::None => {}
                    }
                    match timing.flash_scope {
                        Scope::Target => {
                            flash_maps.target_mut(condition).insert(
                                timing.frame,
                                rank(condition, timing.frame, Scope::Target),
                                timing.into(),
                            );
                        }
                        Scope::Screen => {
                            flash_maps.screen_mut(condition).insert(
                                timing.frame,
                                rank(condition, timing.frame, Scope::Screen),
                                timing.into(),
                            );
                        }
                        Scope::HideTarget => {
                            flash_maps.hide_mut(condition).insert(
                                timing.frame,
                                rank(condition, timing.frame, Scope::HideTarget),
                                timing.into(),
                            );
                        }
                        Scope::None => {}
                    }
                });
                modified = true;
            }

            if duration_changed {
                update_flash_maps(timing.condition, |condition| match timing.flash_scope {
                    Scope::Target => {
                        flash_maps
                            .target_mut(condition)
                            .get_mut(timing.frame, rank(condition, timing.frame, Scope::Target))
                            .unwrap()
                            .duration = timing.flash_duration;
                    }
                    Scope::Screen => {
                        flash_maps
                            .screen_mut(condition)
                            .get_mut(timing.frame, rank(condition, timing.frame, Scope::Screen))
                            .unwrap()
                            .duration = timing.flash_duration;
                    }
                    Scope::HideTarget => {
                        flash_maps
                            .hide_mut(condition)
                            .get_mut(
                                timing.frame,
                                rank(condition, timing.frame, Scope::HideTarget),
                            )
                            .unwrap()
                            .duration = timing.flash_duration;
                    }
                    Scope::None => unreachable!(),
                });
                modified = true;
            }

            if matches!(timing.flash_scope, Scope::Target | Scope::Screen) {
                let changed = ui
                    .add(Field::new("Flash Color", |ui: &mut egui::Ui| {
                        let mut color = [
                            timing.flash_color.red.clamp(0., 255.).round() as u8,
                            timing.flash_color.green.clamp(0., 255.).round() as u8,
                            timing.flash_color.blue.clamp(0., 255.).round() as u8,
                            timing.flash_color.alpha.clamp(0., 255.).round() as u8,
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
                    }))
                    .changed();
                if changed {
                    update_flash_maps(timing.condition, |condition| match timing.flash_scope {
                        Scope::Target => {
                            flash_maps
                                .target_mut(condition)
                                .get_mut(timing.frame, rank(condition, timing.frame, Scope::Target))
                                .unwrap()
                                .color = timing.flash_color;
                        }
                        Scope::Screen => {
                            flash_maps
                                .screen_mut(condition)
                                .get_mut(timing.frame, rank(condition, timing.frame, Scope::Screen))
                                .unwrap()
                                .color = timing.flash_color;
                        }
                        Scope::None | Scope::HideTarget => unreachable!(),
                    });
                    modified = true;
                }
            }
        })
        .response;

    if modified {
        response.mark_changed();
    }
    response
}
