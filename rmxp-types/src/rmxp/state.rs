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

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::State")]
pub struct State {
    pub id: i32,
    pub name: String,
    pub animation_id: i32,
    pub restriction: i32,
    pub nonresistance: bool,
    pub zero_hp: bool,
    pub cant_get_exp: bool,
    pub cant_evade: bool,
    pub slip_damage: bool,
    pub rating: i32,
    pub hit_rate: i32,
    pub maxhp_rate: i32,
    pub maxsp_rate: i32,
    pub str_rate: i32,
    pub dex_rate: i32,
    pub agi_rate: i32,
    pub int_rate: i32,
    pub atk_rate: i32,
    pub pdef_rate: i32,
    pub mdef_rate: i32,
    pub eva: i32,
    pub battle_only: bool,
    pub hold_turn: i32,
    pub auto_release_prob: i32,
    pub shock_release_prob: i32,
    pub guard_element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}
