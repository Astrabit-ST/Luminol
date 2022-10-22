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

use serde::{Deserialize, Serialize};

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
#[serde(from = "EventCommand")]
#[serde(into = "EventCommand")]
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
        choice_type: i32,
        indent: usize,
    },
    /// When, id 402
    ///
    /// Fields (???: [`i32`])
    When { unknown: i32 },
    /// When Cancel, id 403
    ///
    /// Fields (???: [`i32`])
    WhenCancel { unknown: i32 },
    /// Conditional Branch (If statement), id 111
    ///
    /// Fields (kind: [`i32`], params: [`Vec<ParameterType>`])
    Conditional {
        kind: i32,
        params: Vec<ParameterType>,
    },
    /// Else, id 411
    Else,
    /// Loop
    Loop,
    /// Comment, id 108
    Comment { text: String },
    /// CommentExt, id 408
    CommentExt { text: String },
    /// Wait, id 106
    Wait { time: i32 },
    /// Script, id 355
    Script { text: String },
    /// Extended script, id 655
    ScriptExt { text: String },

    //? Special commands ?//
    /// Invalid, invalid command ID
    Invalid {
        code: i32,
        parameters: Vec<ParameterType>,
    },
    /// Fields: (params: [`Vec<ParameterType>`])
    Custom { params: Vec<ParameterType> },
    /// Break from...?
    Break,
}

pub use CommandKind::*;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(missing_docs)]
pub struct EventCommand {
    pub code: i32,
    pub indent: usize,
    pub parameters: Vec<ParameterType>,
}

// TODO: Make this a macro

impl From<EventCommand> for Command {
    fn from(cmd: EventCommand) -> Self {
        let EventCommand {
            code,
            indent,
            parameters,
        } = cmd;

        Self {
            indent,
            kind: match code {
                0 => Break,
                101 => Text {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                401 => TextExt {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                106 => Wait {
                    time: parameters[0].clone().into_integer().unwrap(),
                },
                108 => Comment {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                408 => CommentExt {
                    text: parameters[0].clone().into_string().unwrap(),
                },
                355 => Script {
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
                112 => Loop,
                _ => Invalid { code, parameters },
            },
        }
    }
}

// TODO: Make this a macro

impl From<Command> for EventCommand {
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

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct MoveCommand {
    pub code: i32,
    pub parameters: Vec<ParameterType>,
}
