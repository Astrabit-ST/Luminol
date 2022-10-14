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

#[allow(unused_imports)]
use super::rmxp_structs::rpg;

// FIXME: NOT ALL OF THESE ARE KNOWN

/// An enum representing event commands.
#[derive(Debug, Clone)]
pub enum Command {
    /// Show text, id 101
    /// Fields: (text: String)
    Text(String),
    /// Show Choices, id 102
    /// Fields: (choices: Vec<String>, choice_type: i32)
    Choices(Vec<String>, i32),
    /// When, id 402
    /// Fields (???: i32)
    When(i32),
    /// When Cancel, id 403
    /// Fields (???: i32)
    WhenCancel(i32),
    /// Invalid, invalid command ID
    Invalid,
}

impl serde::Serialize for Command {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_i8(0)
    }
}

impl<'de> serde::Deserialize<'de> for Command {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CommandVisitor)
    }
}

struct CommandVisitor;

impl<'de> serde::de::Visitor<'de> for CommandVisitor {
    type Value = Command;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("just compile bro")
    }
}
