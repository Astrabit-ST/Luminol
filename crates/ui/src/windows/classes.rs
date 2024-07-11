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
    selected_class_name: Option<String>,
    previous_class: Option<usize>,

    collapsing_view: luminol_components::CollapsingView,
    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }

    fn show_learning_header(
        ui: &mut egui::Ui,
        skills: &luminol_data::rpg::Skills,
        learning: &luminol_data::rpg::class::Learning,
    ) {
        ui.label(format!(
            "Lvl {}: {}",
            learning.level,
            skills.data.get(learning.skill_id).map_or("", |s| &s.name)
        ));
    }

    fn show_learning_body(
        ui: &mut egui::Ui,
        update_state: &luminol_core::UpdateState<'_>,
        skills: &luminol_data::rpg::Skills,
        class_id: usize,
        learning: (usize, &mut luminol_data::rpg::class::Learning),
    ) -> egui::Response {
        let (learning_index, learning) = learning;
        let mut modified = false;

        let mut response = egui::Frame::none()
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    modified |= columns[0]
                        .add(luminol_components::Field::new(
                            "Level",
                            egui::Slider::new(&mut learning.level, 1..=99),
                        ))
                        .changed();

                    modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "Skill",
                            luminol_components::OptionalIdComboBox::new(
                                update_state,
                                (class_id, learning_index, "skill_id"),
                                &mut learning.skill_id,
                                0..skills.data.len(),
                                |id| {
                                    skills.data.get(id).map_or_else(
                                        || "".into(),
                                        |s| format!("{:0>3}: {}", id + 1, s.name),
                                    )
                                },
                            ),
                        ))
                        .changed();
                });
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
        egui::Id::new("class_editor")
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
        let mut classes = data.classes();
        let system = data.system();
        let states = data.states();
        let skills = data.skills();
        let weapons = data.weapons();
        let armors = data.armors();

        let mut modified = false;

        self.selected_class_name = None;

        let name = if let Some(name) = &self.selected_class_name {
            format!("Editing class {:?}", name)
        } else {
            "Class Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Classes",
                    &mut classes.data,
                    |class| format!("{:0>3}: {}", class.id + 1, class.name),
                    |ui, classes, id, update_state| {
                        let class = &mut classes[id];
                        self.selected_class_name = Some(class.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut class.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Position",
                                    luminol_components::EnumComboBox::new(
                                        (class.id, "position"),
                                        &mut class.position,
                                    ),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Skills",
                                    |ui: &mut egui::Ui| {
                                        if self.previous_class != Some(class.id) {
                                            self.collapsing_view.clear_animations();
                                        }
                                        self.collapsing_view.show(
                                            ui,
                                            class.id,
                                            &mut class.learnings,
                                            |ui, _i, learning| {
                                                Self::show_learning_header(ui, &skills, learning)
                                            },
                                            |ui, i, learning| {
                                                Self::show_learning_body(
                                                    ui,
                                                    update_state,
                                                    &skills,
                                                    class.id,
                                                    (i, learning),
                                                )
                                            },
                                        )
                                    },
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
                                    (class.id, "weapon_set"),
                                    &mut class.weapon_set,
                                    0..weapons.data.len(),
                                    |id| {
                                        weapons.data.get(id).map_or_else(
                                            || "".into(),
                                            |w| format!("{:0>3}: {}", id + 1, w.name),
                                        )
                                    },
                                );
                                if self.previous_class != Some(class.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Equippable Weapons",
                                        selection,
                                    ))
                                    .changed();

                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
                                    (class.id, "armor_set"),
                                    &mut class.armor_set,
                                    0..armors.data.len(),
                                    |id| {
                                        armors.data.get(id).map_or_else(
                                            || "".into(),
                                            |a| format!("{:0>3}: {}", id + 1, a.name),
                                        )
                                    },
                                );
                                if self.previous_class != Some(class.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Equippable Armor",
                                        selection,
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                class
                                    .element_ranks
                                    .resize_with_value(system.elements.len(), 3);
                                let mut selection = luminol_components::RankSelection::new(
                                    update_state,
                                    (class.id, "element_ranks"),
                                    &mut class.element_ranks,
                                    |id| {
                                        system.elements.get(id + 1).map_or_else(
                                            || "".into(),
                                            |e| format!("{:0>3}: {}", id + 1, e),
                                        )
                                    },
                                );
                                if self.previous_class != Some(class.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new("Elements", selection))
                                    .changed();

                                class
                                    .state_ranks
                                    .resize_with_value(states.data.len() + 1, 3);
                                let mut selection = luminol_components::RankSelection::new(
                                    update_state,
                                    (class.id, "state_ranks"),
                                    &mut class.state_ranks,
                                    |id| {
                                        states.data.get(id).map_or_else(
                                            || "".into(),
                                            |s| format!("{:0>3}: {}", id + 1, s.name),
                                        )
                                    },
                                );
                                if self.previous_class != Some(class.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("States", selection))
                                    .changed();
                            });
                        });

                        self.previous_class = Some(class.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            classes.modified = true;
        }

        drop(classes);
        drop(system);
        drop(states);
        drop(skills);
        drop(weapons);
        drop(armors);

        *update_state.data = data; // restore data
    }
}
