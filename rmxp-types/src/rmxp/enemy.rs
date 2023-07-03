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
pub use crate::{optional_path, Path, Table1};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Enemy")]
pub struct Enemy {
    pub id: i32,
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
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub element_ranks: Table1,
    pub state_ranks: Table1,
    pub actions: Vec<Action>,
    pub exp: i32,
    pub gold: i32,
    pub item_id: i32,
    pub weapon_id: i32,
    pub armor_id: i32,
    pub treasure_prob: i32,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Enemy::Action")]
pub struct Action {
    pub kind: i32,
    pub basic: i32,
    pub skill_id: i32,
    pub condition_turn_a: i32,
    pub condition_turn_b: i32,
    pub condition_hp: i32,
    pub condition_level: i32,
    pub condition_switch_id: i32,
    pub rating: i32,
}
