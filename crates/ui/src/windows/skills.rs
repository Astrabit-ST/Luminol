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

#[derive(Default)]
pub struct Window {
    selected_skill_name: Option<String>,
    previous_skill: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("skill_editor")
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
        let mut skills = data.skills();
        let animations = data.animations();
        let common_events = data.common_events();
        let system = data.system();
        let states = data.states();

        let mut modified = false;

        self.selected_skill_name = None;

        let name = if let Some(name) = &self.selected_skill_name {
            format!("Editing skill {:?}", name)
        } else {
            "Skill Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Skills",
                    &mut skills.data,
                    |skill| format!("{:0>4}: {}", skill.id + 1, skill.name),
                    |ui, skills, id, update_state| {
                        let skill = &mut skills[id];
                        self.selected_skill_name = Some(skill.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut skill.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();

                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Description",
                                    egui::TextEdit::multiline(&mut skill.description)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Scope",
                                        luminol_components::EnumComboBox::new(
                                            (skill.id, "scope"),
                                            &mut skill.scope,
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Occasion",
                                        luminol_components::EnumComboBox::new(
                                            (skill.id, "occasion"),
                                            &mut skill.occasion,
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "User Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (skill.id, "animation1_id"),
                                            &mut skill.animation1_id,
                                            0..animations.data.len(),
                                            |id| {
                                                animations.data.get(id).map_or_else(
                                                    || "".into(),
                                                    |a| format!("{:0>4}: {}", id + 1, a.name),
                                                )
                                            },
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Target Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (skill.id, "animation2_id"),
                                            &mut skill.animation2_id,
                                            0..animations.data.len(),
                                            |id| {
                                                animations.data.get(id).map_or_else(
                                                    || "".into(),
                                                    |a| format!("{:0>4}: {}", id + 1, a.name),
                                                )
                                            },
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Menu Use SE",
                                        egui::Label::new("TODO"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Common Event",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (skill.id, "common_event_id"),
                                            &mut skill.common_event_id,
                                            0..common_events.data.len(),
                                            |id| {
                                                common_events.data.get(id).map_or_else(
                                                    || "".into(),
                                                    |e| format!("{:0>4}: {}", id + 1, e.name),
                                                )
                                            },
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "SP Cost",
                                        egui::DragValue::new(&mut skill.sp_cost)
                                            .range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Power",
                                        egui::DragValue::new(&mut skill.power),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "ATK-F",
                                        egui::Slider::new(&mut skill.atk_f, 0..=200).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EVA-F",
                                        egui::Slider::new(&mut skill.eva_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "STR-F",
                                        egui::Slider::new(&mut skill.str_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "DEX-F",
                                        egui::Slider::new(&mut skill.dex_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "AGI-F",
                                        egui::Slider::new(&mut skill.agi_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "INT-F",
                                        egui::Slider::new(&mut skill.int_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Hit Rate",
                                        egui::Slider::new(&mut skill.hit, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Variance",
                                        egui::Slider::new(&mut skill.variance, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "PDEF-F",
                                        egui::Slider::new(&mut skill.pdef_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "MDEF-F",
                                        egui::Slider::new(&mut skill.mdef_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
                                    (skill.id, "element_set"),
                                    &mut skill.element_set,
                                    1..system.elements.len(),
                                    |id| {
                                        system.elements.get(id).map_or_else(
                                            || "".into(),
                                            |e| format!("{id:0>4}: {}", e),
                                        )
                                    },
                                );
                                if self.previous_skill != Some(skill.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new("Elements", selection))
                                    .changed();

                                let mut selection =
                                    luminol_components::IdVecPlusMinusSelection::new(
                                        update_state,
                                        (skill.id, "state_set"),
                                        &mut skill.plus_state_set,
                                        &mut skill.minus_state_set,
                                        0..states.data.len(),
                                        |id| {
                                            states.data.get(id).map_or_else(
                                                || "".into(),
                                                |s| format!("{:0>4}: {}", id + 1, s.name),
                                            )
                                        },
                                    );
                                if self.previous_skill != Some(skill.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("State Change", selection))
                                    .changed();
                            });
                        });

                        self.previous_skill = Some(skill.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            skills.modified = true;
        }

        drop(skills);
        drop(animations);
        drop(common_events);
        drop(system);
        drop(states);

        *update_state.data = data; // restore data
    }
}
