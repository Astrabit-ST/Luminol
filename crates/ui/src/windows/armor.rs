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

#[derive(Default)]
pub struct Window {
    selected_armor_name: Option<String>,
    previous_armor: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        if let Some(name) = &self.selected_armor_name {
            format!("Editing armor {:?}", name)
        } else {
            "Armor Editor".into()
        }
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("armor_editor")
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
        let mut armors = update_state.data.armors();
        let system = update_state.data.system();
        let states = update_state.data.states();

        let mut modified = false;

        self.selected_armor_name = None;

        let response = egui::Window::new(self.name())
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    "Armor",
                    update_state
                        .project_config
                        .as_ref()
                        .expect("project not loaded"),
                    &mut armors.data,
                    |armor| format!("{:0>3}: {}", armor.id, armor.name),
                    |ui, armor| {
                        self.selected_armor_name = Some(armor.name.clone());

                        modified |= ui
                            .add(luminol_components::Field::new(
                                "Name",
                                egui::TextEdit::singleline(&mut armor.name)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();

                        modified |= ui
                            .add(luminol_components::Field::new(
                                "Description",
                                egui::TextEdit::multiline(&mut armor.description)
                                    .desired_width(f32::INFINITY),
                            ))
                            .changed();

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Kind",
                                        luminol_components::EnumComboBox::new(
                                            (armor.id, "kind"),
                                            &mut armor.kind,
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Auto State",
                                        luminol_components::OptionalIdComboBox::new(
                                            (armor.id, "auto_state"),
                                            &mut armor.auto_state_id,
                                            states.data.len(),
                                            |id| {
                                                states.data.get(id).map_or_else(
                                                    || "".into(),
                                                    |s| format!("{id:0>3}: {}", s.name),
                                                )
                                            },
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Price",
                                        egui::DragValue::new(&mut armor.price)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EVA",
                                        egui::DragValue::new(&mut armor.eva)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "PDEF",
                                        egui::DragValue::new(&mut armor.pdef)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "MDEF",
                                        egui::DragValue::new(&mut armor.mdef)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(4, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "STR+",
                                        egui::DragValue::new(&mut armor.str_plus),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "DEX+",
                                        egui::DragValue::new(&mut armor.dex_plus),
                                    ))
                                    .changed();

                                modified |= columns[2]
                                    .add(luminol_components::Field::new(
                                        "AGI+",
                                        egui::DragValue::new(&mut armor.agi_plus),
                                    ))
                                    .changed();

                                modified |= columns[3]
                                    .add(luminol_components::Field::new(
                                        "INT+",
                                        egui::DragValue::new(&mut armor.int_plus),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    (armor.id, "guard_element_set"),
                                    &mut armor.guard_element_set,
                                    system.elements.len(),
                                    |id| {
                                        system.elements.get(id).map_or_else(
                                            || "".into(),
                                            |e| format!("{id:0>3}: {}", e),
                                        )
                                    },
                                );
                                if self.previous_armor != Some(armor.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Element Defense",
                                        selection,
                                    ))
                                    .changed();

                                let mut selection = luminol_components::IdVecSelection::new(
                                    (armor.id, "guard_state_set"),
                                    &mut armor.guard_state_set,
                                    states.data.len(),
                                    |id| {
                                        states.data.get(id).map_or_else(
                                            || "".into(),
                                            |s| format!("{id:0>3}: {}", s.name),
                                        )
                                    },
                                );
                                if self.previous_armor != Some(armor.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("States", selection))
                                    .changed();
                            });
                        });

                        self.previous_armor = Some(armor.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            armors.modified = true;
        }
    }
}
