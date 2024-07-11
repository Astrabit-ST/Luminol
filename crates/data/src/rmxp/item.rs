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
pub use crate::{
    id_alox, id_serde, id_vec_alox, id_vec_serde, optional_id_alox, optional_id_serde,
    optional_path_alox, optional_path_serde, rpg::AudioFile, Path,
};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Item")]
pub struct Item {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub icon_name: Option<camino::Utf8PathBuf>,
    pub description: String,
    pub scope: crate::rpg::Scope,
    pub occasion: crate::rpg::Occasion,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub animation1_id: Option<usize>,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub animation2_id: Option<usize>,
    pub menu_se: AudioFile,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub common_event_id: Option<usize>,
    pub price: i32,
    pub consumable: bool,
    pub parameter_type: ParameterType,
    pub parameter_points: i32,
    pub recover_hp_rate: i32,
    pub recover_hp: i32,
    // These fields are missing in rmxp data *sometimes*.
    // Why? Who knows!
    #[marshal(default)]
    #[serde(default)]
    pub recover_sp_rate: i32,
    #[marshal(default)]
    #[serde(default)]
    pub recover_sp: i32,
    pub hit: i32,
    pub pdef_f: i32,
    pub mdef_f: i32,
    pub variance: i32,
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub element_set: Vec<usize>,
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
pub enum ParameterType {
    #[default]
    None = 0,
    #[strum(to_string = "Max HP")]
    MaxHP = 1,
    #[strum(to_string = "Max SP")]
    MaxSP = 2,
    #[strum(to_string = "STR")]
    Str = 3,
    #[strum(to_string = "DEX")]
    Dex = 4,
    #[strum(to_string = "AGI")]
    Agi = 5,
    #[strum(to_string = "INT")]
    Int = 6,
}

impl ParameterType {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}
