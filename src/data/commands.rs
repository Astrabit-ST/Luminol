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
#[serde(from = "alox_48::Value")]
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
    Bool(bool),
}

impl From<alox_48::Value> for ParameterType {
    fn from(value: alox_48::Value) -> Self {
        use alox_48::Value;

        match value {
            Value::Integer(i) => Self::Integer(i as _),
            Value::String(str) => Self::String(str.to_string().unwrap()),
            // Value::Symbol(sym) => Self::String(sym),
            Value::Object(obj) if obj.class == "RPG::AudioFile" => {
                Self::AudioFile(rpg::AudioFile {
                    name: obj.fields["name"]
                        .clone()
                        .into_string()
                        .unwrap()
                        .to_string()
                        .unwrap(),
                    volume: obj.fields["volume"].clone().into_integer().unwrap() as _,
                    pitch: obj.fields["pitch"].clone().into_integer().unwrap() as _,
                })
            }
            Value::Object(obj) if obj.class == "RPG::MoveRoute" => {
                Self::MoveRoute(rpg::MoveRoute {
                    repeat: obj.fields["repeat"].clone().into_bool().unwrap(),
                    skippable: obj.fields["skippable"].clone().into_bool().unwrap(),
                    list: obj.fields["list"]
                        .clone()
                        .into_array()
                        .unwrap()
                        .into_iter()
                        .map(|obj| {
                            let obj = obj.into_object().unwrap();

                            intermediate::MoveCommand {
                                code: obj.fields["code"].clone().into_integer().unwrap() as _,
                                parameters: obj.fields["parameters"]
                                    .clone()
                                    .into_array()
                                    .unwrap()
                                    .into_iter()
                                    .map(Into::into)
                                    .collect(),
                            }
                            .into()
                        })
                        .collect(),
                })
            }
            Value::Object(obj) if obj.class == "RPG::MoveCommand" => Self::MoveCommand(
                intermediate::MoveCommand {
                    code: obj.fields["code"].clone().into_integer().unwrap() as _,
                    parameters: obj.fields["parameters"]
                        .clone()
                        .into_array()
                        .unwrap()
                        .into_iter()
                        .map(Into::into)
                        .collect(),
                }
                .into(),
            ),
            Value::Float(f) => Self::Float(f as _),
            Value::Array(ary) => Self::Array(
                ary.clone()
                    .into_iter()
                    .map(|v| v.into_string().unwrap().to_string().unwrap())
                    .collect(),
            ),
            Value::Bool(b) => Self::Bool(b),
            Value::Userdata(data) if data.class == "Color" => {
                let floats = bytemuck::cast_slice(&data.data);

                Self::Color(Color {
                    red: floats[0],
                    green: floats[1],
                    blue: floats[2],
                    alpha: floats[3],
                })
            }
            Value::Userdata(data) if data.class == "Tone" => {
                let floats = bytemuck::cast_slice(&data.data);

                Self::Tone(Tone {
                    red: floats[0],
                    green: floats[1],
                    blue: floats[2],
                    gray: floats[3],
                })
            }
            _ => panic!("Unexpected type {value:#?}"),
        }
    }
}

impl From<String> for ParameterType {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

// FIXME: NOT ALL OF THESE ARE KNOWN

/// An enum representing event commands.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(try_from = "intermediate::EventCommand")]
#[serde(into = "intermediate::EventCommand")]
pub struct Command {
    /// Command indent
    pub indent: usize,
    /// Type of command
    pub kind: CommandKind,
    code: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    /// Change text options, id 104
    TextOptions { position: TextPosition, show: bool },
    /// Button input, id 105
    ButtonInput { id: usize },
    /// Conditional Branch (If statement), id 111
    ///
    /// Fields (kind: [`conditional_kind`])
    Conditional { kind: ConditionalKind },
    /// Else, id 411
    Else,
    /// Branch end, id 412
    BranchEnd,
    /// Loop, id 112
    Loop,
    /// Comment, id 108
    Comment { text: String },
    /// CommentExt, id 408
    CommentExt { text: String },
    /// Wait, id 106
    Wait { time: i32 },

    /// Break Loop, id 113
    BreakLoop,
    /// Repeat above, id 413
    RepeatAbove,

