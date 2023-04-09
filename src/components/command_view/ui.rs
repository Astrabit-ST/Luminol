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

use command_lib::CommandKind;
use itertools::Itertools;

impl CommandView {
    #[allow(clippy::ptr_arg)]
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        db: &CommandDB,
        commands: &mut Vec<rpg::EventCommand>,
        info: &'static UpdateInfo,
    ) {
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
        let mut iter = commands.iter_mut().enumerate().peekable();
        while let Some(i) = iter.next() {
            self.command_ui(ui, db, i, &mut iter, info);
        }

        ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowUp) {
                self.selected_index = self.selected_index.saturating_sub(1);
            }

            if i.key_pressed(egui::Key::ArrowDown) {
                self.selected_index = (self.selected_index + 1).min(commands.len() - 1);
            }

            if i.key_pressed(egui::Key::Enter) {
                let index = self.selected_index;
                match commands[index].code {
                    0 => self.window_state = WindowState::Insert { index, tab: 0 },
                    _ => self.window_state = WindowState::Edit { index },
                }
            }
        });

        let mut open = true;
        match self.window_state {
            WindowState::Edit { index } => {
                egui::Window::new(format!("Editing a command at {index}"))
                    .open(&mut open)
                    .id(self.id.with("window"))
                    .show(ui.ctx(), |ui| {
                        let Some(command) = commands.get_mut(index) else {
                            self.window_state = WindowState::None;
                            eprintln!("editing command does not exist");
                            return;
                        };
                        let desc = db.get(command.code).expect(
                            "the user is editing a command that is invalid. this should not happen",
                        );

                        ui.monospace(&desc.name);
                        if !desc.description.is_empty() {
                            ui.monospace(&desc.description);
                        }
                        ui.separator();

                        match desc.kind {
                            CommandKind::Branch { ref parameters, .. }
                            | CommandKind::Single(ref parameters) => {
                                for parameter in parameters {
                                    self.parameter_ui(ui, parameter, command, info)
                                }
                            }
                            CommandKind::Multi { code, highlight } => {
                                let mut str = String::new();
                                let indent = command.indent;

                                for command in &mut commands[index..] {
                                    if command.code != code && command.code != desc.code {
                                        break;
                                    }

                                    str += command.parameters[0].into_string();
                                    str += "\n";
                                }

                                let mut text_edit = egui::TextEdit::multiline(&mut str)
                                    .code_editor()
                                    .desired_width(f32::INFINITY);
                                let mut layouter =
                                    |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                        let theme = info.saved_state.borrow().theme;
                                        let mut layout_job = syntax_highlighting::highlight(
                                            ui.ctx(),
                                            theme,
                                            string,
                                            "rb",
                                        );
                                        layout_job.wrap.max_width = wrap_width;
                                        ui.fonts(|f| f.layout_job(layout_job))
                                    };

                                if highlight {
                                    text_edit = text_edit.layouter(&mut layouter);
                                }

                                if ui.add(text_edit).changed() {
                                    let mut index = index;
                                    let mut command_code = commands[index].code;

                                    for s in str.lines() {
                                        if command_code == code || command_code == desc.code {
                                            *get_or_resize!(commands[index].parameters, 0)
                                                .into_string() = s.to_string();
                                        } else {
                                            commands.insert(
                                                index,
                                                rpg::EventCommand {
                                                    code,
                                                    indent,
                                                    parameters: vec![s.into()],
                                                    guid: rand::random(),
                                                },
                                            )
                                        }
                                        index += 1;
                                        command_code = commands[index].code;
                                    }

                                    if command_code == code {
                                        let mut range = index..index;
                                        while commands
                                            .get(range.end)
                                            .is_some_and(|c| c.code == code)
                                        {
                                            range.end += 1;
                                        }
                                        commands.drain(range);
                                    }
                                }
                            }
                        }
                    });
            }
            WindowState::Insert { index, mut tab } => {
                egui::Window::new(format!("Inserting a command at index {index}"))
                    .open(&mut open)
                    .id(self.id.with("window"))
                    .show(ui.ctx(), |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..=(db.len() / 32) {
                                ui.selectable_value(&mut tab, i, i.to_string());
                                ui.separator();
                            }
                            self.window_state = WindowState::Insert { index, tab };
                        });
                        ui.separator();

                        let iter = db
                            .iter()
                            .enumerate()
                            .filter(|(i, _)| ((tab * 32)..((tab + 1) * 32)).contains(i))
                            .chunks(16);

                        for chunk in iter.into_iter() {
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    for (_, desc) in chunk {
                                        let mut response = ui.button(&desc.name);
                                        if !desc.description.is_empty() {
                                            response =
                                                response.on_disabled_hover_text(&desc.description);
                                        }
                                        if response.clicked() {
                                            let indent = commands[index].indent;
                                            commands.insert(
                                                index,
                                                rpg::EventCommand {
                                                    code: desc.code,
                                                    indent,
                                                    parameters: vec![],
                                                    guid: rand::random(),
                                                },
                                            );
                                            if let CommandKind::Branch { end_code, .. } = desc.kind
                                            {
                                                commands.insert(
                                                    index + 1,
                                                    rpg::EventCommand {
                                                        code: 0,
                                                        indent: indent + 1,
                                                        parameters: vec![],
                                                        guid: rand::random(),
                                                    },
                                                );
                                                commands.insert(
                                                    index + 2,
                                                    rpg::EventCommand {
                                                        code: end_code,
                                                        indent,
                                                        parameters: vec![],
                                                        guid: rand::random(),
                                                    },
                                                );
                                            }

                                            self.window_state = WindowState::Edit { index };
                                        }
                                    }
                                });

                                ui.separator()
                            });
                        }
                    });
            }
            WindowState::None => {}
        }
        if !open {
            self.window_state = WindowState::None;
        }
    }
}
