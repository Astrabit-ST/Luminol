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
use crate::{id_alox, id_serde, optional_id_alox, optional_id_serde, rpg::EventCommand};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Troop")]
pub struct Troop {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    pub members: Vec<Member>,
    pub pages: Vec<Page>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Troop::Member")]
pub struct Member {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub enemy_id: usize,
    pub x: i32,
    pub y: i32,
    pub hidden: bool,
    pub immortal: bool,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Troop::Page")]
pub struct Page {
    pub condition: Condition,
    pub span: i32,
    pub list: Vec<EventCommand>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Troop::Page::Condition")]
pub struct Condition {
    pub turn_valid: bool,
    pub enemy_valid: bool,
    pub actor_valid: bool,
    pub switch_valid: bool,
    pub turn_a: i32,
    pub turn_b: i32,
    pub enemy_index: usize,
    pub enemy_hp: i32,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub actor_id: Option<usize>,
    pub actor_hp: i32,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub switch_id: Option<usize>,
}
