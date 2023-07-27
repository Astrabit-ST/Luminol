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
pub use crate::{id, optional_id, optional_path, Path, Table1};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Enemy")]
pub struct Enemy {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path")]
    pub battler_name: Path,
    pub battler_hue: i32,
    pub maxhp: i32,
    pub maxsp: i32,
    pub str: i32,
    pub dex: i32,
    pub agi: i32,
    pub int: i32,
    pub atk: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub eva: i32,
    #[serde(with = "optional_id")]
    pub animation1_id: Option<usize>,
    #[serde(with = "optional_id")]
    pub animation2_id: Option<usize>,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub actions: Vec<Action>,
    pub exp: i32,
    // FIXME: make optional
    pub gold: i32,
    #[serde(with = "optional_id")]
    pub item_id: Option<usize>,
    #[serde(with = "optional_id")]
    pub weapon_id: Option<usize>,
    #[serde(with = "optional_id")]
    pub armor_id: Option<usize>,
    pub treasure_prob: i32,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Enemy::Action")]
pub struct Action {
    pub kind: i32,
    pub basic: i32,
    #[serde(with = "optional_id")]
    pub skill_id: Option<usize>,
    pub condition_turn_a: i32,
    pub condition_turn_b: i32,
    pub condition_hp: i32,
    pub condition_level: i32,
    #[serde(with = "optional_id")]
    pub condition_switch_id: Option<usize>,
    pub rating: i32,
}
