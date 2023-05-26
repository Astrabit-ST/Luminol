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
use super::WindowState;
use crate::prelude::*;

use command_lib::{CommandKind, Parameter, ParameterKind};

impl CommandView {
    pub fn command_ui<'i, I>(
        &mut self,
        ui: &mut egui::Ui,
        db: &CommandDB,
        (index, command): (usize, &'i mut rpg::EventCommand),
        iter: &mut std::iter::Peekable<I>,
    ) where
        I: Iterator<Item = (usize, &'i mut rpg::EventCommand)>,
    {
        let desc = match db.get(command.code as _) {
            Some(desc) => desc,
            None if command.code == 0 => {
                if ui
                    .selectable_value(&mut self.selected_index, index, "#> Insert")
                    .double_clicked()
                {
                    self.window_state = WindowState::Insert { index, tab: 0 };
                }

                return;
            }
            None => {
                ui.selectable_value(
                    &mut self.selected_index,
                    index,
                    error!(format!("Unrecognized command {} 🔥", command.code)),
                );
                ui.horizontal(|ui| {
                    ui.monospace(error!(
                        "You may want to add this command to the database if it is custom"
                    ));
                    ui.hyperlink_to(
                        "or file an issue.",
                        "https://github.com/Astrabit-ST/Luminol/issues",
                    )
                });

                #[cfg(debug_assertions)]
                {
                    ui.label("Parameters:");
                    ui.label(format!("{:#?}", command.parameters));
                }

                return;
            }
        };

        let label = match desc.kind {
            CommandKind::Branch { ref parameters, .. } | CommandKind::Single(ref parameters) => {
                let mut str = format!("[{}] {}:", desc.code, desc.name);

                for parameter in parameters.iter() {
                    parameter_label(&mut str, parameter, command).expect("failed to format");
                }

                Some(str)
            }
            CommandKind::Multi { .. } => None,
        };

        let response = match desc.kind {
            CommandKind::Branch { end_code, .. } => {
                let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    egui::Id::new(command.guid),
                    true,
                );
                let open = header.is_open();

                let response = header
                    .show_header(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_index,
                            index,
                            color_text!(label.unwrap(), Color32::BLUE),
                        )
                    })
                    .body(|ui| {
                        while let Some((index, command)) = iter.next() {
                            if command.code == end_code {
                                break;
                            }

                            self.command_ui(ui, db, (index, command), iter);
                        }
                    });

                if !open {
                    for (_, command) in iter.by_ref() {
                        if command.code == end_code {
                            break;
                        }
                    }
                }

                ui.label(color_text!("End Branch", Color32::BLUE));

                response.1.inner
            }
            CommandKind::Multi { code, highlight } => {
                let mut str = get_or_resize!(command.parameters, 0).into_string().clone();

                loop {
                    match iter.peek_mut() {
                        Some((_, command)) if command.code == code => {
                            match command.parameters.get_mut(0) {
                                Some(p) => {
                                    str += "\n";
                                    str += p.into_string();
                                }
                                None => break,
                            }

                            iter.next();
                        }
                        _ => break,
                    }
                }

                ui.horizontal(|ui| {
                    ui.label(color_text!(
                        format!("[{}] {}:", desc.code, desc.name),
                        Color32::YELLOW
                    ));
                    if highlight {
                        let theme = state!().saved_state.borrow().theme;

                        ui.selectable_value(
                            &mut self.selected_index,
                            index,
                            syntax_highlighting::highlight(ui.ctx(), theme, &str, "rb"),
                        )
                    } else {
                        ui.selectable_value(&mut self.selected_index, index, str)
                    }
                })
                .inner
            }
            CommandKind::Single(_) => {
                //
                ui.selectable_value(&mut self.selected_index, index, label.unwrap())
            }
        };

        let response = response.context_menu(|ui| {
            if ui.button("Insert above").clicked() {
                self.window_state = WindowState::Insert {
                    index: index.saturating_sub(1),
                    tab: 0,
                };
            }
            if ui.button("Delete").clicked() {}
            if ui.button("Insert below").clicked() {
                self.window_state = WindowState::Insert {
                    index: index + 1,
                    tab: 0,
                };
            }
        });

        if response.double_clicked()
            && match desc.kind {
                CommandKind::Multi { .. } => true,
                CommandKind::Branch { ref parameters, .. }
                | CommandKind::Single(ref parameters) => !parameters.is_empty(),
            }
        {
            self.window_state = WindowState::Edit { index };
        }
    }
}

fn parameter_label(
    string: &mut String,
    parameter: &Parameter,
    command: &mut rpg::EventCommand,
) -> std::fmt::Result {
    use std::fmt::Write;

    match parameter {
        Parameter::Group { parameters, .. } => parameters
            .iter()
            .try_for_each(|p| parameter_label(string, p, command)),
        Parameter::Selection {
            parameters, index, ..
        } => {
            let Some((_, parameter)) = parameters.iter().find(|(i, _)| {
                *i as i32
                    == match command.parameters.get_mut(index.as_usize()) {
                        Some(i) => *i.into_integer(),
                        None => return false
                    }
            }) else {
                return write!(string, " invalid selection");
            };
            parameter_label(string, parameter, command)
        }
        Parameter::Single { kind, index, .. } => {
            let Some(value) = command.parameters.get_mut(index.as_usize()) else {
                return write!(string, " missing value");
            };
            match kind {
                ParameterKind::Int => {
                    write!(string, " {}", value.into_integer())
                }
                ParameterKind::Switch => {
                    let id = *value.into_integer() as usize;
                    let system = state!().data_cache.system();
                    let switch = system
                        .variables
                        .get(id)
                        .map(String::as_str)
                        .unwrap_or(" invalid switch");
                    write!(string, " [{id}: {switch}]")
                }
                ParameterKind::Variable => {
                    let id = *value.into_integer() as usize;
                    let system = state!().data_cache.system();
                    let variable = system
                        .variables
                        .get(id)
                        .map(String::as_str)
                        .unwrap_or(" invalid variable");
                    write!(string, " [{id}: {variable}]")
                }
                ParameterKind::Enum { variants } => {
                    let value = variants
                        .iter()
                        .find(|(_, i)| *i as i32 == *value.into_integer())
                        .map(|(s, _)| s.as_str())
                        .unwrap_or("Invalid value");
                    write!(string, " {}", value)
                }
                ParameterKind::SelfSwitch => {
                    write!(string, " {}", value.into_string())
                }
                ParameterKind::IntBool => {
                    write!(string, " {}", value.truthy())
                }
                ParameterKind::String => {
                    write!(string, " {}", value.into_string())
                }
            }
        }
        Parameter::Label(s) => write!(string, " {s}"),
        Parameter::Dummy => Ok(()),
    }
}
