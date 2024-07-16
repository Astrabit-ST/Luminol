// Copyright (C) 2024 Melody Madeline Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use alox_48::Value;

use crate::rgss_structs::{Color, Tone};
use crate::shared::{AudioFile, MoveCommand, MoveRoute};

#[derive(Debug, Clone, PartialEq, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(from = "alox_48::Value", into = "alox_48::Value")] // TODO make this serde compatible
#[allow(missing_docs)]
pub enum ParameterType {
    Integer(i32),
    String(String),
    Color(Color),
    Tone(Tone),
    AudioFile(AudioFile),
    Float(f64),
    MoveRoute(MoveRoute),
    MoveCommand(MoveCommand),
    Array(Vec<ParameterType>),
    Bool(bool),

    #[default]
    None,
}

// FIXME this really should be try_from and try_into
impl From<alox_48::Value> for ParameterType {
    fn from(value: alox_48::Value) -> Self {
        match value {
            Value::Nil => Self::None,
            Value::Integer(v) => Self::Integer(v),
            Value::Float(v) => Self::Float(v),
            Value::String(v) => Self::String(String::from_utf8(v.data).unwrap()),
            Value::Array(v) => Self::Array(v.into_iter().map(|v| v.into()).collect()),
            Value::Bool(v) => Self::Bool(v),
            Value::Userdata(userdata) => match userdata.class.as_str() {
                "Color" => Self::Color(Color::from(userdata)),
                "Tone" => Self::Tone(Tone::from(userdata)),
                _ => panic!("Unsupported userdata type: {:#?}", userdata),
            },
            Value::Object(alox_48::Object { ref class, .. }) => match class.as_str() {
                "RPG::AudioFile" => Self::AudioFile(alox_48::from_value(&value).unwrap()),
                "RPG::MoveRoute" => Self::MoveRoute(alox_48::from_value(&value).unwrap()),
                "RPG::MoveCommand" => Self::MoveCommand(alox_48::from_value(&value).unwrap()),
                _ => panic!("Unsupported object type: {:#?}", value),
            },
            Value::Instance(i) => (*i.value).into(),
            _ => panic!("Unsupported value type: {:#?}", value),
        }
    }
}

impl From<ParameterType> for alox_48::Value {
    fn from(value: ParameterType) -> Self {
        match value {
            ParameterType::None => Value::Nil,
            ParameterType::Integer(v) => Value::Integer(v),
            ParameterType::Float(v) => Value::Float(v),
            ParameterType::String(v) => Value::String(v.into()),
            ParameterType::Array(v) => Value::Array(v.into_iter().map(|v| v.into()).collect()),
            ParameterType::Bool(v) => Value::Bool(v),
            ParameterType::Color(v) => Value::Userdata(v.into()),
            ParameterType::Tone(v) => Value::Userdata(v.into()),
            ParameterType::AudioFile(v) => alox_48::to_value(v).unwrap(),
            ParameterType::MoveRoute(v) => alox_48::to_value(v).unwrap(),
            ParameterType::MoveCommand(v) => alox_48::to_value(v).unwrap(),
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

                #[doc = "Converts this parameter into a `" $name "` if it is not already, replacing it with the provided default."]
                pub fn [<into_ $name:lower _with>](&mut self, default: $type) -> &mut $type {
                    match self {
                        ParameterType::$name(ref mut v) => v,
                        _ => {
                            #[cfg(debug_assertions)]
                            eprintln!(concat!("Parameter was of wrong type, expected ", stringify!($name), " got {:#?} instead"), self);

                            *self = ParameterType::$name(default);

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
    Float, f64,
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

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn new_none() -> Self {
        Self::None
    }
}

impl From<()> for ParameterType {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<&str> for ParameterType {
    fn from(value: &str) -> Self {
        Self::String(value.to_string())
    }
}