    /// Exit event processing, id 115
    ExitEvent,
    /// Erase event, id 116
    EraseEvent,
    /// Call common event, id 117
    CallCommonEvent { event: usize },
    /// Label, id 118
    Label { text: String },
    /// Jump to label, id 119
    JumpToLabel { label: String },
    /// Control switches, id 121
    ControlSwitches {
        range: RangeInclusive<usize>,
        state: bool,
    },
    /// Control variables, id 122
    ControlVariables {
        range: RangeInclusive<usize>,
        kind: VariableKind,
        operation: VariableOperation,
    },
    /// Control Self Switch, id 123
    ControlSelfSwitch { switch: String, state: bool },

    /// Change items. id 126
    ChangeItems { id: usize, operation: Operation },

    /// Change party member, id 129
    ChangeParty {
        id: usize,
        add: bool,
        initialize: bool,
    },

    /// Transfer player, id 201
    TransferPlayer {
        variable: bool,
        transfer_id: i32,
        transfer_x: i32,
        transfer_y: i32,
        direction: Direction,
        fade: bool,
    },

    /// Scroll screen, id 203
    ScrollScreen {
        direction: usize,
        distance: usize,
        speed: usize,
    },

    /// Play animation, id 207
    PlayAnimation { target: i32, id: usize },

    /// Move route, id 209
    MoveRoute { target: i32, route: rpg::MoveRoute },
    /// Wait until move route finished, id 210
    WaitMoveRoute,

    /// Change screen tone, id 223
    ScreenTone { tone: Tone, duration: i32 },
    /// Screen flash, id 224
    ScreenFlash { color: Color, duration: i32 },

    /// Screen shake, id 225
    ScreenShake { power: i32, speed: i32, time: i32 },

    /// Show picture, id 231
    ShowPicture {
        id: usize,
        name: String,
        variable: bool,
        x: i32,
        y: i32,
        zoom_x: usize,
        zoom_y: usize,
        opacity: u8,
        center: bool,
        blend_type: BlendType,
    },
    /// Move Picture, id 232
    MovePicture {
        id: usize,
        duration: usize,
        variable: bool,
        x: i32,
        y: i32,
        zoom_x: usize,
        zoom_y: usize,
        opacity: u8,
        center: bool,
        blend_type: BlendType,
    },

    /// Erase picture, id 235
    ErasePicture { id: usize },

    /// Play BGM, id 241
    PlayBGM { file: rpg::AudioFile },
    /// Fade BGM, id 242
    FadeBGM { time: i32 },
    /// Memorize BGM, id 247
    MemorizeBGM,
    /// Restore BGM, id 248
    RestoreBGM,
    /// PLay ME, id 249
    PlayME { file: rpg::AudioFile },
    /// Play SE, id 250
    PlaySE { file: rpg::AudioFile },

    /// Change actor graphic, id 322
    ChangeActorGraphic {
        id: usize,
        character_name: String,
        character_hue: i32,
        battler_name: String,
        battler_hue: i32,
    },

