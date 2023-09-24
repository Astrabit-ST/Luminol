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
pub use crate::{id, id_vec, Table1};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Class")]
pub struct Class {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    pub position: Position,
    #[serde(with = "id_vec")]
    pub weapon_set: Vec<usize>,
    #[serde(with = "id_vec")]
    pub armor_set: Vec<usize>,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub learnings: Vec<Learning>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Class::Learning")]
pub struct Learning {
    pub level: i32,
    #[serde(with = "id")]
    pub skill_id: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
pub enum Position {
    #[default]
    Front = 0,
    Middle = 1,
    Rear = 2,
}
