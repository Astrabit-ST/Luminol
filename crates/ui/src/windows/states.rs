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
    selected_state_name: Option<String>,
    previous_state: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("state_editor")
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
        let mut states = data.states();
        let animations = data.animations();
        let system = data.system();

        let mut modified = false;

        self.selected_state_name = None;

        let name = if let Some(name) = &self.selected_state_name {
            format!("Editing state {:?}", name)
        } else {
            "State Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "States",
                    &mut states.data,
                    |state| format!("{:0>4}: {}", state.id + 1, state.name),
                    |ui, states, id, update_state| {
                        let state = &mut states[id];
                        self.selected_state_name = Some(state.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut state.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (state.id, "animation_id"),
                                            &mut state.animation_id,
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
                                        "Restriction",
                                        luminol_components::EnumComboBox::new(
                                            (state.id, "restriction"),
                                            &mut state.restriction,
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Nonresistance",
                                        egui::Checkbox::without_text(&mut state.nonresistance),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Count as 0 HP",
                                        egui::Checkbox::without_text(&mut state.zero_hp),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(3, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Can't Get EXP",
                                        egui::Checkbox::without_text(&mut state.cant_get_exp),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Can't Evade",
                                        egui::Checkbox::without_text(&mut state.cant_evade),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "Slip Damage",
                                        egui::Checkbox::without_text(&mut state.slip_damage),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Rating",
                                        egui::DragValue::new(&mut state.rating).range(0..=10),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EVA",
                                        egui::DragValue::new(&mut state.eva),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Max HP %",
                                        egui::Slider::new(&mut state.maxhp_rate, 0..=200)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Max SP %",
                                        egui::Slider::new(&mut state.maxsp_rate, 0..=200)
                                            .suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "STR %",
                                        egui::Slider::new(&mut state.str_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "DEX %",
                                        egui::Slider::new(&mut state.dex_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "AGI %",
                                        egui::Slider::new(&mut state.agi_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "INT %",
                                        egui::Slider::new(&mut state.int_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Hit Rate %",
                                        egui::Slider::new(&mut state.hit_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "ATK %",
                                        egui::Slider::new(&mut state.atk_rate, 0..=200).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "PDEF %",
                                        egui::Slider::new(&mut state.pdef_rate, 0..=200)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "MDEF %",
                                        egui::Slider::new(&mut state.mdef_rate, 0..=200)
                                            .suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Auto Release Probability",
                                        egui::Slider::new(&mut state.auto_release_prob, 0..=100)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Auto Release Interval",
                                        egui::DragValue::new(&mut state.hold_turn)
                                            .range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Damage Release Probability",
                                        egui::Slider::new(&mut state.shock_release_prob, 0..=100)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Battle Only",
                                        egui::Checkbox::without_text(&mut state.battle_only),
                                    ))
                                    .changed();
                            });
                        });

                        let mut state = std::mem::take(state);
                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
                                    (state.id, "guard_element_set"),
                                    &mut state.guard_element_set,
                                    1..system.elements.len(),
                                    |id| {
                                        system.elements.get(id).map_or_else(
                                            || "".into(),
                                            |e| format!("{id:0>4}: {}", e),
                                        )
                                    },
                                );
                                if self.previous_state != Some(state.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Element Defense",
                                        selection,
                                    ))
                                    .changed();

                                let mut selection =
                                    luminol_components::IdVecPlusMinusSelection::new(
                                        update_state,
                                        (state.id, "state_set"),
                                        &mut state.plus_state_set,
                                        &mut state.minus_state_set,
                                        0..states.len(),
                                        |id| {
                                            if id == state.id {
                                                format!("{:0>4}: {}", id + 1, state.name)
                                            } else {
                                                states.get(id).map_or_else(
                                                    || "".into(),
                                                    |s| format!("{:0>4}: {}", id + 1, s.name),
                                                )
                                            }
                                        },
                                    );
                                if self.previous_state != Some(state.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("State Change", selection))
                                    .changed();
                            });
                        });
                        states[id] = state;

                        self.previous_state = Some(id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            states.modified = true;
        }

        drop(states);
        drop(animations);
        drop(system);

        *update_state.data = data; // restore data
    }
}
