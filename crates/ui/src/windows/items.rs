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

/// Database - Items management window.
#[derive(Default)]
pub struct Window {
    // ? Items ?
    selected_item_name: Option<String>,

    // ? Icon Graphic Picker ?
    _icon_picker: Option<luminol_modals::graphic_picker::Modal>,

    // ? Menu Sound Effect Picker ?
    _menu_se_picker: Option<luminol_modals::sound_picker::Modal>,

    previous_item: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new() -> Self {
        Default::default()
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        if let Some(name) = &self.selected_item_name {
            format!("Editing item {:?}", name)
        } else {
            "Item Editor".into()
        }
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("item_editor")
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
        let mut items = update_state.data.items();
        let animations = update_state.data.animations();
        let common_events = update_state.data.common_events();
        let system = update_state.data.system();
        let states = update_state.data.states();

        let mut modified = false;

        self.selected_item_name = None;

        let response = egui::Window::new(self.name())
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    "Items",
                    update_state
                        .project_config
                        .as_ref()
                        .expect("project not loaded"),
                    &mut items.data,
                    |item| format!("{:0>4}: {}", item.id + 1, item.name),
                    |ui, item| {
                        self.selected_item_name = Some(item.name.clone());

                        ui.with_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut item.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();

                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Description",
                                    egui::TextEdit::multiline(&mut item.description)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Scope",
                                        luminol_components::EnumComboBox::new(
                                            (item.id, "scope"),
                                            &mut item.scope,
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Occasion",
                                        luminol_components::EnumComboBox::new(
                                            (item.id, "occasion"),
                                            &mut item.occasion,
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "User Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            (item.id, "animation1_id"),
                                            &mut item.animation1_id,
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
                                            (item.id, "animation2_id"),
                                            &mut item.animation2_id,
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

                        ui.with_stripe(true, |ui| {
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
                                            (item.id, "common_event_id"),
                                            &mut item.common_event_id,
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

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Price",
                                        egui::DragValue::new(&mut item.price)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Consumable",
                                        egui::Checkbox::without_text(&mut item.consumable),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Parameter",
                                        luminol_components::EnumComboBox::new(
                                            "parameter_type",
                                            &mut item.parameter_type,
                                        ),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add_enabled(
                                        !item.parameter_type.is_none(),
                                        luminol_components::Field::new(
                                            "Parameter Increment",
                                            egui::DragValue::new(&mut item.parameter_points)
                                                .clamp_range(0..=i32::MAX),
                                        ),
                                    )
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Recover HP %",
                                        egui::Slider::new(&mut item.recover_hp_rate, 0..=100)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Recover HP Points",
                                        egui::DragValue::new(&mut item.recover_hp)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Recover SP %",
                                        egui::Slider::new(&mut item.recover_sp_rate, 0..=100)
                                            .suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Recover SP Points",
                                        egui::DragValue::new(&mut item.recover_sp)
                                            .clamp_range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Hit Rate",
                                        egui::Slider::new(&mut item.hit, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Variance",
                                        egui::Slider::new(&mut item.variance, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "PDEF-F",
                                        egui::Slider::new(&mut item.pdef_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "MDEF-F",
                                        egui::Slider::new(&mut item.mdef_f, 0..=100).suffix("%"),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    (item.id, "element_set"),
                                    &mut item.element_set,
                                    1..system.elements.len(),
                                    |id| {
                                        system.elements.get(id).map_or_else(
                                            || "".into(),
                                            |e| format!("{id:0>4}: {}", e),
                                        )
                                    },
                                );
                                if self.previous_item != Some(item.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[0]
                                    .add(luminol_components::Field::new("Elements", selection))
                                    .changed();

                                let mut selection =
                                    luminol_components::IdVecPlusMinusSelection::new(
                                        (item.id, "state_set"),
                                        &mut item.plus_state_set,
                                        &mut item.minus_state_set,
                                        0..states.data.len(),
                                        |id| {
                                            states.data.get(id).map_or_else(
                                                || "".into(),
                                                |s| format!("{:0>4}: {}", id + 1, s.name),
                                            )
                                        },
                                    );
                                if self.previous_item != Some(item.id) {
                                    selection.clear_search();
                                }
                                modified |= columns[1]
                                    .add(luminol_components::Field::new("State Change", selection))
                                    .changed();
                            });
                        });

                        self.previous_item = Some(item.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            items.modified = true;
        }
    }
}
