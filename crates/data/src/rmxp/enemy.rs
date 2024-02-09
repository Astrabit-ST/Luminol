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

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Enemy::Action")]
pub struct Action {
    pub kind: Kind,
    pub basic: Basic,
    #[serde(with = "id")]
    pub skill_id: usize,
    pub condition_turn_a: i32,
    pub condition_turn_b: i32,
    pub condition_hp: i32,
    pub condition_level: i32,
    #[serde(with = "optional_id")]
    pub condition_switch_id: Option<usize>,
    pub rating: i32,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            kind: Kind::default(),
            basic: Basic::default(),
            skill_id: 0,
            condition_turn_a: 0,
            condition_turn_b: 1,
            condition_hp: 100,
            condition_level: 1,
            condition_switch_id: None,
            rating: 5,
        }
    }
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
pub enum Kind {
    #[default]
    Basic = 0,
    Skill = 1,
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
pub enum Basic {
    #[default]
    Attack = 0,
    Defend = 1,
    Escape = 2,
    #[strum(to_string = "Do Nothing")]
    DoNothing = 3,
}
