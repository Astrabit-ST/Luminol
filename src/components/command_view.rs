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

pub struct CommandView {
    insert_window: Option<InsertCommandWindow>,
}

impl Default for CommandView {
    fn default() -> Self {
        Self {
            insert_window: None,
        }
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

impl CommandView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, db: &CommandDB, commands: &mut Vec<rpg::EventCommand>) {
        for (index, command) in commands.iter_mut().enumerate() {
            let desc = match db.get(command.code as _) {
                Some(desc) => desc,
                None if command.code == 0 => {
                    if ui.button("#> Insert").double_clicked() {
                        self.insert_window = Some(InsertCommandWindow::new(index));
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

        if let Some(ref mut window) = self.insert_window {
            if !window.show(ui.ctx(), db, commands) {
                self.insert_window = None;
            }
        }
    }
}

struct InsertCommandWindow {
    index: usize,
}

impl InsertCommandWindow {
    fn new(index: usize) -> Self {
        Self { index }
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        db: &CommandDB,
        commands: &mut Vec<rpg::EventCommand>,
    ) -> bool {
        let mut open = true;
        let mut save_changes = false;
        let mut window_open = true;

        egui::Window::new("Event Commands")
            .open(&mut window_open)
            .show(ctx, |ui| {
                super::close_options_ui(ui, &mut open, &mut save_changes);
            });

        if save_changes {}

        open & window_open
    }
}
