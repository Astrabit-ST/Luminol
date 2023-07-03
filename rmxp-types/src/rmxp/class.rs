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
pub use crate::Table1;

#[derive(Default, Debug, serde:: Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Class")]
pub struct Class {
    pub id: i32,
    pub name: String,
    pub position: i32,
    pub weapon_set: Vec<i32>,
    pub armor_set: Vec<i32>,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub learnings: Vec<Learning>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Class::Learning")]
pub struct Learning {
    pub level: i32,
    pub skill_id: i32,
}
