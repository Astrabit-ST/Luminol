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
use crate::{id, optional_id, rpg::EventCommand};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Troop")]
pub struct Troop {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    pub members: Vec<Member>,
    pub pages: Vec<TroopPage>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Troop::Member")]
pub struct Member {
    #[serde(with = "id")]
    pub enemy_id: usize,
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
    pub immortal: bool,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Troop::Page")]
pub struct TroopPage {
    pub condition: TroopCondition,
    pub span: i32,
    pub list: Vec<EventCommand>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Troop::Page::Condition")]
pub struct TroopCondition {
    pub turn_valid: bool,
    pub enemy_valid: bool,
    pub actor_valid: bool,
    pub switch_valid: bool,
    pub turn_a: i32,
    pub turn_b: i32,
    pub enemy_index: usize,
    pub enemy_hp: i32,
    #[serde(with = "optional_id")]
    pub actor_id: Option<usize>,
    pub actor_hp: i32,
    #[serde(with = "optional_id")]
    pub switch_id: Option<usize>,
}
