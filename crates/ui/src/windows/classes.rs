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
    selected_class_name: Option<String>,
    previous_class: Option<usize>,

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
        let _system = update_state.data.system();
        let _states = update_state.data.states();
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
                    |class| format!("{:0>3}: {}", class.id, class.name),
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
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    (class.id, "weapon_set"),
                                    &mut class.weapon_set,
                                    weapons.data.len(),
                                    |id| {
                                        weapons.data.get(id).map_or_else(
                                            || "".into(),
                                            |w| format!("{id:0>3}: {}", w.name),
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
                                    armors.data.len(),
                                    |id| {
                                        armors.data.get(id).map_or_else(
                                            || "".into(),
                                            |a| format!("{id:0>3}: {}", a.name),
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
