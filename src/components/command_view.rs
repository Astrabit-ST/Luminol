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
            MoveCommand::{self, *},
            MOVE_FREQS, MOVE_SPEEDS,
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
const MOVE_ROUTE: Color32 = Color32::from_rgb(252, 140, 3);
const DATA: Color32 = Color32::from_rgb(252, 93, 93);
const AUDIO: Color32 = Color32::from_rgb(101, 252, 232);

/// An event command viewer.

pub struct CommandView<'co> {
    commands: &'co mut Node,
    custom_id_source: &'co str,
    map_id: Option<i32>,
}

#[derive(Clone)]
struct Memory {
    selected_index: usize,
    move_route_modal: (bool, i32, Option<rpg::MoveRoute>),
    map_id: Option<i32>,
}

impl<'co> CommandView<'co> {
    /// Create a new command viewer.
    pub fn new(commands: &'co mut Node, custom_id_source: &'co str, map_id: Option<i32>) -> Self {
        Self {
            commands,
            custom_id_source,
            map_id,
        }
    }

    /// Show the viewer.
    pub fn ui(self, ui: &mut egui::Ui, info: &'static UpdateInfo) {
        ui.vertical(|ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
            ui.visuals_mut().override_text_color = Some(NORMAL);
            ui.visuals_mut().button_frame = false;

            let memory = ui.memory().data.get_temp(egui::Id::new(format!(
                "command_view_memory_{}",
                self.custom_id_source
            )));
            let mut memory = memory.unwrap_or(Memory {
                selected_index: 0,
                move_route_modal: (false, 2, None),
                map_id: self.map_id,
            });

            // TODO: Find a better way of doing this.
            let mut win_open = memory.move_route_modal.0;
            if memory.move_route_modal.0 {
                egui::Window::new("Preview Move Route")
                    .open(&mut win_open)
                    .show(ui.ctx(), |ui| {
                        let mut map = info.data_cache.get_map(self.map_id.unwrap());

                        ui.label("Starting Direction");
                        ui.radio_value(&mut memory.move_route_modal.1, 2, "Up");
                        ui.radio_value(&mut memory.move_route_modal.1, 8, "Down");
                        ui.radio_value(&mut memory.move_route_modal.1, 4, "Left");
                        ui.radio_value(&mut memory.move_route_modal.1, 6, "Right");

                        ui.horizontal(|ui| {
                            if ui.button("Ok").clicked() {
                                memory.move_route_modal.0 = false;
                                map.preview_move_route = Some((
                                    memory.move_route_modal.1,
                                    memory.move_route_modal.2.take().unwrap(),
                                ))
                            }

                            if ui.button("Apply").clicked() {
                                map.preview_move_route = Some((
                                    memory.move_route_modal.1,
                                    memory.move_route_modal.2.clone().unwrap(),
                                ))
                            }

                            if ui.button("Cancel").clicked() {
                                memory.move_route_modal.0 = false;
                            }
                        });
                    });
            }
            memory.move_route_modal.0 &= win_open;

            Self::render_command(
                ui,
                self.commands,
                &mut 0,
                &mut memory,
                self.custom_id_source,
                info,
            );

            ui.memory().data.insert_temp(
                egui::Id::new(format!("command_view_memory_{}", self.custom_id_source)),
                memory,
            );
        });
    }

    fn render_command(
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        memory: &mut Memory,
        custom_id_source: &'co str,
        info: &'static UpdateInfo,
    ) {
        let Memory {
            selected_index,
            move_route_modal,
            map_id,
            ..
        } = memory;
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
                Self::collapsible(
                    format!("When {choice}"),
                    ui,
                    node,
                    index,
                    memory,
                    custom_id_source,
                    info,
                );
            }
            WhenCancel => {
                Self::collapsible(
                    "When Cancel".to_string(),
                    ui,
                    node,
                    index,
                    memory,
                    custom_id_source,
                    info,
                );
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
            Conditional { .. } => {
                Self::collapsible(
                    "Conditional Branch".to_string(),
                    ui,
                    node,
                    index,
                    memory,
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
                    memory,
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
                    memory,
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
            CommandKind::Wait { time } => {
                ui.selectable_value(selected_index, *index, format!("Wait {} frames", *time * 2));
            }
            CommandKind::Script { text } => {
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
            CommandKind::Invalid { code, parameters } => {
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
                        "{}_{}_collapsible_command",
                        custom_id_source, index
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
                                move_route_modal.0 = true;
                                move_route_modal.2 = Some(route.clone());
                            };
                        });
                    }
                        *index += 1;
                    })
                    .body(|ui| {
                        let system = info.data_cache.system();
                        let system = system.as_ref().unwrap();

                        for command in route.list.iter() {
                            let label = match command {
                                Down => "Move Down".to_string(),
                                Left => "Move Left".to_string(),
                                Right => "Move Right".to_string(),
                                Up => "Move Up".to_string(),
                                LowerLeft => "Move Lower Left".to_string(),
                                LowerRight => "Move Lower Right".to_string(),
                                UpperLeft => "Move Upper Left".to_string(),
                                UpperRight => "Move Upper Right".to_string(),
                                Random => "Move Random".to_string(),
                                MoveTowards => "Move Towards Player".to_string(),
                                MoveAway => "Move Away from Player".to_string(),
                                Forward => "Move Forwards".to_string(),
                                Backwards => "Move Backwards".to_string(),
                                Jump { x_plus, y_plus } => format!("Jump ({x_plus},{y_plus})px"),
                                MoveCommand::Wait { time } => format!("Wait {time} frames"),
                                TurnDown => "Turn Down".to_string(),
                                TurnLeft => "Turn Left".to_string(),
                                TurnRight => "Turn Right".to_string(),
                                TurnUp => "Turn Up".to_string(),
                                TurnRight90 => "Turn Right 90deg".to_string(),
                                TurnLeft90 => "Turn Left 90deg".to_string(),
                                Turn180 => "Turn 180deg".to_string(),
                                TurnRightOrLeft => "Turn Right or Left".to_string(),
                                TurnRandom => "Turn Randomly".to_string(),
                                TurnTowardsPlayer => "Turn Towards Player".to_string(),
                                TurnAwayFromPlayer => "Turn Away from Player".to_string(),
                                SwitchON { switch_id } => {
                                    format!("Switch [{switch_id}: {}] ON", system.switches[*switch_id])
                                }
                                SwitchOFF { switch_id } => {
                                    format!("Switch [{switch_id}: {}] OFF", system.switches[*switch_id])
                                }
                                ChangeSpeed { speed } => {
                                    format!("Set Speed to {speed}: {}", MOVE_SPEEDS[*speed - 1])
                                }
                                ChangeFreq { freq } => {
                                    format!("Set Frequency to {freq}: {}", MOVE_FREQS[*freq - 1])
                                }
                                MoveON => "Set Move Animation ON".to_string(),
                                MoveOFF => "Set Move Animation OFF".to_string(),
                                StopON => "Set Stop Animation ON".to_string(),
                                StopOFF => "Set Stop Animation OFF".to_string(),
                                DirFixON => "Set Direction Fix ON".to_string(),
                                DirFixOFF => "Set Direction Fix OFF".to_string(),
                                ThroughON => "Set Through ON".to_string(),
                                ThroughOFF => "Set Through OFF".to_string(),
                                AlwaysTopON => "Set Always on Top ON".to_string(),
                                AlwaysTopOFF => "Set Always on Top OFF".to_string(),
                                ChangeGraphic {
                                    character_name,
                                    character_hue,
                                    direction,
                                    pattern
                                } => format!("Set graphic to '{character_name}' with hue: {character_hue}, direction: {direction}, pattern: {pattern}"),
                                ChangeOpacity { opacity } => format!("Set opacity to {opacity}"),
                                ChangeBlend { blend } => format!(
                                    "Set blend type to {}",
                                    match blend {
                                        0 => "Normal",
                                        1 => "Additive",
                                        2 => "Subtractive",
                                        _ => unreachable!(),
                                    }
                                ),
                                MoveCommand::PlaySE { file } => format!(
                                    "Play SE \"{}\", vol: {}, pitch: {}",
                                    file.name, file.volume, file.pitch
                                ),
                                MoveCommand::Script { text } => format!("Script: {text}"),

                                Break => "".to_string(),
                                MoveCommand::Invalid { code, parameters } => {
                                    format!("Invalid command {code} {:#?}", parameters)
                                }
                            };
                            ui.selectable_value(
                                selected_index,
                                *index,
                                RichText::new(format!("$> {label}")).color(MOVE_ROUTE),
                            );
                            *index += 1;
                        }
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
            Self::render_command(ui, node, index, memory, custom_id_source, info);
        });
    }

    fn collapsible(
        text: String,
        ui: &mut egui::Ui,
        node: &mut Node,
        index: &mut usize,
        memory: &mut Memory,
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
                        &mut memory.selected_index,
                        *index,
                        RichText::new(text).color(CONTROL_FLOW),
                    )
                })
                .body(|ui| {
                    node.branch(Branch::Right, |node| {
                        Self::render_command(ui, node, index, memory, custom_id_source, info)
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
