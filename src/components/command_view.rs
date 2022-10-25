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

use crate::{
    data::{
        command_tree::{Branch, Node},
        commands::{Command, CommandKind::*, MOVE_SPEEDS},
    },
    UpdateInfo,
};

const CONTROL_FLOW: Color32 = Color32::from_rgb(0, 102, 255);
const ERROR: Color32 = Color32::RED;
const NORMAL: Color32 = Color32::WHITE;
const COMMENT: Color32 = Color32::GREEN;
const SCRIPT: Color32 = Color32::YELLOW;
const MOVE_ROUTE: Color32 = Color32::from_rgb(252, 140, 3);
const DATA: Color32 = Color32::from_rgb(252, 93, 93);
const AUDIO: Color32 = Color32::from_rgb(101, 252, 232);

/// An event command viewer.

pub struct CommandView<'co> {
    commands: &'co mut Node,
    custom_id_source: &'co str,
}

impl<'co> CommandView<'co> {
    /// Create a new command viewer.
    pub fn new(commands: &'co mut Node, custom_id_source: &'co str) -> Self {
        Self {
            commands,
            custom_id_source,
        }
    }

    /// Show the viewer.
    pub fn ui(self, ui: &mut egui::Ui, info: &'static UpdateInfo) {
        ui.vertical(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(NORMAL);
            ui.visuals_mut().button_frame = false;

            let mut selected_index = ui
                .memory()
                .data
                .get_temp(egui::Id::new("command_view_selected_index"));
            let mut selected_index = *selected_index.get_or_insert(1000);

            Self::render_command(
                ui,
                self.commands,
                &mut 0,
                &mut selected_index,
                self.custom_id_source,
                info,
            );

            ui.memory()
                .data
                .insert_temp(egui::Id::new("command_view_selected_index"), selected_index);
        });
    }

    fn render_command(
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        selected_index: &mut usize,
        custom_id_source: &'co str,
        info: &'static UpdateInfo,
    ) {
        *index += 1;

        let Command { kind, .. } = &mut node.data;
        match kind {
            Insert => {
                ui.selectable_value(selected_index, *index, "@>");
            }
            Text { text } => {
                ui.selectable_value(selected_index, *index, format!("Show Text: {}", text));
            }
            TextExt { text } => {
                //
                ui.label(format!("          :  {}", text));
            }
            Conditional { .. } => {
                Self::collapsible(
                    "Conditional Branch".to_string(),
                    ui,
                    node,
                    index,
                    selected_index,
                    custom_id_source,
                    info,
                );
            }
            Else => {
                Self::collapsible(
                    "Else".to_string(),
                    ui,
                    node,
                    index,
                    selected_index,
                    custom_id_source,
                    info,
                );
            }
            BranchEnd => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Branch End").color(CONTROL_FLOW),
                );
            }
            Loop => {
                //
                Self::collapsible(
                    "Loop".to_string(),
                    ui,
                    node,
                    index,
                    selected_index,
                    custom_id_source,
                    info,
                );
            }
            BreakLoop => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Break Loop").color(CONTROL_FLOW),
                );
            }
            LoopEnd => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Loop End").color(CONTROL_FLOW),
                );
            }
            Comment { text } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("Comment: {}", text)).color(COMMENT),
                );
            }
            CommentExt { text } => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    //                               "Comment: {}"
                    RichText::new(format!("       : {}", text)).color(COMMENT),
                );
            }
            Wait { time } => {
                ui.selectable_value(selected_index, *index, format!("Wait {} frames", *time * 2));
            }
            Script { text } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("Script: {}", text)).color(SCRIPT),
                );
            }
            ScriptExt { text } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("      : {}", text)).color(SCRIPT),
                );
            }
            Invalid { code, parameters } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("Invalid Command {} 🔥", code)).color(ERROR),
                )
                .on_hover_text(
                    RichText::new("This happens when Luminol does not recognize a command ID.")
                        .color(ERROR),
                );
                ui.colored_label(ERROR, format!("Parameters: \n{:#?}", parameters));
            }
            MoveDisplay => {}
            WaitMoveRoute => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Wait for Move's completion").color(MOVE_ROUTE),
                );
            }
            ControlSwitches { range, state } => {
                let str = format!(
                    "Set Switch{} = {}",
                    if range.end() == range.start() {
                        let system = info.data_cache.system();
                        let system = system.as_ref().unwrap();

                        format!(" [{}: {}]", range.start(), system.switches[*range.start()])
                    } else {
                        format!("es [{}..{}]", range.start(), range.end())
                    },
                    match state {
                        true => "ON",
                        false => "OFF",
                    }
                );

                ui.selectable_value(selected_index, *index, RichText::new(str).color(DATA));
            }
            PlaySE { file } => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!(
                        "Play SE \"{}\", vol: {}, pitch: {}",
                        file.name, file.volume, file.pitch
                    ))
                    .color(AUDIO),
                );
            }
            ScrollScreen {
                direction,
                distance,
                speed,
            } => {
                let str = format!(
                    "Scroll Map {} {} tiles, speed {}: {}",
                    match *direction {
                        2 => "Down",
                        4 => "Left",
                        6 => "Right",
                        8 => "Up",
                        _ => unreachable!(),
                    },
                    distance,
                    speed,
                    MOVE_SPEEDS[*speed - 1]
                );

                ui.selectable_value(selected_index, *index, RichText::new(str).color(MOVE_ROUTE));
            }
            _ => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("{:#?} ???", kind)).color(SCRIPT),
                )
                .on_hover_text(
                    RichText::new(
                        "Luminol recognizes this command, but there is no code to render it.",
                    )
                    .color(ERROR),
                );
            }
        };

        node.branch(Branch::Left, |node| {
            Self::render_command(ui, node, index, selected_index, custom_id_source, info);
        });
    }

    fn collapsible(
        text: String,
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        selected_index: &mut usize,
        custom_id_source: &'co str,
        info: &'static UpdateInfo,
    ) -> CollapsingResponse<()> {
        ui.vertical(|ui| {
            let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                egui::Id::new(format!(
                    "{}_{}_collapsible_command",
                    custom_id_source, index
                )),
                true,
            );
            let openness = header.openness(ui.ctx());

            let ret_response = header
                .show_header(ui, |ui| {
                    ui.selectable_value(
                        selected_index,
                        *index,
                        RichText::new(text).color(CONTROL_FLOW),
                    )
                })
                .body(|ui| {
                    node.branch(Branch::Right, |node| {
                        Self::render_command(
                            ui,
                            node,
                            index,
                            selected_index,
                            custom_id_source,
                            info,
                        )
                    })
                });

            let response = if let Some(ret_response_2) = ret_response.2 {
                egui::collapsing_header::CollapsingResponse {
                    header_response: ret_response.0,
                    body_response: Some(ret_response_2.response),
                    body_returned: Some(()),
                    openness,
                }
            } else {
                CollapsingResponse {
                    header_response: ret_response.0,
                    body_response: None,
                    body_returned: None,
                    openness,
                }
            };

            if response.fully_closed() {
                Self::break_until(node)
            }
            response
        })
        .inner
    }

    fn break_until(_node: &mut Node) {}
}
