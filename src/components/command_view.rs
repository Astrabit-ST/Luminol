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
        commands::{
            Command,
            CommandKind::{self, *},
            MOVE_SPEEDS,
        },
        rmxp_structs::rpg,
    },
    UpdateInfo,
};

const CONTROL_FLOW: Color32 = Color32::from_rgb(0, 102, 255);
const ERROR: Color32 = Color32::RED;
const NORMAL: Color32 = Color32::WHITE;
const COMMENT: Color32 = Color32::GREEN;
const SCRIPT: Color32 = Color32::YELLOW;
#[allow(missing_docs)]
pub const MOVE_ROUTE: Color32 = Color32::from_rgb(252, 140, 3);
const DATA: Color32 = Color32::from_rgb(252, 93, 93);
const AUDIO: Color32 = Color32::from_rgb(101, 252, 232);

/// An event command viewer.

pub struct CommandView<'co> {
    pub(crate) custom_id_source: &'co str,
    map_id: Option<i32>,
}

#[derive(Clone)]
pub(crate) struct Memory {
    pub selected_index: usize,
    pub move_route_modal: (i32, Option<rpg::MoveRoute>),
    pub map_id: Option<i32>,
}

impl<'co> CommandView<'co> {
    /// Create a new command viewer.
    pub fn new(custom_id_source: &'co str, map_id: Option<i32>) -> Self {
        Self {
            custom_id_source,
            map_id,
        }
    }

    /// Show the viewer.
    pub fn ui(mut self, ui: &mut egui::Ui, info: &'static UpdateInfo, commands: &'co mut Node) {
        let memory = ui.memory().data.get_temp(egui::Id::new(format!(
            "command_view_memory_{}",
            self.custom_id_source
        )));
        let mut memory = memory.unwrap_or(Memory {
            selected_index: 0,
            move_route_modal: (2, None),
            map_id: self.map_id,
        });

        ui.vertical(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(NORMAL);
            ui.visuals_mut().button_frame = false;

            self.render_command(ui, commands, &mut 0, &mut memory, info);
        });

        self.modals(ui, &mut memory, info);

