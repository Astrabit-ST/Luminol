// Copyright (C) 2023 Lily Lyons
//
// This file is part of Luminol
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
use command_lib::{CommandDescription, CommandKind, Index, Parameter};

use ui_example::UiExample;

use crate::{fl, prelude::*};

pub mod parameter_ui;
pub mod ui_example;

pub struct CommandGeneratorWindow {
    commands: Vec<CommandDescription>,
    ui_examples: Vec<UiExample>,
}

impl Default for CommandGeneratorWindow {
    fn default() -> Self {
        Self {
            commands: command_db!().user.clone(),
            ui_examples: vec![],
        }
    }
}

impl CommandGeneratorWindow {
    /// Updates all of the parameter indexes, if they are assumed
    fn recalculate_parameter_index(parameter: &mut Parameter, passed_index: &mut u8) {
        match parameter {
            Parameter::Group { parameters, .. } => {
                for parameter in parameters.iter_mut() {
                    Self::recalculate_parameter_index(parameter, passed_index);
                }
            }
            Parameter::Selection {
                index, parameters, ..
            } => {
                if let Index::Assumed(ref mut assumed_index) = index {
                    *assumed_index = *passed_index;
                }

                // Add one for ourselves
                *passed_index += 1;

                // The intent here is to make each selection have the same starting index
                // The max index is taken here
                *passed_index = parameters
                    .iter_mut()
                    .map(|(_, parameter)| {
                        let mut passed_index = *passed_index;
                        Self::recalculate_parameter_index(parameter, &mut passed_index);
                        passed_index
                    })
                    .max()
                    .unwrap_or(0)
            }
            Parameter::Single { index, .. } => {
                if let Index::Assumed(ref mut assumed_index) = index {
                    *assumed_index = *passed_index;
                }

                *passed_index += 1;
            }
            _ => {}
        }
    }
}

impl window::Window for CommandGeneratorWindow {
    fn name(&self) -> String {
        fl!("window_commandgen_title_label")
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Luminol Command Maker")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mut del_index = None;
                for (idx, command) in self.commands.iter_mut().enumerate() {
                    ui.push_id(command.guid, |ui| {
                        let header =
                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                format!("command_{idx}").into(),
                                false,
                            );
                        header
                            .show_header(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}:", fl!("name")));
                                    ui.text_edit_singleline(&mut command.name);

                                    ui.label(format!("{}:", fl!("code")));
                                    ui.add(egui::DragValue::new(&mut command.code));
                                });

                                if ui
                                    .button(
                                        egui::RichText::new("-")
                                            .monospace()
                                            .color(egui::Color32::RED),
                                    )
                                    .clicked()
                                {
                                    del_index = Some(idx)
                                }
                            })
                            .body(|ui| {
                                ui.label(format!("{}:", fl!("description")));
                                ui.text_edit_multiline(&mut command.description)
                                    .on_hover_text(fl!("window_commandgen_desc_label"));
                                ui.label(fl!("window_commandgen_lumi_label"));
                                ui.text_edit_multiline(&mut command.lumi_text)
                                    .on_hover_text(fl!("window_commandgen_lumi_onhover_label"));

                                ui.separator();

                                ui.label(fl!("type"));
                                ui.horizontal(|ui| {
                                    ui.menu_button(
                                        format!("{} ⏷", <&str>::from(&command.kind)),
                                        |ui| {
                                            for kind in CommandKind::iter() {
                                                let text = <&str>::from(&kind);
                                                ui.selectable_value(&mut command.kind, kind, text);
                                            }
                                        },
                                    );
                                    match command.kind {
                                        CommandKind::Multi {
                                            ref mut code,
                                            ref mut highlight,
                                        } => {
                                            ui.label(fl!("window_commandgen_contcode_label"))
                                                .on_hover_text(fl!(
                                                    "window_commandgen_contcode_onhover_label"
                                                ));
                                            ui.add(egui::DragValue::new(code));
                                            ui.checkbox(
                                                highlight,
                                                fl!("window_commandgen_syntax_highlighting_cb"),
                                            );
                                        }
                                        CommandKind::Branch {
                                            ref mut end_code, ..
                                        } => {
                                            ui.label(fl!("window_commandgen_endcode_label"))
                                                .on_hover_text(fl!(
                                                    "window_commandgen_endcode_onhover_label"
                                                ));
                                            ui.add(egui::DragValue::new(end_code));
                                        }
                                        _ => {}
                                    }
                                });

                                ui.checkbox(&mut command.hidden, fl!("window_commandgen_him_cb"));

                                ui.separator();

                                if let CommandKind::Single(ref mut parameters)
                                | CommandKind::Branch {
                                    ref mut parameters, ..
                                } = command.kind
                                {
                                    ui.collapsing(fl!("parameters"), |ui| {
                                        let mut del_idx = None;

                                        let mut passed_index = 0;
                                        for (ele, parameter) in parameters.iter_mut().enumerate() {
                                            parameter_ui::parameter_ui(
                                                ui,
                                                parameter,
                                                (ele, &mut del_idx),
                                            );

                                            Self::recalculate_parameter_index(
                                                parameter,
                                                &mut passed_index,
                                            );
                                        }

                                        if let Some(idx) = del_idx {
                                            parameters.remove(idx);
                                        }

                                        if ui
                                            .button(
                                                egui::RichText::new("+")
                                                    .monospace()
                                                    .color(egui::Color32::GREEN),
                                            )
                                            .clicked()
                                        {
                                            parameters.push(Parameter::default());
                                        }
                                    });
                                }
                            });

                        if command.parameter_count() > 0
                            && ui.button(fl!("window_commandgen_preview_btn")).clicked()
                        {
                            self.ui_examples.push(UiExample::new(command));
                        }

                        ui.separator();
                    });
                }

                if let Some(idx) = del_index {
                    self.commands.remove(idx);
                }

                ui.horizontal(|ui| {
                    if ui
                        .button(
                            egui::RichText::new("+")
                                .monospace()
                                .color(egui::Color32::GREEN),
                        )
                        .clicked()
                    {
                        self.commands.push(CommandDescription::default());
                    }

                    if ui.button(fl!("save")).clicked() {
                        command_db!().user = self.commands.clone();
                    }
                });
            });

            self.ui_examples.retain_mut(|e| e.update(ctx));
        });
    }
}
