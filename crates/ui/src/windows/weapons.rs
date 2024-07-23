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
    selected_weapon_name: Option<String>,
    previous_weapon: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("weapon_editor")
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
        let mut weapons = data.weapons();
        let animations = data.animations();
        let system = data.system();
        let states = data.states();

        let mut modified = false;

        self.selected_weapon_name = None;

        let name = if let Some(name) = &self.selected_weapon_name {
            format!("Editing weapon {:?}", name)
        } else {
            "Weapon Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Weapons",
                    &mut weapons.data,
                    |weapon| format!("{:0>4}: {}", weapon.id + 1, weapon.name),
                    |ui, weapons, id, update_state| {
                        let weapon = &mut weapons[id];
                        self.selected_weapon_name = Some(weapon.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut weapon.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();

                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Description",
                                    egui::TextEdit::multiline(&mut weapon.description)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "User Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
                                            (weapon.id, "animation1_id"),
                                            &mut weapon.animation1_id,
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
                                            (weapon.id, "animation2_id"),
                                            &mut weapon.animation2_id,
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
                                        "Price",
                                        egui::DragValue::new(&mut weapon.price).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "ATK",
                                        egui::DragValue::new(&mut weapon.atk).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "PDEF",
                                        egui::DragValue::new(&mut weapon.pdef).range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "MDEF",
                                        egui::DragValue::new(&mut weapon.mdef).range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "STR+",
                                        egui::DragValue::new(&mut weapon.str_plus),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "DEX+",
                                        egui::DragValue::new(&mut weapon.dex_plus),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "AGI+",
                                        egui::DragValue::new(&mut weapon.agi_plus),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "INT+",
                                        egui::DragValue::new(&mut weapon.int_plus),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
                                    (weapon.id, "element_set"),
                                    &mut weapon.element_set,
                                    1..system.elements.len(),
                                    |id| {
                                        system.elements.get(id).map_or_else(
                                            || "".into(),
                                            |e| format!("{id:0>4}: {}", e),
                                        )
                                    },
                                );
                                if self.previous_weapon != Some(weapon.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new("Elements", selection))
                                    .changed();

                                let mut selection =
                                    luminol_components::IdVecPlusMinusSelection::new(
                                        update_state,
                                        (weapon.id, "state_set"),
                                        &mut weapon.plus_state_set,
                                        &mut weapon.minus_state_set,
                                        0..states.data.len(),
                                        |id| {
                                            states.data.get(id).map_or_else(
                                                || "".into(),
                                                |s| format!("{:0>4}: {}", id + 1, s.name),
                                            )
                                        },
                                    );
                                if self.previous_weapon != Some(weapon.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("State Change", selection))
                                    .changed();
                            });
                        });

                        self.previous_weapon = Some(weapon.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            weapons.modified = true;
        }

        drop(weapons);
        drop(animations);
        drop(system);
        drop(states);

        *update_state.data = data; // restore data
    }
}
