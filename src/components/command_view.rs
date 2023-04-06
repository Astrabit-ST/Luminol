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

use std::hash::Hash;

pub use crate::prelude::*;
use command_lib::{CommandKind, Parameter, ParameterKind};
use itertools::Itertools;
use std::collections::HashMap;

pub struct CommandView {
    selected_index: usize,
    window_state: WindowState,
    id: egui::Id,
    modals: HashMap<u64, bool>, // todo find a better way to handle modals
}

enum WindowState {
    Insert { index: usize, tab: usize },
    Edit { index: usize },
    None,
}

impl Default for CommandView {
    fn default() -> Self {
        Self {
            selected_index: 0,
            window_state: WindowState::None,
            id: egui::Id::new("command_view"),
            modals: HashMap::new(),
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
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            ..Default::default()
        }
    }

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
            self.command_ui(ui, db, i, &mut iter);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected_index = self.selected_index.saturating_sub(1);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.selected_index = (self.selected_index + 1).min(commands.len() - 1);
        }

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
                                        let theme =
                                            syntax_highlighting::CodeTheme::from_memory(ui.ctx());
                                        let mut layout_job = syntax_highlighting::highlight(
                                            ui.ctx(),
                                            &theme,
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

    #[allow(clippy::only_used_in_recursion)]
    fn parameter_ui(
        &mut self,
        ui: &mut egui::Ui,
        parameter: &Parameter,
        command: &mut rpg::EventCommand,
        info: &'static UpdateInfo,
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
                            self.parameter_ui(ui, parameter, command, info);
                        });
                    });
                }
            }
            Parameter::Group { parameters, .. } => {
                ui.group(|ui| {
                    for parameter in parameters.iter() {
                        self.parameter_ui(ui, parameter, command, info);
                    }
                });
            }
            Parameter::Single {
                index,
                description,
                name,
                kind,
                guid,
            } => {
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
                        switch::Modal::new(self.id.with(guid)).button(ui, state, &mut data, info);
                        *value.into_integer() = data as i32;
                    }
                    ParameterKind::Variable => {
                        let mut data = *value.into_integer_with(1) as usize;
                        let state = self.modals.entry(*guid).or_insert(false);
                        variable::Modal::new(self.id.with(guid)).button(ui, state, &mut data, info);
                    }

                    ParameterKind::String => {
                        ui.text_edit_singleline(value.into_string());
                    }
                }
            }
            Parameter::Dummy => {}
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
                            color_text!(format!("[{}]: {}", desc.code, desc.name), Color32::BLUE),
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
                        let theme = syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                        ui.selectable_value(
                            &mut self.selected_index,
                            index,
                            syntax_highlighting::highlight(ui.ctx(), &theme, &str, "rb"),
                        )
                    } else {
                        ui.selectable_value(&mut self.selected_index, index, str)
                    }
                })
                .inner
            }
            CommandKind::Single(_) => {
                //
                ui.selectable_value(
                    &mut self.selected_index,
                    index,
                    format!("[{}]: {}", desc.code, desc.name),
                )
            }
        };

        let response = response.context_menu(|ui| {
            if ui.button("Insert above").clicked() {}
            if ui.button("Delete").clicked() {}
            if ui.button("Insert below").clicked() {}
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