        ui.memory().data.insert_temp(
            egui::Id::new(format!("command_view_memory_{}", self.custom_id_source)),
            memory,
        );
    }

    fn render_command(
        &mut self,
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        memory: &mut Memory,
        info: &'static UpdateInfo,
    ) {
        let Memory {
            selected_index,
            move_route_modal,
            map_id,
        } = memory;
        *index += 1;

        let Command { kind, .. } = &mut node.data;
        match kind {
            Insert => {
                ui.selectable_value(selected_index, *index, "@>");
            }
            Text { text } => {
                ui.selectable_value(selected_index, *index, format!("Show Text: {text}"));
            }
            TextExt { text } => {
                //
                ui.label(format!("          :  {text}"));
            }
            Choices { choices, .. } => {
                let text = format!("Show Choices [\n  {}\n]", choices.join(",\n  "));

                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(text).color(CONTROL_FLOW),
                );
            }
            When { choice } => {
                // TODO: Display choice text
                self.collapsible(format!("When {choice}"), ui, node, index, memory, info);
            }
            WhenCancel => {
                self.collapsible("When Cancel".to_string(), ui, node, index, memory, info);
            }
            ChoiceEnd => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Choice End").color(CONTROL_FLOW),
                );
            }
            ExitEvent => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Exit Event Processing").color(CONTROL_FLOW),
                );
            }
            CallCommonEvent { event } => {
                let common_events = info.data_cache.common_events();
                let common_events = common_events.as_ref().unwrap();

                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!(
                        "Call Common Event [{}]",
                        common_events[*event].name
                    ))
                    .color(CONTROL_FLOW),
                );
            }
            Conditional { kind } => {
                use crate::data::commands::{ConditionalKind::*, ConditionalOperator::*};

                let text = {
                    let system = info.data_cache.system();
                    let system = system.as_ref().unwrap();

                    format!(
                        "Conditional Branch: {}",
                        match kind {
                            Switch { id, state } => {
                                format!(
                                    "[{id}: {}] is {}",
                                    system.switches[*id],
                                    match *state {
                                        true => "ON",
                                        false => "OFF",
                                    }
                                )
                            }
                            Variable {
                                id,
                                const_value,
                                variable_value,
                                operator,
                            } => {
                                format!(
                                    "[{id}: {}] is {} {}",
                                    system.variables[*id],
                                    match operator {
                                        Equal => "==",
                                        GreaterEqual => ">=",
                                        LessEqual => "<=",
                                        Greater => ">",
                                        Less => "<",
                                        NotEqual => "!=",
                                    },
                                    {
                                        if let Some(val) = const_value {
                                            val.to_string()
                                        } else {
                                            format!(
                                                "[{}: {}]",
                                                variable_value.unwrap(),
                                                system.variables[variable_value.unwrap()]
                                            )
                                        }
                                    }
                                )
                            }
                            SelfSwitch { char, state } => {
                                format!(
                                    "Self Switch '{char}' is {}",
                                    match *state {
                                        true => "ON",
                                        false => "OFF",
                                    }
                                )
                            }
                            Item { id } => {
                                let items = info.data_cache.items();
                                let items = items.as_ref().unwrap();

                                format!("Has item {}", items[*id].name)
                            }
                            Script { text } => {
                                text.clone()
                            }
                            _ => "".to_string(),
                        }
                    )
                };

                self.collapsible(text, ui, node, index, memory, info);
            }
            Else => {
                self.collapsible("Else".to_string(), ui, node, index, memory, info);
            }
            BranchEnd => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new("Branch End").color(CONTROL_FLOW),
                );
            }
            Loop => {
                self.collapsible("Loop".to_string(), ui, node, index, memory, info);
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
                    RichText::new(format!("Comment: {text}")).color(COMMENT),
                );
            }
            CommentExt { text } => {
                ui.selectable_value(
                    selected_index,
                    *index,
                    //                               "Comment: {}"
                    RichText::new(format!("       : {text}")).color(COMMENT),
                );
            }
            CommandKind::Wait { time } => {
                ui.selectable_value(selected_index, *index, format!("Wait {} frames", *time * 2));
            }
            CommandKind::Script { text } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("Script: {text}")).color(SCRIPT),
                );
            }
            ScriptExt { text } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("      : {text}")).color(SCRIPT),
                );
            }
            CommandKind::Invalid { code, parameters } => {
                //
                ui.selectable_value(
                    selected_index,
                    *index,
                    RichText::new(format!("Invalid Command {code} 🔥")).color(ERROR),
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
            // RMXP provides in editor-commands to display the move route commands.
            // IMHO they are wasteful and we don't need them.
            // Instead, we render the move route directly.
            MoveRoute { target, route } => {
                let target_name = match *target {
                    -1 => "Player".to_string(),
                    0 => "This event".to_string(),
                    _ => map_id
                        .map(|id| {
                            let map = info.data_cache.get_map(id);
                            map.events[*target as usize].name.clone()
                        })
                        .unwrap_or(format!("Event {target}")),
                };

                let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    egui::Id::new(format!(
                        "{}_{index}_collapsible_command",
                        self.custom_id_source
                    )),
                    true,
                );

                // If the header is closed
                if header.openness(ui.ctx()) <= 0. {
                    // Update the index to the length of the list
                    *index += route.list.len();
                }

                header
                    .show_header(ui, |ui| {
                        let response = ui.selectable_value(
                            selected_index,
                            *index,
                            RichText::new(format!("Set Move Route: {target_name}"))
                                .color(MOVE_ROUTE),
                        );
                        if map_id.is_some() {
                            response.context_menu(|ui| {
                                if ui.button("Preview Move Route").clicked() {
                                    move_route_modal.1 = Some(route.clone());
                                };
                            });
                        }
                        *index += 1;
                    })
                    .body(|ui| {
                        crate::components::move_display::MoveDisplay::new(route).ui(
                            ui,
                            selected_index,
                            index,
                            info,
                        );
                    });
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
            CommandKind::PlaySE { file } => {
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
                    "Scroll Map {} {distance} tiles, speed {speed}: {}",
                    match *direction {
                        2 => "Down",
                        4 => "Left",
                        6 => "Right",
                        8 => "Up",
                        _ => unreachable!(),
                    },
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
            self.render_command(ui, node, index, memory, info);
        });
    }

    fn collapsible(
        &mut self,
        text: String,
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        memory: &mut Memory,

        info: &'static UpdateInfo,
    ) -> CollapsingResponse<()> {
        ui.vertical(|ui| {
            let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                egui::Id::new(format!(
                    "{}_{index}_collapsible_command",
                    self.custom_id_source
                )),
                true,
            );
            let openness = header.openness(ui.ctx());

            let ret_response = header
                .show_header(ui, |ui| {
                    ui.selectable_value(
                        &mut memory.selected_index,
                        *index,
                        RichText::new(text).color(CONTROL_FLOW),
                    )
                })
                .body(|ui| {
                    node.branch(Branch::Right, |node| {
                        self.render_command(ui, node, index, memory, info)
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
