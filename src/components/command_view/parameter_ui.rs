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

use crate::prelude::*;

use command_lib::{Parameter, ParameterKind};

impl CommandView {
    #[allow(clippy::only_used_in_recursion)]
    pub fn parameter_ui(
        &mut self,
        ui: &mut egui::Ui,
        parameter: &Parameter,
        command: &mut rpg::EventCommand,
    ) {
        match parameter {
            Parameter::Selection {
                index, parameters, ..
            } => {
                if !parameters.iter().map(|(i, _)| *i).any(|v| {
                    v as i32 == *get_or_resize!(command.parameters, index.as_usize()).into_integer()
                }) {
                    ui.label(error!("Your definition of this command is likely incomplete. This selection has an out of bounds value 🔥"));
                    return;
                }
                for (value, parameter) in parameters.iter() {
                    ui.horizontal(|ui| {
                        let selection =
                            get_or_resize!(command.parameters, index.as_usize()).into_integer();
                        ui.radio_value(selection, *value as _, "");
                        ui.add_enabled_ui(*value as i32 == *selection, |ui| {
                            self.parameter_ui(ui, parameter, command);
                        });
                    });
                }
            }
            Parameter::Group { parameters, .. } => {
                ui.group(|ui| {
                    for parameter in parameters.iter() {
                        self.parameter_ui(ui, parameter, command);
                    }
                });
            }
            Parameter::Dummy => {}
            Parameter::Label(l) => {
                ui.label(l);
            }

            Parameter::Single {
                index, kind, guid, .. // TODO: Display description and name
            } => {
                if !ui.is_enabled() {
                    match kind {
                        ParameterKind::Int => {
                            ui.add(egui::DragValue::new(&mut 0));
                        }
                        ParameterKind::String => {
                            ui.text_edit_singleline(&mut "");
                        }
                        _ => {
                            let _ = ui.button("Disabled");
                        }
                    }
                    return;
                }

                let value = get_or_resize!(command.parameters, index.as_usize());
                match kind {
                    ParameterKind::Int => {
                        ui.add(egui::DragValue::new(value.into_integer()));
                    }
                    ParameterKind::IntBool => {
                        let mut bool = value.truthy();
                        ui.checkbox(&mut bool, "");
                        *value.into_integer() = bool as _;
                    }
                    ParameterKind::Enum { variants } => {
                        let value = value.into_integer();
                        ui.menu_button(
                            format!(
                                "{} ⏷",
                                variants
                                    .iter()
                                    .find(|(_, i)| *i == *value as i8)
                                    .map(|(s, _)| s.as_str())
                                    .unwrap_or("Invalid value")
                            ),
                            |ui| {
                                for (name, variant) in variants.iter() {
                                    ui.selectable_value(value, *variant as _, name);
                                }
                            },
                        );
                    }

                    ParameterKind::SelfSwitch => {
                        let value = value.into_string_with("A".to_string()); // we convert the value into
                        ui.menu_button(format!("Self Switch {value} ⏷"), |ui| {
                            ui.selectable_value(value, "A".to_string(), "A");
                            ui.selectable_value(value, "B".to_string(), "B");
                            ui.selectable_value(value, "C".to_string(), "C");
                            ui.selectable_value(value, "D".to_string(), "D");
                        });
                    }
                    ParameterKind::Switch => {
                        let mut data = *value.into_integer_with(1) as usize;
                        let state = self.modals.entry(*guid).or_insert(false);
                        switch::Modal::new(self.id.with(guid)).button(ui, state, &mut data);
                        *value.into_integer() = data as i32;
                    }
                    ParameterKind::Variable => {
                        let mut data = *value.into_integer_with(1) as usize;
                        let state = self.modals.entry(*guid).or_insert(false);
                        variable::Modal::new(self.id.with(guid)).button(ui, state, &mut data);
                        *value.into_integer() = data as i32;
                    }

                    ParameterKind::String => {
                        ui.text_edit_singleline(value.into_string());
                    }
                }
            }
        }
    }
}
