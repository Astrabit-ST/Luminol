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

use crate::rgss_structs::{Color, Tone};
use crate::rpg::{AudioFile, MoveCommand, MoveRoute};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
#[allow(missing_docs)]
#[serde(from = "alox_48::Value")]
#[serde(into = "alox_48::Value")]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(AudioFile),
    Float(f32),
    MoveRoute(MoveRoute),
    MoveCommand(MoveCommand),
    Array(Vec<ParameterType>),
    Bool(bool),

    #[default]
    None,
}

impl From<alox_48::Value> for ParameterType {
    fn from(value: alox_48::Value) -> Self {
        match value {
            alox_48::Value::Integer(i) => Self::Integer(i as _),
            alox_48::Value::String(str) => Self::String(str.to_string_lossy().into_owned()),
            alox_48::Value::Object(obj) if obj.class == "RPG::AudioFile" => {
                Self::AudioFile(obj.into())
            }
            alox_48::Value::Object(obj) if obj.class == "RPG::MoveRoute" => {
                Self::MoveRoute(obj.into())
            }
            alox_48::Value::Object(obj) if obj.class == "RPG::MoveCommand" => {
                Self::MoveCommand(obj.into())
            }
            alox_48::Value::Float(f) => Self::Float(f as _),
            alox_48::Value::Array(ary) => Self::Array(ary.into_iter().map(|v| v.into()).collect()),
            alox_48::Value::Bool(b) => Self::Bool(b),
            alox_48::Value::Userdata(data) if data.class == "Color" => {
                Self::Color(Color::from(data))
            }
            alox_48::Value::Userdata(data) if data.class == "Tone" => Self::Tone(Tone::from(data)),
            _ => panic!("Unexpected type {value:#?}"),
        }
    }
}

impl From<ParameterType> for alox_48::Value {
    fn from(value: ParameterType) -> Self {
        match value {
            ParameterType::Integer(i) => alox_48::Value::Integer(i as _),
            ParameterType::String(s) => alox_48::Value::String(s.into()),
            ParameterType::Color(c) => c.into(),
            ParameterType::Tone(t) => t.into(),
            ParameterType::Float(f) => alox_48::Value::Float(f as _),
            ParameterType::Array(a) => {
                alox_48::Value::Array(a.into_iter().map(Into::into).collect())
            }
            ParameterType::Bool(b) => alox_48::Value::Bool(b),

            ParameterType::MoveRoute(r) => alox_48::Value::Object(r.into()),
            ParameterType::MoveCommand(c) => alox_48::Value::Object(c.into()),
            ParameterType::AudioFile(a) => alox_48::Value::Object(a.into()),

            ParameterType::None => alox_48::Value::Nil,
        }
    }
}

macro_rules! variant_impl {

    ($($name:ident, $type:ty),*) => {

        $(paste::paste! {
            impl ParameterType {
                #[doc = "Converts this parameter into a `" $name "` if it is not already, and returns the contained value."]
                pub fn [<into_ $name:lower>](&mut self) -> &mut $type {
                    match self {
                        ParameterType::$name(ref mut v) => v,
                        _ => {
                            #[cfg(debug_assertions)]
                            eprintln!(concat!("Parameter was of wrong type, expected ", stringify!($name), " got {:#?} instead"), self);

                            *self = ParameterType::$name(Default::default());

                            match self {
                                ParameterType::$name(ref mut v) => v,
                                _ => unreachable!(),
                            }
                        }
                    }
                }

                #[doc = "Gets this parameter as a reference to `" $name "` and returns None if the parameter was not a `" $name "`."]
                pub fn [<as_ $name:lower>](&self) -> Option<&$type> {
                    match self {
                        ParameterType::$name(ref v) => Some(v),
                        _ => None
                    }
                }

                #[doc = "Gets this parameter as a mutable reference to `" $name "` and returns None if the parameter was not a `" $name "`."]
                pub fn [<as_ $name:lower _mut>](&mut self) -> Option<&mut $type> {
                    match self {
                        ParameterType::$name(ref mut v) => Some(v),
                        _ => None
                    }
                }

                pub fn [<is_ $name:lower>](&self) -> bool {
                    matches!(self, ParameterType::$name(_))
                }

                pub fn [<new_ $name:lower>](v: $type) -> Self {
                    ParameterType::$name(v)
                }
            }

            impl From<$type> for ParameterType {
                fn from(v: $type) -> Self {
                    ParameterType::$name(v)
                }
            }

            impl TryFrom<ParameterType> for $type {
                type Error = ParameterType;

                fn try_from(v: ParameterType) -> Result<Self, Self::Error> {
                    match v {
                        ParameterType::$name(v) => Ok(v),
                        v => Err(v)
                    }
                }
            }
        })*
    };
}

variant_impl! {
    Integer, i32,
    String, String,
    Color, Color,
    Tone, Tone,
    AudioFile, AudioFile,
    Float, f32,
    MoveRoute, MoveRoute,
    MoveCommand, MoveCommand,
    Array, Vec<ParameterType>,
    Bool, bool
}

impl ParameterType {
    pub fn truthy(&self) -> bool {
        !self.falsey()
    }

    pub fn falsey(&self) -> bool {
        matches!(self, Self::None | Self::Bool(false) | Self::Integer(0))
    }
}
