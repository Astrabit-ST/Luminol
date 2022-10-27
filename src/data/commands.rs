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

use std::ops::RangeInclusive;

use serde::{Deserialize, Serialize};

use super::rmxp_structs::intermediate;
#[allow(unused_imports)]
use super::{rgss_structs::*, rmxp_structs::rpg};
use enum_as_inner::EnumAsInner;

#[derive(Debug, Deserialize, Serialize, Clone, EnumAsInner, PartialEq)]
#[allow(missing_docs)]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(rpg::AudioFile),
    Float(f32),
    MoveRoute(rpg::MoveRoute),
    MoveCommand(MoveCommand),
    Array(Vec<String>),
    TrueClass(bool),
    FalseClass(bool),
}

impl From<String> for ParameterType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

// FIXME: NOT ALL OF THESE ARE KNOWN

/// An enum representing event commands.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(from = "intermediate::EventCommand")]
#[serde(into = "intermediate::EventCommand")]
pub struct Command {
    /// Command indent
    pub indent: usize,
    /// Type of command
    pub kind: CommandKind,
}

#[derive(Debug, Clone, EnumAsInner, PartialEq)]
#[allow(missing_docs)]
pub enum CommandKind {
    /// Show text, id 101
    ///
    /// Fields: (text: [`String`])
    Text { text: String },
    /// Extended text, id 401
    /// Represents the next line of text
    ///
    /// Fields: (text: [`String`])
    TextExt { text: String },
    /// Show Choices, id 102
    ///
    /// Fields: (choices: [`Vec<String>`], choice_type: [`i32`])
    Choices {
        choices: Vec<String>,
        cancel_type: i32,
    },
    /// When, id 402
    ///
    /// Fields (???: [`i32`])
    When { choice: i32 },
    /// When Cancel, id 403
    WhenCancel,
    /// Choice end, id 404
    ChoiceEnd,
    /// Conditional Branch (If statement), id 111
    ///
    /// Fields (kind: [`i32`], params: [`Vec<ParameterType>`])
    Conditional {
        kind: i32,
        params: Vec<ParameterType>,
    },
    /// Else, id 411
    Else,
    /// Branch end, id 412
    BranchEnd,
    /// Loop, id 112
    Loop,
    /// Loop end, id 413
    LoopEnd,
    /// Comment, id 108
    Comment { text: String },
    /// CommentExt, id 408
    CommentExt { text: String },
    /// Wait, id 106
    Wait { time: i32 },

    /// Break Loop, id 113
    BreakLoop,

    /// Exit event processing, id 115
    ExitEvent,

    /// Call common event, id 117
    CallCommonEvent { event: usize },

    /// Control switches, id 121
    ControlSwitches {
        range: RangeInclusive<usize>,
        state: bool,
    },

    /// Script, id 355
    Script { text: String },
    /// Extended script, id 655
    ScriptExt { text: String },

    /// Scroll screen, id 203
    ScrollScreen {
        direction: usize,
        distance: usize,
        speed: usize,
    },

    /// Move route, id 209
    MoveRoute { target: i32, route: rpg::MoveRoute },
    /// Wait until move route finished, id 210
    WaitMoveRoute,

    /// Play SE, id 250
    PlaySE { file: rpg::AudioFile },

    //? Special commands ?//
    /// Special editor move command display.
    /// We don't need it.
    MoveDisplay,
    /// Invalid, invalid command ID
    Invalid {
        code: i32,
        parameters: Vec<ParameterType>,
    },
    /// Fields: (params: [`Vec<ParameterType>`])
    Custom { params: Vec<ParameterType> },
    /// Insert command
    Insert,
}

pub use CommandKind::*;

// TODO: Make this a macro

