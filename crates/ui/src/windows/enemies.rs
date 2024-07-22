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
use luminol_core::Modal;
use luminol_modals::graphic_picker::hue::Modal as GraphicPicker;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(strum::Display, strum::EnumIter)]
#[derive(serde::Deserialize, serde::Serialize)]
pub enum TreasureType {
    #[default]
    None,
    Item,
    Weapon,
    Armor,
}

pub struct Window {
    selected_enemy_name: Option<String>,
    previous_enemy: Option<usize>,

    graphic_picker: GraphicPicker,

    collapsing_view: luminol_components::CollapsingView,
    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let enemies = update_state.data.enemies();
        let enemy = &enemies.data[0];
        Self {
            selected_enemy_name: None,
            previous_enemy: None,

            graphic_picker: GraphicPicker::new(
                update_state,
                "Graphics/Battlers".into(),
                enemy.battler_name.as_deref(),
                enemy.battler_hue,
                egui::vec2(196., 256.),
                "enemy_battler_picker",
            ),

            collapsing_view: luminol_components::CollapsingView::new(),
            view: luminol_components::DatabaseView::new(),
        }
    }

    fn show_action_header(
        ui: &mut egui::Ui,
        skills: &luminol_data::rpg::Skills,
        action: &luminol_data::rpg::enemy::Action,
    ) {
        let mut conditions = Vec::with_capacity(4);
        if action.condition_turn_a != 0 || action.condition_turn_b != 1 {
            conditions.push(if action.condition_turn_b == 0 {
                format!("Turn {}", action.condition_turn_a,)
            } else if action.condition_turn_a == 0 {
                format!("Turn {}x", action.condition_turn_b,)
            } else if action.condition_turn_b == 1 {
                format!("Turn {} + x", action.condition_turn_a,)
            } else {
                format!(
                    "Turn {} + {}x",
                    action.condition_turn_a, action.condition_turn_b,
                )
            })
        }
        if action.condition_hp < 100 {
            conditions.push(format!("{}% HP", action.condition_hp,));
        }
        if action.condition_level > 1 {
            conditions.push(format!("Level {}", action.condition_level,));
        }
        if let Some(id) = action.condition_switch_id {
            conditions.push(format!("Switch {:0>4}", id + 1));
        }

        ui.label(format!(
            "{}{}",
            match action.kind {
                luminol_data::rpg::enemy::Kind::Basic => {
                    action.basic.to_string()
                }
                luminol_data::rpg::enemy::Kind::Skill => {
                    skills
                        .data
                        .get(action.skill_id)
                        .map_or_else(|| "".into(), |s| s.name.clone())
                }
            },
            if conditions.is_empty() {
                String::new()
            } else {
                format!(": {}", conditions.join(", "))
            }
        ));
    }

    fn show_action_body(
        ui: &mut egui::Ui,
        update_state: &luminol_core::UpdateState<'_>,
        system: &luminol_data::rpg::System,
        skills: &luminol_data::rpg::Skills,
        enemy_id: usize,
        action: (usize, &mut luminol_data::rpg::enemy::Action),
    ) -> egui::Response {
        let (action_index, action) = action;
        let mut modified = false;

        let mut response = egui::Frame::none()
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    modified |= columns[0]
                        .add(luminol_components::Field::new(
                            "Turn Offset",
                            egui::DragValue::new(&mut action.condition_turn_a).range(0..=i32::MAX),
                        ))
                        .changed();

                    modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "Turn Interval",
                            egui::DragValue::new(&mut action.condition_turn_b).range(0..=i32::MAX),
                        ))
                        .changed();
                });

                ui.columns(2, |columns| {
                    modified |= columns[0]
                        .add(luminol_components::Field::new(
                            "Max HP %",
                            egui::Slider::new(&mut action.condition_hp, 0..=100).suffix("%"),
                        ))
                        .changed();

                    modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "Min Level",
                            egui::Slider::new(&mut action.condition_level, 1..=99),
                        ))
                        .changed();
                });

                modified |= ui
                    .add(luminol_components::Field::new(
                        "Switch",
                        luminol_components::OptionalIdComboBox::new(
                            update_state,
                            (enemy_id, action_index, "condition_switch_id"),
                            &mut action.condition_switch_id,
                            0..system.switches.len(),
                            |id| {
                                system
                                    .switches
                                    .get(id)
                                    .map_or_else(|| "".into(), |s| format!("{:0>4}: {}", id + 1, s))
                            },
                        ),
                    ))
                    .changed();

                ui.columns(2, |columns| {
                    modified |= columns[0]
                        .add(luminol_components::Field::new(
                            "Kind",
                            luminol_components::EnumComboBox::new(
                                (enemy_id, action_index, "kind"),
                                &mut action.kind,
                            ),
                        ))
                        .changed();

                    match action.kind {
                        luminol_data::rpg::enemy::Kind::Basic => {
                            modified |= columns[1]
                                .add(luminol_components::Field::new(
                                    "Basic Type",
                                    luminol_components::EnumComboBox::new(
                                        (enemy_id, action_index, "basic"),
                                        &mut action.basic,
                                    ),
                                ))
                                .changed();
                        }
                        luminol_data::rpg::enemy::Kind::Skill => {
                            modified |= columns[1]
                                .add(luminol_components::Field::new(
                                    "Skill",
                                    luminol_components::OptionalIdComboBox::new(
                                        update_state,
                                        (enemy_id, action_index, "skill_id"),
                                        &mut action.skill_id,
                                        0..skills.data.len(),
                                        |id| {
                                            skills.data.get(id).map_or_else(
                                                || "".into(),
                                                |s| format!("{:0>4}: {}", id + 1, s.name),
                                            )
                                        },
                                    ),
                                ))
                                .changed();
                        }
                    }
                });

                modified |= ui
                    .add(luminol_components::Field::new(
                        "Rating",
                        egui::Slider::new(&mut action.rating, 1..=10),
                    ))
                    .changed();
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
        egui::Id::new("enemy_editor")
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
        let mut enemies = data.enemies();
        let animations = data.animations();
        let system = data.system();
        let states = data.states();
        let skills = data.skills();
        let items = data.items();
        let weapons = data.weapons();
        let armors = data.armors();

        let mut modified = false;

        self.selected_enemy_name = None;

        let name = if let Some(name) = &self.selected_enemy_name {
            format!("Editing enemy {:?}", name)
        } else {
            "Enemy Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Enemies",
                    &mut enemies.data,
                    |enemy| format!("{:0>4}: {}", enemy.id + 1, enemy.name),
                    |ui, enemies, id, update_state| {
                        let enemy = &mut enemies[id];
                        self.selected_enemy_name = Some(enemy.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            ui.horizontal(|ui| {
                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Graphic",
                                        self.graphic_picker.button(
                                            (&mut enemy.battler_name, &mut enemy.battler_hue),
                                            update_state,
                                        ),
                                    ))
                                    .changed();
                                if self.previous_enemy != Some(enemy.id) {
                                    // avoid desyncs by resetting the modal if the item has changed
                                    self.graphic_picker.reset(
                                        update_state,
                                        (&mut enemy.battler_name, &mut enemy.battler_hue),
                                    );
                                }

                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Name",
                                        egui::TextEdit::singleline(&mut enemy.name)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Attacker Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (enemy.id, "animation1_id"),
                                            &mut enemy.animation1_id,
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
                                            (enemy.id, "animation2_id"),
                                            &mut enemy.animation2_id,
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

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "EXP",
                                        egui::DragValue::new(&mut enemy.exp).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Gold",
                                        egui::DragValue::new(&mut enemy.gold).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "Max HP",
                                        egui::DragValue::new(&mut enemy.maxhp).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "Max SP",
                                        egui::DragValue::new(&mut enemy.maxsp).range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "STR",
                                        egui::DragValue::new(&mut enemy.str).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "DEX",
                                        egui::DragValue::new(&mut enemy.dex).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "AGI",
                                        egui::DragValue::new(&mut enemy.agi).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "INT",
                                        egui::DragValue::new(&mut enemy.int).range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "ATK",
                                        egui::DragValue::new(&mut enemy.atk).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EVA",
                                        egui::DragValue::new(&mut enemy.eva).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "PDEF",
                                        egui::DragValue::new(&mut enemy.pdef).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "MDEF",
                                        egui::DragValue::new(&mut enemy.mdef).range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        let mut treasure_type = if enemy.item_id.is_some() {
                            TreasureType::Item
                        } else if enemy.weapon_id.is_some() {
                            TreasureType::Weapon
                        } else if enemy.armor_id.is_some() {
                            TreasureType::Armor
                        } else {
                            TreasureType::None
                        };

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Treasure Type",
                                        luminol_components::EnumComboBox::new(
                                            (enemy.id, "treasure_type"),
                                            &mut treasure_type,
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Treasure Probability",
                                        egui::Slider::new(&mut enemy.treasure_prob, 0..=100)
                                            .suffix("%"),
                                    ))
                                    .changed();
                            });

                            match treasure_type {
                                TreasureType::None => {
                                    enemy.item_id = None;
                                    enemy.weapon_id = None;
                                    enemy.armor_id = None;
                                }

                                TreasureType::Item => {
                                    enemy.weapon_id = None;
                                    enemy.armor_id = None;
                                    if enemy.item_id.is_none() {
                                        enemy.item_id = Some(0);
                                    }
                                    modified |= ui
                                        .add(luminol_components::Field::new(
                                            "Treasure",
                                            luminol_components::OptionalIdComboBox::new(
                                                update_state,
                                                (enemy.id, "item_id"),
                                                &mut enemy.item_id,
                                                0..items.data.len(),
                                                |id| {
                                                    items.data.get(id).map_or_else(
                                                        || "".into(),
                                                        |i| format!("{:0>4}: {}", id + 1, i.name),
                                                    )
                                                },
                                            )
                                            .allow_none(false),
                                        ))
                                        .changed();
                                }

                                TreasureType::Weapon => {
                                    enemy.item_id = None;
                                    enemy.armor_id = None;
                                    if enemy.weapon_id.is_none() {
                                        enemy.weapon_id = Some(0);
                                    }
                                    modified |= ui
                                        .add(luminol_components::Field::new(
                                            "Treasure",
                                            luminol_components::OptionalIdComboBox::new(
                                                update_state,
                                                (enemy.id, "weapon_id"),
                                                &mut enemy.weapon_id,
                                                0..weapons.data.len(),
                                                |id| {
                                                    weapons.data.get(id).map_or_else(
                                                        || "".into(),
                                                        |w| format!("{:0>4}: {}", id + 1, w.name),
                                                    )
                                                },
                                            )
                                            .allow_none(false),
                                        ))
                                        .changed();
                                }

                                TreasureType::Armor => {
                                    enemy.item_id = None;
                                    enemy.weapon_id = None;
                                    if enemy.armor_id.is_none() {
                                        enemy.armor_id = Some(0);
                                    }
                                    modified |= ui
                                        .add(luminol_components::Field::new(
                                            "Treasure",
                                            luminol_components::OptionalIdComboBox::new(
                                                update_state,
                                                (enemy.id, "armor_id"),
                                                &mut enemy.armor_id,
                                                0..armors.data.len(),
                                                |id| {
                                                    armors.data.get(id).map_or_else(
                                                        || "".into(),
                                                        |a| format!("{:0>4}: {}", id + 1, a.name),
                                                    )
                                                },
                                            )
                                            .allow_none(false),
                                        ))
                                        .changed();
                                }
                            };
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Actions",
                                    |ui: &mut egui::Ui| {
                                        if self.previous_enemy != Some(enemy.id) {
                                            self.collapsing_view.clear_animations();
                                        }
                                        self.collapsing_view.show(
                                            ui,
                                            enemy.id,
                                            &mut enemy.actions,
                                            |ui, _i, action| {
                                                Self::show_action_header(ui, &skills, action)
                                            },
                                            |ui, i, action| {
                                                Self::show_action_body(
                                                    ui,
                                                    update_state,
                                                    &system,
                                                    &skills,
                                                    enemy.id,
                                                    (i, action),
                                                )
                                            },
                                        )
                                    },
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                enemy
                                    .element_ranks
                                    .resize_with_value(system.elements.len(), 3);
                                let mut selection = luminol_components::RankSelection::new(
                                    update_state,
                                    (enemy.id, "element_ranks"),
                                    &mut enemy.element_ranks,
                                    |id| {
                                        system.elements.get(id + 1).map_or_else(
                                            || "".into(),
                                            |e| format!("{:0>4}: {}", id + 1, e),
                                        )
                                    },
                                );
                                if self.previous_enemy != Some(enemy.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new("Elements", selection))
                                    .changed();

                                enemy
                                    .state_ranks
                                    .resize_with_value(states.data.len() + 1, 3);
                                let mut selection = luminol_components::RankSelection::new(
                                    update_state,
                                    (enemy.id, "state_ranks"),
                                    &mut enemy.state_ranks,
                                    |id| {
                                        states.data.get(id).map_or_else(
                                            || "".into(),
                                            |s| format!("{:0>4}: {}", id + 1, s.name),
                                        )
                                    },
                                );
                                if self.previous_enemy != Some(enemy.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("States", selection))
                                    .changed();
                            });
                        });

                        self.previous_enemy = Some(enemy.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            enemies.modified = true;
        }

        drop(enemies);
        drop(animations);
        drop(system);
        drop(states);
        drop(skills);
        drop(items);
        drop(weapons);
        drop(armors);

        *update_state.data = data; // restore data
    }
}
