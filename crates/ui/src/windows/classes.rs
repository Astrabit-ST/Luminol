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

use luminol_components::UiExt;

use egui::collapsing_header::CollapsingState;

#[derive(Default)]
pub struct Window {
    selected_class_name: Option<String>,
    previous_class: Option<usize>,

    depersisted_skills: usize,
    expanded_skill: luminol_data::OptionVec<Option<usize>>,
    change_expanded_skill_immediately: bool,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        if let Some(name) = &self.selected_class_name {
            format!("Editing class {:?}", name)
        } else {
            "Class Editor".into()
        }
    }

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
        let mut classes = update_state.data.classes();
        let system = update_state.data.system();
        let states = update_state.data.states();
        let skills = update_state.data.skills();
        let weapons = update_state.data.weapons();
        let armors = update_state.data.armors();

        let mut modified = false;

        self.selected_class_name = None;

        let response = egui::Window::new(self.name())
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    "Classes",
                    update_state
                        .project_config
                        .as_ref()
                        .expect("project not loaded"),
                    &mut classes.data,
                    |class| format!("{:0>3}: {}", class.id + 1, class.name),
                    |ui, class| {
                        self.selected_class_name = Some(class.name.clone());

                        modified |= ui
                            .add(luminol_components::Field::new(
                                "Name",
                                egui::TextEdit::singleline(&mut class.name)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();

                        ui.with_stripe(true, |ui| {
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

                        ui.with_stripe(false, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Skills",
                                |ui: &mut egui::Ui| {
                                    ui.with_cross_justify(|ui| {
                                        let mut deleted_skill = None;
                                        let mut new_skill = false;

                                        if self.previous_class != Some(class.id) {
                                            self.change_expanded_skill_immediately = true;
                                        }

                                        let inner_response = ui.group(|ui| {
                                            if self.expanded_skill.get(class.id).is_none() {
                                                self.expanded_skill.insert(class.id, None);
                                            }
                                            let expanded_skill =
                                                self.expanded_skill.get_mut(class.id).unwrap();

                                            for (i, learning) in
                                                class.learnings.iter_mut().enumerate()
                                            {
                                                let ui_id = ui.make_persistent_id(i);

                                                // Forget whether the collapsing header was open from the last time
                                                // the editor was open
                                                let depersisted = i < self.depersisted_skills;
                                                if !depersisted {
                                                    self.depersisted_skills += 1;
                                                    if let Some(h) =
                                                        CollapsingState::load(ui.ctx(), ui_id)
                                                    {
                                                        h.remove(ui.ctx());
                                                    }
                                                    ui.ctx()
                                                        .animate_bool_with_time(ui_id, false, 0.);
                                                }

                                                let mut header =
                                                    CollapsingState::load_with_default_open(
                                                        ui.ctx(),
                                                        ui_id,
                                                        false,
                                                    );
                                                let expanded = (self
                                                    .change_expanded_skill_immediately
                                                    || depersisted)
                                                    && *expanded_skill == Some(i);
                                                header.set_open(expanded);
                                                if self.change_expanded_skill_immediately {
                                                    ui.ctx().animate_bool_with_time(
                                                        ui_id, expanded, 0.,
                                                    );
                                                }

                                                let layout = *ui.layout();
                                                let (expand_button_response, _, _) = header
                                                    .show_header(ui, |ui| {
                                                        ui.with_layout(layout, |ui| {
                                                            ui.add(
                                                                egui::Label::new(format!(
                                                                    "Lvl {}: {}",
                                                                    learning.level,
                                                                    skills
                                                                        .data
                                                                        .get(learning.skill_id)
                                                                        .map_or("", |s| &s.name)
                                                                ))
                                                                .truncate(true),
                                                            );
                                                        });
                                                    })
                                                    .body(|ui| {
                                                        ui.columns(2, |columns| {
                                                            modified |= columns[0]
                                                                .add(
                                                                    luminol_components::Field::new(
                                                                        "Level",
                                                                        egui::Slider::new(
                                                                            &mut learning.level,
                                                                            1..=99,
                                                                        ),
                                                                    ),
                                                                )
                                                                .changed();

                                                            modified |= columns[1]
                                                                .add(luminol_components::Field::new(
                                                                    "Skill",
                                                                    luminol_components::OptionalIdComboBox::new(
                                                                        (class.id, i, "skill_id"),
                                                                        &mut learning.skill_id,
                                                                        0..skills.data.len(),
                                                                        |id| {
                                                                            skills.data.get(id).map_or_else(
                                                                                || "".into(),
                                                                                |s| {
                                                                                    format!(
                                                                                        "{:0>3}: {}",
                                                                                        id + 1,
                                                                                        s.name
                                                                                    )
                                                                                },
                                                                            )
                                                                        },
                                                                    ),
                                                                ))
                                                                .changed();
                                                        });

                                                        if ui.button("Delete skill").clicked() {
                                                            deleted_skill = Some(i);
                                                        }
                                                    });

                                                if expand_button_response.clicked() {
                                                    *expanded_skill =
                                                        (*expanded_skill != Some(i)).then_some(i);
                                                }
                                            }

                                            ui.add_space(2. * ui.spacing().item_spacing.y);

                                            if ui.button("New skill").clicked() {
                                                *expanded_skill = Some(class.learnings.len());
                                                class.learnings.push(Default::default());
                                                new_skill = true;
                                            }
                                        });

                                        self.change_expanded_skill_immediately = false;

                                        if let Some(i) = deleted_skill {
                                            if let Some(expanded_skill) =
                                                self.expanded_skill.get_mut(class.id)
                                            {
                                                if *expanded_skill == Some(i) {
                                                    self.change_expanded_skill_immediately = true;
                                                    *expanded_skill = None;
                                                } else if expanded_skill.is_some()
                                                    && *expanded_skill > Some(i)
                                                {
                                                    self.change_expanded_skill_immediately = true;
                                                    *expanded_skill =
                                                        Some(expanded_skill.unwrap() - 1);
                                                }
                                            }

                                            class.learnings.remove(i);
                                        }

                                        self.depersisted_skills = class.learnings.len();
                                        if new_skill {
                                            self.depersisted_skills -= 1;
                                        }

                                        inner_response.response
                                    })
                                    .response
                                },
                            ));
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
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

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::RankSelection::new(
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

                                let mut selection =
                                    luminol_components::RankSelection::new(
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
    }
}