impl From<intermediate::EventCommand> for Command {
    fn from(cmd: intermediate::EventCommand) -> Self {
        let intermediate::EventCommand {
            code,
            indent,
            parameters,
        } = cmd;

        Self {
            indent,
            kind: match code {
                0 => CommandKind::Insert,
                101 => Text {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                401 => TextExt {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                102 => Choices {
                    choices: parameters[0].clone().into_array().unwrap(),
                    cancel_type: parameters[1].clone().into_integer().unwrap(),
                },
                402 => When {
                    choice: parameters[0].clone().into_integer().unwrap(),
                },
                403 => WhenCancel,
                404 => ChoiceEnd,
                106 => CommandKind::Wait {
                    time: parameters[0].clone().into_integer().unwrap(),
                },
                108 => Comment {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                408 => CommentExt {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                115 => ExitEvent,
                117 => CallCommonEvent {
                    event: parameters[0].clone().into_integer().unwrap() as usize,
                },
                121 => ControlSwitches {
                    range: (parameters[0].clone().into_integer().unwrap() as usize)
                        ..=(parameters[1].clone().into_integer().unwrap() as usize),
                    state: parameters[2].clone().into_integer().unwrap() == 0,
                },
                355 => CommandKind::Script {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                655 => ScriptExt {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                111 => Conditional {
                    kind: parameters[0].clone().into_integer().unwrap(),
                    params: parameters[1..].to_vec(),
                },
                411 => Else,
                412 => BranchEnd,
                112 => Loop,
                113 => BreakLoop,
                413 => LoopEnd,
                203 => ScrollScreen {
                    direction: parameters[0].clone().into_integer().unwrap() as usize,
                    distance: parameters[1].clone().into_integer().unwrap() as usize,
                    speed: parameters[2].clone().into_integer().unwrap() as usize,
                },
                209 => MoveRoute {
                    target: *parameters[0].as_integer().unwrap(),
                    route: parameters[1].as_move_route().unwrap().clone(),
                },
                210 => WaitMoveRoute,
                250 => CommandKind::PlaySE {
                    file: parameters[0].as_audio_file().unwrap().clone(),
                },
                509 => MoveDisplay,
                _ => CommandKind::Invalid { code, parameters },
            },
        }
    }
}

// TODO: Make this a macro

impl From<Command> for intermediate::EventCommand {
    fn from(cmd: Command) -> Self {
        let (code, parameters) = match cmd.kind {
            Text { text } => (101, vec![text.into()]),
            _ => (0, vec![]),
        };

        Self {
            indent: cmd.indent,
            code,
            parameters,
        }
    }
}

/// An enum representing move commands.
#[allow(missing_docs)]
#[derive(Debug, Clone, EnumAsInner, PartialEq, Serialize, Deserialize)]
#[serde(from = "intermediate::MoveCommand")]
#[serde(into = "intermediate::MoveCommand")]
pub enum MoveCommand {
    /// Move down, id 1
    Down,
    /// Move left, id 2
    Left,
    /// Move right, id 3
    Right,
    /// Move up, id 4
    Up,
    /// Move lower left, 5
    LowerLeft,
    /// Move lower right, 6
    LowerRight,
    /// Move upper left, 7
    UpperLeft,
    /// Move upper right, 8
    UpperRight,
    /// Move random, 9
    Random,
    /// Move towards player, 10
    MoveTowards,
    /// Move away from player, 11
    MoveAway,
    /// Step forward, 12
    Forward,
    /// Step backwards, 13
    Backwards,
    /// Jump, 14
    Jump {
        x_plus: i32,
        y_plus: i32,
    },
    /// Wait, 15
    Wait {
        time: i32,
    },
    /// Turn down, 16
    TurnDown,
    /// Turn down, 17
    TurnLeft,
    /// Turn down, 18
    TurnRight,
    /// Turn down, 19
    TurnUp,
    /// Turn right 90, 20
    TurnRight90,
    /// Turn left 90, 21
    TurnLeft90,
    /// Turn 180, 22
    Turn180,
    /// Turn right or left 90, 23
    TurnRightOrLeft,
    /// Turn random, 24
    TurnRandom,
    /// Turn towards player, 25
    TurnTowardsPlayer,
    /// Turn away from player, 26
    TurnAwayFromPlayer,
    /// Switch ON, 27
    SwitchON {
        switch_id: usize,
    },
    /// Switch OFF, 28
    SwitchOFF {
        switch_id: usize,
    },
    /// Change speed, 29
    ChangeSpeed {
        speed: usize,
    },
    /// Change freq, 30
    ChangeFreq {
        freq: usize,
    },
    /// Move anim ON, 31
    MoveON,
    /// Move anim OFF, 32
    MoveOFF,
    /// Stop anim ON, 33
    StopON,
    /// Stop anim OFF, 34
    StopOFF,
    /// Direction fix ON, 35
    DirFixON,
    /// Direction fix OFF, 36
    DirFixOFF,
    /// Through ON, 37
    ThroughON,
    /// Through OFF, 38
    ThroughOFF,
    /// Always on top ON, 39
    AlwaysTopON,
    /// Always on top OFF, 40
    AlwaysTopOFF,
    /// Change graphic, 41
    ChangeGraphic {
        character_name: String,
        character_hue: i32,
        direction: i32,
        pattern: i32,
    },
    /// Change opacity, 42
    ChangeOpacity {
        opacity: i32,
    },
    /// Change blending, 43
    ChangeBlend {
        blend: i32,
    },
    /// Play SE, 44
    PlaySE {
        file: rpg::AudioFile,
    },
    /// Script, 45
    Script {
        text: String,
    },

    Break,
    Invalid {
        code: i32,
        parameters: Vec<ParameterType>,
    },
}

pub use MoveCommand::*;

impl From<intermediate::MoveCommand> for MoveCommand {
    fn from(value: intermediate::MoveCommand) -> Self {
        let intermediate::MoveCommand { code, parameters } = value;

        match code {
            1 => Down,
            2 => Left,
            3 => Right,
            4 => Up,
            5 => LowerLeft,
            6 => LowerRight,
            7 => UpperLeft,
            8 => UpperRight,
            9 => Random,
            10 => MoveTowards,
            11 => MoveAway,
            12 => Forward,
            13 => Backwards,
            14 => Jump {
                x_plus: *parameters[0].as_integer().unwrap(),
                y_plus: *parameters[1].as_integer().unwrap(),
            },
            15 => MoveCommand::Wait {
                time: *parameters[0].as_integer().unwrap(),
            },
            16 => TurnDown,
            17 => TurnLeft,
            18 => TurnRight,
            19 => TurnUp,
            20 => TurnRight90,
            21 => TurnLeft90,
            22 => Turn180,
            23 => TurnRightOrLeft,
            24 => TurnRandom,
            25 => TurnTowardsPlayer,
            26 => TurnAwayFromPlayer,
            27 => SwitchON {
                switch_id: *parameters[0].as_integer().unwrap() as usize,
            },
            28 => SwitchOFF {
                switch_id: *parameters[0].as_integer().unwrap() as usize,
            },
            29 => ChangeSpeed {
                speed: *parameters[0].as_integer().unwrap() as usize,
            },
            30 => ChangeFreq {
                freq: *parameters[0].as_integer().unwrap() as usize,
            },
            31 => MoveON,
            32 => MoveOFF,
            33 => StopON,
            34 => StopOFF,
            35 => DirFixON,
            36 => DirFixOFF,
            37 => ThroughON,
            38 => ThroughOFF,
            39 => AlwaysTopON,
            40 => AlwaysTopOFF,
            41 => ChangeGraphic {
                character_name: parameters[0].as_string().unwrap().clone(),
                character_hue: *parameters[1].as_integer().unwrap(),
                direction: *parameters[2].as_integer().unwrap(),
                pattern: *parameters[3].as_integer().unwrap(),
            },
            42 => ChangeOpacity {
                opacity: *parameters[0].as_integer().unwrap(),
            },
            43 => ChangeBlend {
                blend: *parameters[0].as_integer().unwrap(),
            },
            44 => Self::PlaySE {
                file: parameters[0].as_audio_file().unwrap().clone(),
            },
            45 => Self::Script {
                text: parameters[0].as_string().unwrap().clone(),
            },

            0 => Self::Break,
            _ => MoveCommand::Invalid { code, parameters },
        }
    }
}

impl From<MoveCommand> for intermediate::MoveCommand {
    fn from(value: MoveCommand) -> Self {
        let (code, parameters) = match value {
            Down => (1, vec![]),
            Left => (2, vec![]),
            Right => (3, vec![]),
            Up => (4, vec![]),
            _ => (0, vec![]),
        };

        Self { code, parameters }
    }
}

/// Process a move route by converting it into a series of points.
pub fn process_move_route(
    move_route: &rpg::MoveRoute,
    directions: &mut Vec<i32>,
    points: &mut Vec<egui::Pos2>,
) {
    for command in move_route.list.iter() {
        let current_pos = points.last().unwrap();
        let current_direction = directions.last().unwrap();
        match command {
            Down => {
                directions.push(2);
                points.push(egui::pos2(current_pos.x, current_pos.y + 1.));
            }
            Left => {
                directions.push(4);
                points.push(egui::pos2(current_pos.x - 1., current_pos.y));
            }
            Right => {
                directions.push(6);
                points.push(egui::pos2(current_pos.x + 1., current_pos.y));
            }
            Up => {
                directions.push(8);
                points.push(egui::pos2(current_pos.x, current_pos.y - 1.));
            }
            LowerLeft => {
                if *current_direction == 4 {
                    directions.push(6)
                } else if *current_direction == 8 {
                    directions.push(2)
                }
                points.push(egui::pos2(current_pos.x - 1., current_pos.y + 1.));
            }
            LowerRight => {
                if *current_direction == 6 {
                    directions.push(4)
                } else if *current_direction == 8 {
                    directions.push(2)
                }
                points.push(egui::pos2(current_pos.x + 1., current_pos.y + 1.));
            }
            UpperLeft => {
                if *current_direction == 6 {
                    directions.push(4)
                } else if *current_direction == 2 {
                    directions.push(8)
                }
                points.push(egui::pos2(current_pos.x - 1., current_pos.y - 1.));
            }
            UpperRight => {
                if *current_direction == 4 {
                    directions.push(6)
                } else if *current_direction == 2 {
                    directions.push(8)
                }
                points.push(egui::pos2(current_pos.x + 1., current_pos.y - 1.));
            }
            Forward => match current_direction {
                2 => {
                    points.push(egui::pos2(current_pos.x, current_pos.y + 1.));
                }
                4 => {
                    points.push(egui::pos2(current_pos.x - 1., current_pos.y));
                }
                6 => {
                    points.push(egui::pos2(current_pos.x + 1., current_pos.y));
                }
                8 => {
                    points.push(egui::pos2(current_pos.x, current_pos.y - 1.));
                }
                _ => unreachable!(),
            },
            Backwards => match current_direction {
                2 => {
                    points.push(egui::pos2(current_pos.x, current_pos.y - 1.));
                    directions.push(8);
                }
                4 => {
                    points.push(egui::pos2(current_pos.x + 1., current_pos.y));
                    directions.push(6);
                }
                6 => {
                    points.push(egui::pos2(current_pos.x - 1., current_pos.y));
                    directions.push(4);
                }
                8 => {
                    points.push(egui::pos2(current_pos.x, current_pos.y + 1.));
                    directions.push(2);
                }
                _ => unreachable!(),
            },
            TurnDown => {
                directions.push(2);
            }
            TurnLeft => {
                directions.push(4);
            }
            TurnRight => {
                directions.push(6);
            }
            TurnUp => {
                directions.push(8);
            }
            TurnRight90 => {
                directions.push(match current_direction {
                    2 => 4,
                    4 => 8,
                    6 => 2,
                    8 => 6,
                    _ => unreachable!(),
                });
            }
            TurnLeft90 => {
                directions.push(match current_direction {
                    2 => 6,
                    4 => 2,
                    6 => 8,
                    8 => 4,
                    _ => unreachable!(),
                });
            }
            Turn180 => {
                directions.push(match current_direction {
                    2 => 8,
                    4 => 6,
                    6 => 4,
                    8 => 2,
                    _ => unreachable!(),
                });
            }
            _ => {}
        }
    }
}

/// TODO: Make into enums

/// Move types
pub const MOVE_TYPES: [&str; 4] = ["Fixed", "Random", "Approach", "Custom"];
/// Move speeds
pub const MOVE_SPEEDS: [&str; 6] = ["Slowest", "Slower", "Slow", "Fast", "Faster", "Fastest"];
/// Move frequencies
pub const MOVE_FREQS: [&str; 6] = ["Lowest", "Lower", "Low", "High", "Higher", "Highest"];
