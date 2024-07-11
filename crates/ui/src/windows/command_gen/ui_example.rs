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

use command_lib::{CommandDescription, CommandKind, Parameter, ParameterKind};

pub struct UiExample {
    command: CommandDescription,
}

impl UiExample {
    pub fn new(desc: &CommandDescription) -> Self {
        Self {
            command: desc.clone(),
        }
    }

    pub fn update(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        egui::Window::new(format!(
            "[{}] {} UI example",
            self.command.code, self.command.name
        ))
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.command.name).monospace());
            ui.label(egui::RichText::new(&self.command.description).monospace());

            ui.separator();

            let mut index = 0;
            match self.command.kind {
                CommandKind::Branch {
                    ref mut parameters, ..
                }
                | CommandKind::Single(ref mut parameters) => {
                    for parameter in parameters {
                        Self::parameter_ui(ui, parameter, &mut index);
                    }
                }
                CommandKind::Multi { .. } => {
                    ui.text_edit_multiline(&mut "".to_string());
                }
            }
        });
        open
    }

    fn parameter_ui(ui: &mut egui::Ui, parameter: &mut Parameter, index: &mut u8) {
        match parameter {
            Parameter::Selection {
                ref mut parameters, ..
            } => {
                for (_, parameter) in parameters {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut false, "");
                        ui.vertical(|ui| {
                            ui.add_enabled_ui(false, |ui| Self::parameter_ui(ui, parameter, index));
                        });
                    });
                }
            }
            Parameter::Group {
                ref mut parameters, ..
            } => {
                ui.group(|ui| {
                    for parameter in parameters {
                        Self::parameter_ui(ui, parameter, index);
                    }
                });
            }
            Parameter::Single {
                index: parameter_index,
                description,
                name,
                kind,
                ..
            } => {
                if !name.is_empty() {
                    ui.label(format!("[{}]: {}", parameter_index.as_u8(), name,))
                        .on_hover_text(&*description);
                }

                match kind {
                    ParameterKind::Switch => {
                        ui.button("Switch: [000: EXAMPLE]").clicked();
                    }
                    ParameterKind::Variable => {
                        ui.button("Variable [000: EXAMPLE]").clicked();
                    }
                    ParameterKind::String => {
                        ui.text_edit_singleline(&mut "".to_string());
                    }
                    ParameterKind::Int => {
                        ui.add(egui::DragValue::new(&mut 0i16));
                    }
                    ParameterKind::IntBool => {
                        ui.checkbox(&mut false, "");
                    }
                    ParameterKind::Enum { ref variants } => {
                        let (first_name, mut first_id) = variants.first().unwrap();
                        ui.menu_button(format!("{first_name} ⏷"), |ui| {
                            for (name, id) in variants.iter() {
                                ui.selectable_value(&mut first_id, *id, name);
                            }
                        });
                    }
                    ParameterKind::SelfSwitch => {
                        ui.menu_button("A ⏷", |ui| {
                            for char in ['A', 'B', 'C', 'D'] {
                                ui.selectable_value(&mut 'A', char, char.to_string());
                            }
                        });
                    }
                }
            }
            Parameter::Dummy => {}
            Parameter::Label(ref l) => {
                ui.label(l);
            }
        }

        *index += 1;
    }
}
