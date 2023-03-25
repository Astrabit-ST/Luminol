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

use command_lib::CommandKind;
use rmxp_types::rpg;

use crate::data::command_db::CommandDB;

pub struct CommandView<'a> {
    commands: &'a mut Vec<rpg::EventCommand>,
    insert_index: Option<usize>,
}

macro_rules! color_text {
    ($text:expr, $color:expr) => {
        egui::RichText::new($text).color($color)
    };
}

macro_rules! error {
    ($text:expr) => {
        color_text!($text, egui::Color32::RED)
    };
}

impl<'a> CommandView<'a> {
    pub fn new(commands: &'a mut Vec<rpg::EventCommand>) -> Self {
        Self {
            commands,
            insert_index: None,
        }
    }

    pub fn ui(mut self, ui: &mut egui::Ui, db: &CommandDB) {
        for (index, command) in self.commands.iter_mut().enumerate() {
            let desc = match db.get(command.code as _) {
                Some(desc) => desc,
                None if command.code == 0 => {
                    if ui.button("> Insert").double_clicked() {
                        self.insert_index = Some(index);
                    }

                    continue;
                }
                None => {
                    ui.monospace(error!(format!("Unrecognized command {} 🔥", command.code)));
                    ui.horizontal(|ui| {
                        ui.monospace(error!(
                            "You may want to add this command to the database if it is custom"
                        ));
                        ui.hyperlink_to(
                            "or file an issue.",
                            "https://github.com/Astrabit-ST/Luminol/issues",
                        )
                    });

                    continue;
                }
            };

            let color = match desc.kind {
                CommandKind::Branch(_) => egui::Color32::BLUE,
                CommandKind::Multi(_) => egui::Color32::GOLD,
                CommandKind::Single => egui::Color32::WHITE,
            };

            ui.label(color_text!(&desc.name, color));
        }

        if let Some(index) = self.insert_index {
            if !InsertCommandWindow::new(index, self.commands).show(ui.ctx(), db) {
                self.insert_index = None;
            }
        }
    }
}

struct InsertCommandWindow<'a> {
    index: usize,
    commands: &'a mut Vec<rpg::EventCommand>,
}

impl<'a> InsertCommandWindow<'a> {
    fn new(index: usize, commands: &'a mut Vec<rpg::EventCommand>) -> Self {
        Self { index, commands }
    }

    fn show(mut self, ctx: &egui::Context, db: &CommandDB) -> bool {
        let mut open = true;
        let mut save_changes = false;
        let mut window_open = true;

        egui::Window::new("Event Commands")
            .open(&mut window_open)
            .show(ctx, |ui| {
                if ui.button("Ok").clicked() {
                    open = false;
                    save_changes = true;
                }

                if ui.button("Cancel").clicked() {
                    open = false;
                }

                if ui.button("Apply").clicked() {
                    save_changes = true;
                }
            });

        if save_changes {}

        open & window_open
    }
}
