// Copyright (C) 2022 Lily Lyons
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

use egui::{CollapsingResponse, Color32, RichText};

use crate::data::commands::{Command, CommandKind::*};

const CONTROL_FLOW: Color32 = Color32::BLUE;
const ERROR: Color32 = Color32::RED;
const NORMAL: Color32 = Color32::WHITE;

/// An event command viewer.

pub struct CommandView<'co> {
    commands: &'co mut Vec<Command>,
}

impl<'co> CommandView<'co> {
    /// Create a new command viewer.
    pub fn new(commands: &'co mut Vec<Command>) -> Self {
        Self { commands }
    }

    /// Show the viewer.
    pub fn ui(self, ui: &mut egui::Ui) {
        egui::ScrollArea::both()
            .auto_shrink([false, false])
            .max_height(500.)
            .show(ui, |ui| {
                let mut iter = self.commands.iter_mut().enumerate();
                let mut selected_index = ui
                    .memory()
                    .data
                    .get_temp(egui::Id::new("command_view_selected_index"));
                let mut selected_index = *selected_index.get_or_insert(0);

                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                ui.visuals_mut().override_text_color = Some(NORMAL);
                ui.visuals_mut().button_frame = false;

                while let Some((ele, cmd)) = iter.next() {
                    Self::render_command(ui, ele, cmd, &mut iter, &mut selected_index);

                    if cmd.kind.is_break() {
                        continue;
                    }
                }

                if ui.input().key_pressed(egui::Key::ArrowUp) {
                    selected_index =
                        (selected_index + self.commands.len() - 1) % self.commands.len()
                }
                if ui.input().key_pressed(egui::Key::ArrowDown) {
                    selected_index =
                        (selected_index + self.commands.len() + 1) % self.commands.len()
                }

                ui.memory()
                    .data
                    .insert_temp(egui::Id::new("command_view_selected_index"), selected_index);
            });
    }

    fn render_command<'iter>(
        ui: &mut egui::Ui,
        ele: usize,
        cmd: &mut Command,
        iter: &mut impl Iterator<Item = (usize, &'iter mut Command)>,
        selected_index: &mut usize,
    ) {
        const INDENT_STR: &str = "   ";

        let Command { indent, kind } = cmd;

        if kind.is_break() {
            ui.selectable_value(
                selected_index,
                ele,
                format!("{:0>3}>{}@>", ele, INDENT_STR.repeat(*indent)),
            );
        }

        let response = ui
            .horizontal(|ui| {
                ui.label(format!("{:0>3}>{}", ele, INDENT_STR.repeat(*indent)));

                match kind {
                    Break => {
                        ui.selectable_value(
                            selected_index,
                            ele,
                            RichText::new("Branch End").color(CONTROL_FLOW),
                        );
                    }
                    Text { text } => {
                        ui.selectable_value(selected_index, ele, format!("Show Text: {}", text));
                    }
                    TextExt { text } => {
                        ui.label(format!("         :  {}", text));
                    }
                    Conditional { .. } => {
                        Self::collapsible(
                            "Conditional Branch".to_string(),
                            ui,
                            ele,
                            iter,
                            selected_index,
                        );
                    }
                    Loop => {
                        Self::collapsible("Loop".to_string(), ui, ele, iter, selected_index);
                    }
                    Invalid { code } => {
                        if ui
                            .selectable_value(
                                selected_index,
                                ele,
                                RichText::new(format!("Invalid Command {}", code)).color(ERROR),
                            )
                            .double_clicked()
                        {};
                    }
                    _ => {
                        ui.selectable_value(selected_index, ele, "???");
                    }
                };
            })
            .response;

        if *selected_index == ele
            && (ui.input().key_pressed(egui::Key::ArrowUp)
                || ui.input().key_pressed(egui::Key::ArrowDown))
        {
            response.scroll_to_me(None);
        }
    }

    fn collapsible<'iter>(
        text: String,
        ui: &mut egui::Ui,
        ele: usize,
        iter: &mut impl Iterator<Item = (usize, &'iter mut Command)>,
        selected_index: &mut usize,
    ) -> CollapsingResponse<()> {
        let response = egui::CollapsingHeader::new(RichText::new(text).color(CONTROL_FLOW))
            .default_open(true)
            .id_source(format!("{}_collapsible_command", ele))
            .show(ui, |ui| {
                while let Some((ele, cmd)) = iter.next() {
                    Self::render_command(ui, ele, cmd, iter, selected_index);

                    if iter
                        .peekable()
                        .peek()
                        .is_some_and(|(_, cmd)| !cmd.kind.is_break())
                    {
                        break;
                    }
                }
            });
        if response.fully_closed() {
            Self::break_until(iter)
        }
        response
    }

    fn break_until<'iter>(iter: &mut impl Iterator<Item = (usize, &'iter mut Command)>) {
        while let Some((_, cmd)) = iter.next() {
            if cmd.kind.is_break() {
                break;
            }

            match cmd.kind {
                Loop | Conditional { .. } => Self::break_until(iter),
                _ => (),
            }
        }
    }
}