    /// Script, id 355
    Script { text: String },
    /// Extended script, id 655
    ScriptExt { text: String },

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
    #[default]
    Insert,
}

pub use CommandKind::*;
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalKind {
    Switch {
        id: usize,
        state: bool,
    },
    Variable {
        id: usize,
        const_value: Option<i32>,
        variable_value: Option<usize>,
        operator: ConditionalOperator,
    },
    SelfSwitch {
        char: String,
        state: bool,
    },
    Timer {
        seconds: i32,
        or_more: bool,
    },
    Actor {
        id: usize,
        kind: ActorCondition,
    },
    Enemy {
        id: usize,
        state: Option<usize>,
    },
    Character {
        id: usize,
        direction: Direction,
    },
    Gold {
        amount: i32,
        or_more: bool,
    },
    Item {
        id: usize,
    },
    Weapon {
        id: usize,
    },
    Armor {
        id: usize,
    },
    Button {
        id: usize,
    },
    Script {
        text: String,
    },
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionalOperator {
    Equal,
    GreaterEqual,
    LessEqual,
    Greater,
    Less,
    NotEqual,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum ActorCondition {
    InParty,
    Name(String),
    Skill(usize),
    Weapon(usize),
    Armor(usize),
    State(usize),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub operand: Operand,
    pub kind: OperandKind,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Variable(usize),
    Constant(i32),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum OperandKind {
    Add,
    Subtract,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum TextPosition {
    Top,
    Middle,
    Bottom,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableKind {
    Constant(i32),
    Variable(usize),
    Random(RangeInclusive<i32>),
    Item(usize),
    Actor(usize, ActorVar),
    Enemy(usize, EnemyVar),
    Character(i32, CharacterVar),
    MapID,
    PartySize,
    Gold,
    Steps,
    PlayTime,
    Timer,
    SaveCount,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum ActorVar {
    Level,
    Exp,
    HP,
    SP,
    MaxHP,
    MaxSP,
    Strength,
    Dexterity,
    Agility,
    Intelligence,
    Attack,
    PhysicalDefence,
    MagicDefence,
    Evasion,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum EnemyVar {
    HP,
    SP,
    MaxHP,
    MaxSP,
    Strength,
    Dexterity,
    Agility,
    Intelligence,
    Attack,
    PhysicalDefence,
    MagicDefence,
    Evasion,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum CharacterVar {
    X,
    Y,
    Direction,
    ScreenX,
    ScreenY,
    TerrainTag,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum VariableOperation {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
}

impl Command {
    fn from_cmd(cmd: intermediate::EventCommand) -> Result<Self, ParameterType> {
        use ActorCondition::*;
        use ConditionalKind::*;
        use ConditionalOperator::*;
        let intermediate::EventCommand {
            code,
            indent,
            parameters,
        } = cmd;

        Ok(Self {
            indent,
            code,
            kind: match code {
                0 => CommandKind::Insert,
                101 => Text {
                    text: parameters[0].clone().into_string()?,
                },
                401 => TextExt {
                    text: parameters[0].clone().into_string()?,
                },
                102 => Choices {
                    choices: parameters[0].clone().into_array()?,
                    cancel_type: parameters[1].clone().into_integer()?,
                },
                402 => When {
                    choice: parameters[0].clone().into_integer()?,
                },
                403 => WhenCancel,
                404 => ChoiceEnd,
                104 => TextOptions {
                    position: match parameters[1].clone().into_integer()? {
                        0 => TextPosition::Top,
                        1 => TextPosition::Middle,
                        2 => TextPosition::Bottom,
                        _ => panic!("Invalid text position"),
                    },
                    show: parameters[1].clone().into_integer()? == 0,
                },
                105 => ButtonInput {
                    id: parameters[0].clone().into_integer()? as usize,
                },
                106 => CommandKind::Wait {
                    time: parameters[0].clone().into_integer()?,
                },
                108 => Comment {
                    text: parameters[0].clone().into_string()?,
                },
                408 => CommentExt {
                    text: parameters[0].clone().into_string()?,
                },
                111 => Conditional {
                    kind: match parameters[0].clone().into_integer()? {
                        0 => ConditionalKind::Switch {
                            id: parameters[1].clone().into_integer()? as usize,
                            state: parameters[2].clone().into_integer()? == 0,
                        },
                        1 => {
                            let id = parameters[1].clone().into_integer()? as usize;
                            let operator = match parameters[4].clone().into_integer()? {
                                0 => Equal,
                                1 => GreaterEqual,
                                2 => LessEqual,
                                3 => Greater,
                                4 => Less,
                                5 => NotEqual,
                                _ => panic!("Invalid conditional operator"),
                            };

                            if parameters[2].clone().into_integer()? == 0 {
                                Variable {
                                    id,
                                    const_value: Some(parameters[3].clone().into_integer()?),
                                    variable_value: None,
                                    operator,
                                }
                            } else {
                                Variable {
                                    id,
                                    const_value: None,
                                    variable_value: Some(
                                        parameters[3].clone().into_integer()? as usize
                                    ),
                                    operator,
                                }
                            }
                        }
                        2 => SelfSwitch {
                            char: parameters[1].clone().into_string()?,
                            state: parameters[2].clone().into_integer()? == 0,
                        },
                        3 => Timer {
                            seconds: parameters[1].clone().into_integer()?,
                            or_more: parameters[2].clone().into_integer()? == 0,
                        },
                        4 => {
                            let id = parameters[1].clone().into_integer()? as usize;

                            let kind = match parameters[2].clone().into_integer()? {
                                0 => InParty,
                                1 => Name(parameters[3].clone().into_string()?),
                                2 => Skill(parameters[3].clone().into_integer()? as usize),
                                3 => ActorCondition::Weapon(
                                    parameters[3].clone().into_integer()? as usize
                                ),
                                4 => ActorCondition::Armor(
                                    parameters[3].clone().into_integer()? as usize
                                ),
                                5 => State(parameters[3].clone().into_integer()? as usize),
                                _ => panic!("Actor conditional invalid"),
                            };

                            Actor { id, kind }
                        }
                        5 => {
                            let id = parameters[1].clone().into_integer()? as usize;
                            let state = if parameters[2].clone().into_integer()? == 0 {
                                None
                            } else {
                                Some(parameters[3].clone().into_integer()? as usize)
                            };

                            Enemy { id, state }
                        }
                        6 => Character {
                            id: parameters[1].clone().into_integer()? as usize,
                            direction: match parameters[2].clone().into_integer()? {
                                2 => Direction::Down,
                                4 => Direction::Left,
                                6 => Direction::Right,
                                8 => Direction::Up,
                                _ => panic!("Invalid direction"),
                            },
                        },
                        7 => Gold {
                            amount: parameters[1].clone().into_integer()?,
                            or_more: parameters[2].clone().into_integer()? == 0,
                        },
                        8 => Item {
                            id: parameters[1].clone().into_integer()? as usize,
                        },
                        9 => ConditionalKind::Weapon {
                            id: parameters[1].clone().into_integer()? as usize,
                        },
                        10 => ConditionalKind::Armor {
                            id: parameters[1].clone().into_integer()? as usize,
                        },
                        11 => Button {
                            id: parameters[1].clone().into_integer()? as usize,
                        },
                        12 => ConditionalKind::Script {
                            text: parameters[1].clone().into_string()?,
                        },
                        _ => panic!("Invalid conditional type"),
                    },
                },
                115 => ExitEvent,
                116 => EraseEvent,
                117 => CallCommonEvent {
                    event: parameters[0].clone().into_integer()? as usize,
                },
                118 => Label {
                    text: parameters[0].clone().into_string()?,
                },
                119 => JumpToLabel {
                    label: parameters[0].clone().into_string()?,
                },
                121 => ControlSwitches {
                    range: (parameters[0].clone().into_integer()? as usize)
                        ..=(parameters[1].clone().into_integer()? as usize),
                    state: parameters[2].clone().into_integer()? == 0,
                },
                122 => ControlVariables {
                    range: (parameters[0].clone().into_integer()? as usize)
                        ..=(parameters[1].clone().into_integer()? as usize),
                    kind: match parameters[3].clone().into_integer()? {
                        0 => VariableKind::Constant(parameters[4].clone().into_integer()?),
                        1 => VariableKind::Variable(parameters[4].clone().into_integer()? as usize),
                        2 => VariableKind::Random(
                            (parameters[0].clone().into_integer()?)
                                ..=(parameters[1].clone().into_integer()?),
                        ),
                        3 => VariableKind::Item(parameters[4].clone().into_integer()? as usize),
                        4 => VariableKind::Actor(
                            parameters[4].clone().into_integer()? as usize,
                            match parameters[5].clone().into_integer()? {
                                0 => ActorVar::Level,
                                1 => ActorVar::Exp,
                                2 => ActorVar::HP,
                                3 => ActorVar::SP,
                                4 => ActorVar::MaxHP,
                                5 => ActorVar::MaxSP,
                                6 => ActorVar::Strength,
                                7 => ActorVar::Dexterity,
                                8 => ActorVar::Agility,
                                9 => ActorVar::Intelligence,
                                10 => ActorVar::Attack,
                                11 => ActorVar::PhysicalDefence,
                                12 => ActorVar::MagicDefence,
                                13 => ActorVar::Evasion,
                                _ => panic!("Invalid actor variable type"),
                            },
                        ),
                        5 => VariableKind::Enemy(
                            parameters[4].clone().into_integer()? as usize,
                            match parameters[5].clone().into_integer()? {
                                0 => EnemyVar::HP,
                                1 => EnemyVar::SP,
                                2 => EnemyVar::MaxHP,
                                3 => EnemyVar::MaxSP,
                                4 => EnemyVar::Strength,
                                5 => EnemyVar::Dexterity,
                                6 => EnemyVar::Agility,
                                7 => EnemyVar::Intelligence,
                                8 => EnemyVar::Attack,
                                9 => EnemyVar::PhysicalDefence,
                                10 => EnemyVar::MagicDefence,
                                11 => EnemyVar::Evasion,
                                _ => panic!("Invalid actor variable type"),
                            },
                        ),
                        6 => VariableKind::Character(
                            parameters[4].clone().into_integer()?,
                            match parameters[5].clone().into_integer()? {
                                0 => CharacterVar::X,
                                1 => CharacterVar::Y,
                                2 => CharacterVar::Direction,
                                3 => CharacterVar::ScreenX,
                                4 => CharacterVar::ScreenY,
                                5 => CharacterVar::TerrainTag,
                                _ => panic!("Invalid character variable type"),
                            },
                        ),
                        7 => match parameters[4].clone().into_integer()? {
                            0 => VariableKind::MapID,
                            1 => VariableKind::PartySize,
                            2 => VariableKind::Gold,
                            3 => VariableKind::Steps,
                            4 => VariableKind::PlayTime,
                            5 => VariableKind::Timer,
                            6 => VariableKind::SaveCount,
                            _ => panic!("Invalid variable kind"),
                        },
                        _ => panic!("Invalid variable kind"),
                    },
                    operation: match parameters[2].clone().into_integer()? {
                        0 => VariableOperation::Set,
                        1 => VariableOperation::Add,
                        2 => VariableOperation::Subtract,
                        3 => VariableOperation::Multiply,
                        4 => VariableOperation::Divide,
                        5 => VariableOperation::Modulo,
                        _ => panic!("Invalid variable operation"),
                    },
                },
                123 => ControlSelfSwitch {
                    switch: parameters[0].clone().into_string()?,
                    state: parameters[1].clone().into_integer()? == 0,
                },
                126 => ChangeItems {
                    id: parameters[0].clone().into_integer()? as usize,
                    operation: Operation {
                        operand: match parameters[2].clone().into_integer()? {
                            0 => Operand::Constant(parameters[3].clone().into_integer()?),
                            1 => Operand::Variable(parameters[3].clone().into_integer()? as usize),
                            _ => panic!("Invalid operand type"),
                        },
                        kind: match parameters[1].clone().into_integer()? {
                            1 => OperandKind::Subtract,
                            0 => OperandKind::Add,
                            _ => panic!("Invalid operation kind"),
                        },
                    },
                },
                129 => ChangeParty {
                    id: parameters[0].clone().into_integer()? as usize,
                    add: parameters[1].clone().into_integer()? == 0,
                    initialize: parameters[2].clone().into_integer()? == 1,
                },

                411 => Else,
                412 => BranchEnd,
                112 => Loop,
                113 => BreakLoop,
                413 => RepeatAbove,
                201 => TransferPlayer {
                    variable: parameters[0].clone().into_integer()? != 0,
                    transfer_id: parameters[1].clone().into_integer()?,
                    transfer_x: parameters[2].clone().into_integer()?,
                    transfer_y: parameters[3].clone().into_integer()?,
                    direction: match parameters[4].clone().into_integer()? {
                        0 => Direction::Retain,
                        2 => Direction::Down,
                        4 => Direction::Left,
                        6 => Direction::Right,
                        8 => Direction::Up,
                        _ => panic!("Invalid direction"),
                    },
                    fade: parameters[5].clone().into_integer()? == 0,
                },
                203 => ScrollScreen {
                    direction: parameters[0].clone().into_integer()? as usize,
                    distance: parameters[1].clone().into_integer()? as usize,
                    speed: parameters[2].clone().into_integer()? as usize,
                },
                207 => PlayAnimation {
                    target: parameters[0].clone().into_integer()?,
                    id: parameters[1].clone().into_integer()? as usize,
                },
                209 => MoveRoute {
                    target: parameters[0].clone().into_integer()?,
                    route: parameters[1].clone().into_move_route()?,
                },
                509 => MoveDisplay,
                210 => WaitMoveRoute,
                223 => ScreenTone {
                    tone: parameters[0].clone().into_tone()?,
                    duration: parameters[1].clone().into_integer()?,
                },
                224 => ScreenFlash {
                    color: parameters[0].clone().into_color()?,
                    duration: parameters[1].clone().into_integer()?,
                },
                225 => ScreenShake {
                    power: parameters[0].clone().into_integer()?,
                    speed: parameters[1].clone().into_integer()?,
                    time: parameters[2].clone().into_integer()?,
                },
                231 => ShowPicture {
                    id: parameters[0].clone().into_integer()? as usize,
                    name: parameters[1].clone().into_string()?,
                    center: parameters[2].clone().into_integer()? != 0,
                    variable: parameters[3].clone().into_integer()? != 0,
                    x: parameters[4].clone().into_integer()?,
                    y: parameters[5].clone().into_integer()?,
                    zoom_x: parameters[6].clone().into_integer()? as usize,
                    zoom_y: parameters[7].clone().into_integer()? as usize,
                    opacity: parameters[8].clone().into_integer()? as u8,
                    blend_type: match parameters[9].clone().into_integer()? {
                        0 => BlendType::Normal,
                        1 => BlendType::Additive,
                        2 => BlendType::Subtractive,
                        _ => panic!("Invalid blend type"),
                    },
                },
                232 => MovePicture {
                    id: parameters[0].clone().into_integer()? as usize,
                    duration: parameters[1].clone().into_integer()? as usize,
                    center: parameters[2].clone().into_integer()? != 0,
                    variable: parameters[3].clone().into_integer()? != 0,
                    x: parameters[4].clone().into_integer()?,
                    y: parameters[5].clone().into_integer()?,
                    zoom_x: parameters[6].clone().into_integer()? as usize,
                    zoom_y: parameters[7].clone().into_integer()? as usize,
                    opacity: parameters[8].clone().into_integer()? as u8,
                    blend_type: match parameters[9].clone().into_integer()? {
                        0 => BlendType::Normal,
                        1 => BlendType::Additive,
                        2 => BlendType::Subtractive,
                        _ => panic!("Invalid blend type"),
                    },
                },
                235 => ErasePicture {
                    id: parameters[0].clone().into_integer()? as usize,
                },

                241 => PlayBGM {
                    file: parameters[0].clone().into_audio_file()?,
                },
                242 => FadeBGM {
                    time: parameters[0].clone().into_integer()?,
                },
                247 => MemorizeBGM,
                248 => RestoreBGM,
                249 => PlayME {
                    file: parameters[0].clone().into_audio_file()?,
                },
                250 => CommandKind::PlaySE {
                    file: parameters[0].clone().into_audio_file()?,
                },

                322 => ChangeActorGraphic {
                    id: parameters[0].clone().into_integer()? as usize,
                    character_name: parameters[1].clone().into_string()?,
                    character_hue: parameters[2].clone().into_integer()?,
                    battler_name: parameters[3].clone().into_string()?,
                    battler_hue: parameters[4].clone().into_integer()?,
                },

                355 => CommandKind::Script {
                    text: parameters[0].clone().into_string()?,
                },
                655 => ScriptExt {
                    text: parameters[0].clone().into_string()?,
                },

                _ => CommandKind::Invalid { code, parameters },
            },
        })
    }
}

// TODO: Make this a macro

impl TryFrom<intermediate::EventCommand> for Command {
    type Error = String;

    fn try_from(cmd: intermediate::EventCommand) -> Result<Self, Self::Error> {
        Command::from_cmd(cmd)
            .map_err(|e: ParameterType| format!("Unexpected parameter type {:?}", e))
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
                x_plus: parameters[0].clone().into_integer().unwrap(),
                y_plus: parameters[1].clone().into_integer().unwrap(),
            },
            15 => MoveCommand::Wait {
                time: parameters[0].clone().into_integer().unwrap(),
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
                switch_id: parameters[0].clone().into_integer().unwrap() as usize,
            },
            28 => SwitchOFF {
                switch_id: parameters[0].clone().into_integer().unwrap() as usize,
            },
            29 => ChangeSpeed {
                speed: parameters[0].clone().into_integer().unwrap() as usize,
            },
            30 => ChangeFreq {
                freq: parameters[0].clone().into_integer().unwrap() as usize,
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
                character_name: parameters[0].clone().into_string().unwrap(),
                character_hue: parameters[1].clone().into_integer().unwrap(),
                direction: parameters[2].clone().into_integer().unwrap(),
                pattern: parameters[3].clone().into_integer().unwrap(),
            },
            42 => ChangeOpacity {
                opacity: parameters[0].clone().into_integer().unwrap(),
            },
            43 => ChangeBlend {
                blend: parameters[0].clone().into_integer().unwrap(),
            },
            44 => Self::PlaySE {
                file: parameters[0].clone().into_audio_file().unwrap(),
            },
            45 => Self::Script {
                text: parameters[0].clone().into_string().unwrap(),
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

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Retain = 0,
    Down = 2,
    Left = 4,
    Right = 6,
    Up = 8,
}

#[allow(missing_docs)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BlendType {
    Normal = 0,
    Additive = 1,
    Subtractive = 2,
}
