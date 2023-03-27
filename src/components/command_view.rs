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

use command_lib::{CommandKind, Parameter, ParameterKind};
use egui::Color32;
use rmxp_types::rpg;

use crate::data::command_db::CommandDB;

pub struct CommandView {
    selected_index: usize,
}

impl Default for CommandView {
    fn default() -> Self {
        Self { selected_index: 0 }
    }
}

macro_rules! color_text {
    ($text:expr, $color:expr) => {
        egui::RichText::new($text).monospace().color($color)
    };
}

macro_rules! error {
    ($text:expr) => {
        color_text!($text, egui::Color32::RED)
    };
}

macro_rules! get_or_resize {
    ($var:expr, $index:expr) => {
        if let Some(v) = $var.get_mut($index) {
            v
        } else {
            $var.resize_with($index + 1, Default::default);
            &mut $var[$index]
        }
    };
}

macro_rules! get_or_return {
    ($var:expr, $index:expr) => {
        if let Some(v) = $var.get_mut($index) {
            v
        } else {
            return;
        }
    };
}

impl CommandView {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::ptr_arg)]
    pub fn ui(&mut self, ui: &mut egui::Ui, db: &CommandDB, commands: &mut Vec<rpg::EventCommand>) {
        let mut iter = commands.iter_mut().enumerate().peekable();
        while let Some(i) = iter.next() {
            self.command_ui(ui, db, i, &mut iter);
        }
    }

    fn command_ui<'i, I>(
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
                if ui.button("#> Insert").double_clicked() {
                    // TODO INSERT
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

        match desc.kind {
            CommandKind::Branch(end_code) => {
                egui::CollapsingHeader::new(color_text!(
                    format!("{}: {}", desc.code, desc.name),
                    Color32::BLUE
                ))
                .id_source(command.guid)
                .default_open(true)
                .show(ui, |ui| {
                    while let Some((index, command)) = iter.next() {
                        if command.code == end_code {
                            break;
                        }

                        self.command_ui(ui, db, (index, command), iter);
                    }
                });
                ui.label(color_text!("End Branch", Color32::BLUE));
            }
            CommandKind::Multi(rep_code) => {
                let highlight = match desc.parameters.get(0) {
                    Some(Parameter::Single {
                        kind: ParameterKind::StringMulti { highlight },
                        ..
                    }) => *highlight,
                    _ => {
                        ui.label(error!("Multi command was declared incorrectly 🔥"));
                        return;
                    }
                };

                let mut str = format!(
                    "{}: {}\n",
                    desc.name,
                    get_or_resize!(command.parameters, 0).into_string()
                );

                loop {
                    match iter.peek_mut() {
                        Some((_, command)) if command.code == rep_code => {
                            str += &format!(
                                "{}: {}\n",
                                " ".repeat(desc.name.len()),
                                get_or_resize!(command.parameters, 0).into_string()
                            );

                            iter.next();
                        }
                        _ => break,
                    }
                }

                if highlight {
                    let theme =
                        crate::components::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                    ui.selectable_value(
                        &mut self.selected_index,
                        index,
                        crate::components::syntax_highlighting::highlight(
                            ui.ctx(),
                            &theme,
                            &str,
                            "rb",
                        ),
                    );
                } else {
                    ui.selectable_value(&mut self.selected_index, index, str);
                }
            }
            CommandKind::Single => {}
        }
    }
}
