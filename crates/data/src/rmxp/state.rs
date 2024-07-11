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
use crate::{id_alox, id_serde, id_vec_alox, id_vec_serde, optional_id_alox, optional_id_serde};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::State")]
pub struct State {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub animation_id: Option<usize>,
    pub restriction: Restriction,
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
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub guard_element_set: Vec<usize>,
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub plus_state_set: Vec<usize>,
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub minus_state_set: Vec<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Restriction {
    #[default]
    None = 0,
    #[strum(to_string = "Can't use magic")]
    NoMagic = 1,
    #[strum(to_string = "Always attack enemies")]
    AttackEnemies = 2,
    #[strum(to_string = "Always attack allies")]
    AttackAllies = 3,
    #[strum(to_string = "Can't move")]
    NoMove = 4,
}
