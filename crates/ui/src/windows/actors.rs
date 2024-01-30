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

use luminol_data::rpg::armor::Kind;

#[derive(Default)]
pub struct Window {
    selected_actor_name: Option<String>,
    previous_actor: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        if let Some(name) = &self.selected_actor_name {
            format!("Editing actor {:?}", name)
        } else {
            "Actor Editor".into()
        }
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("actor_editor")
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
        let mut actors = update_state.data.actors();
        let mut classes = update_state.data.classes();
        let weapons = update_state.data.weapons();
        let armors = update_state.data.armors();

        let mut modified = false;

        self.selected_actor_name = None;

        let response = egui::Window::new(self.name())
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    "Actors",
                    update_state
                        .project_config
                        .as_ref()
                        .expect("project not loaded"),
                    &mut actors.data,
                    |actor| format!("{:0>3}: {}", actor.id, actor.name),
                    |ui, actor| {
                        self.selected_actor_name = Some(actor.name.clone());

                        modified |= ui
                            .add(luminol_components::Field::new(
                                "Name",
                                egui::TextEdit::singleline(&mut actor.name)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();

                        ui.with_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Class",
                                    luminol_components::OptionalIdComboBox::new(
                                        (actor.id, "class"),
                                        &mut actor.class_id,
                                        0..classes.data.len(),
                                        |id| {
                                            classes.data.get(id).map_or_else(
                                                || "".into(),
                                                |c| format!("{id:0>3}: {}", c.name),
                                            )
                                        },
                                    ),
                                ))
                                .changed();
                        });

                        if let Some(class) = classes.data.get_mut(actor.class_id) {
                            if !class.weapon_set.is_sorted() {
                                class.weapon_set.sort_unstable();
                            }
                            if !class.armor_set.is_sorted() {
                                class.armor_set.sort_unstable();
                            }
                        }
                        let class = classes.data.get(actor.class_id);

                        ui.with_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Starting Weapon",
                                    |ui: &mut egui::Ui| {
                                        egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(2, |columns| {
                                                    columns[0].add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            (actor.id, "weapon_id"),
                                                            &mut actor.weapon_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| {
                                                                        c.weapon_set.iter().copied()
                                                                    },
                                                                )
                                                                .filter(|id| {
                                                                    (0..weapons.data.len())
                                                                        .contains(id)
                                                                }),
                                                            |id| {
                                                                weapons.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |w| {
                                                                        format!(
                                                                            "{id:0>3}: {}",
                                                                            w.name
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    columns[1]
                                                        .checkbox(&mut actor.weapon_fix, "Fixed");
                                                });
                                            })
                                            .response
                                    },
                                ))
                                .changed();
                        });

                        ui.with_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Starting Shield",
                                    |ui: &mut egui::Ui| {
                                        egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(2, |columns| {
                                                    columns[0].add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            (actor.id, "armor1_id"),
                                                            &mut actor.armor1_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Shield
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{id:0>3}: {}",
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    columns[1]
                                                        .checkbox(&mut actor.armor1_fix, "Fixed");
                                                });
                                            })
                                            .response
                                    },
                                ))
                                .changed();
                        });

                        ui.with_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Starting Helmet",
                                    |ui: &mut egui::Ui| {
                                        egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(2, |columns| {
                                                    columns[0].add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            (actor.id, "armor2_id"),
                                                            &mut actor.armor2_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Helmet
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{id:0>3}: {}",
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    columns[1]
                                                        .checkbox(&mut actor.armor2_fix, "Fixed");
                                                });
                                            })
                                            .response
                                    },
                                ))
                                .changed();
                        });

                        ui.with_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Starting Body Armor",
                                    |ui: &mut egui::Ui| {
                                        egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(2, |columns| {
                                                    columns[0].add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            (actor.id, "armor3_id"),
                                                            &mut actor.armor3_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::BodyArmor
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{id:0>3}: {}",
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    columns[1]
                                                        .checkbox(&mut actor.armor3_fix, "Fixed");
                                                });
                                            })
                                            .response
                                    },
                                ))
                                .changed();
                        });

                        ui.with_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Starting Accessory",
                                    |ui: &mut egui::Ui| {
                                        egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(2, |columns| {
                                                    columns[0].add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            (actor.id, "armor4_id"),
                                                            &mut actor.armor4_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Accessory
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{id:0>3}: {}",
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    );
                                                    columns[1]
                                                        .checkbox(&mut actor.armor4_fix, "Fixed");
                                                });
                                            })
                                            .response
                                    },
                                ))
                                .changed();
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Initial Level",
                                        egui::Slider::new(&mut actor.initial_level, 1..=99),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Final Level",
                                        egui::Slider::new(&mut actor.final_level, 1..=99),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "EXP Curve Basis",
                                        egui::Slider::new(&mut actor.exp_basis, 10..=50),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EXP Curve Inflation",
                                        egui::Slider::new(&mut actor.exp_inflation, 10..=50),
                                    ))
                                    .changed();
                            });
                        });

                        self.previous_actor = Some(actor.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            actors.modified = true;
        }
    }
}
