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

use luminol_modals::graphic_picker::basic::Modal as GraphicPicker;
use luminol_modals::sound_picker::Modal as SoundPicker;

/// Database - Items management window.
pub struct Window {
    selected_item_name: Option<String>,

    menu_se_picker: SoundPicker,
    graphic_picker: GraphicPicker,

    previous_item: Option<usize>,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let items = update_state.data.items();
        let item = &items.data[0];
        Self {
            selected_item_name: None,
            menu_se_picker: SoundPicker::new(luminol_audio::Source::SE, "item_menu_se_picker"),
            graphic_picker: GraphicPicker::new(
                update_state,
                "Graphics/Icons".into(),
                item.icon_name.as_deref(),
                egui::vec2(32., 32.),
                "item_icon_picker",
            ),
            previous_item: None,
            view: luminol_components::DatabaseView::new(),
        }
    }
}

impl luminol_core::Window for Window {
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
        let data = std::mem::take(update_state.data); // take data to avoid borrow checker issues
        let mut items = data.items();
        let animations = data.animations();
        let common_events = data.common_events();
        let system = data.system();
        let states = data.states();

        let mut modified = false;

        self.selected_item_name = None;

        let name = if let Some(name) = &self.selected_item_name {
            format!("Editing item {:?}", name)
        } else {
            "Item Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Items",
                    &mut items.data,
                    |item| format!("{:0>4}: {}", item.id + 1, item.name),
                    |ui, items, id, update_state| {
                        let item = &mut items[id];
                        self.selected_item_name = Some(item.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            ui.horizontal(|ui| {
                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Icon",
                                        self.graphic_picker
                                            .button(&mut item.icon_name, update_state),
                                    ))
                                    .changed();
                                if self.previous_item != Some(item.id) {
                                    // avoid desyncs by resetting the modal if the item has changed
                                    self.graphic_picker.reset(update_state, &mut item.icon_name);
                                }

                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Name",
                                        egui::TextEdit::singleline(&mut item.name)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });

                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Description",
                                    egui::TextEdit::multiline(&mut item.description)
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

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "User Animation",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
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
                                            update_state,
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

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Menu Use SE",
                                        self.menu_se_picker.button(&mut item.menu_se, update_state),
                                    ))
                                    .changed();
                                if self.previous_item != Some(item.id) {
                                    // reset the modal if the item has changed (this is practically a no-op)
                                    self.menu_se_picker.reset(update_state, &mut item.menu_se);
                                }

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Common Event",
                                        luminol_components::OptionalIdComboBox::new(
                                            update_state,
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

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Price",
                                        egui::DragValue::new(&mut item.price).range(0..=i32::MAX),
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

                        ui.with_padded_stripe(true, |ui| {
                            if item.parameter_type.is_none() {
                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Parameter",
                                        luminol_components::EnumComboBox::new(
                                            "parameter_type",
                                            &mut item.parameter_type,
                                        ),
                                    ))
                                    .changed();
                            } else {
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
                                        .add(luminol_components::Field::new(
                                            "Parameter Increment",
                                            egui::DragValue::new(&mut item.parameter_points)
                                                .range(0..=i32::MAX),
                                        ))
                                        .changed();
                                });
                            }
                        });

                        ui.with_padded_stripe(false, |ui| {
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
                                            .range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
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
                                            .range(0..=i32::MAX),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
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

                        ui.with_padded_stripe(true, |ui| {
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

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let mut selection = luminol_components::IdVecSelection::new(
                                    update_state,
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
                                        update_state,
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

        drop(items);
        drop(animations);
        drop(common_events);
        drop(system);
        drop(states);

        *update_state.data = data; // restore data
    }
}
