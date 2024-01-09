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
pub use crate::{id, id_vec, optional_id, optional_path, rpg::AudioFile, Path};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename = "RPG::Item")]
pub struct Item {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub scope: Scope,
    pub occasion: Occasion,
    #[serde(with = "optional_id")]
    pub animation1_id: Option<usize>,
    #[serde(with = "optional_id")]
    pub animation2_id: Option<usize>,
    pub menu_se: AudioFile,
    #[serde(with = "optional_id")]
    pub common_event_id: Option<usize>,
    pub price: i32,
    pub consumable: bool,
    pub parameter_type: ParameterType,
    pub parameter_points: i32,
    pub recover_hp_rate: i32,
    pub recover_hp: i32,
    // These fields are missing in rmxp data *sometimes*.
    // Why? Who knows!
    #[serde(default)]
    pub recover_sp_rate: i32,
    #[serde(default)]
    pub recover_sp: i32,
    pub hit: i32,
    pub pdef_f: i32,
    pub mdef_f: i32,
    pub variance: i32,
    #[serde(with = "id_vec")]
    pub element_set: Vec<usize>,
    #[serde(with = "id_vec")]
    pub plus_state_set: Vec<usize>,
    #[serde(with = "id_vec")]
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
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
pub enum Scope {
    #[default]
    None = 0,
    #[strum(to_string = "One Enemy")]
    OneEnemy = 1,
    #[strum(to_string = "All Enemies")]
    AllEnemies = 2,
    #[strum(to_string = "One Ally")]
    OneAlly = 3,
    #[strum(to_string = "All Allies")]
    AllAllies = 4,
    #[strum(to_string = "One Ally (HP 0)")]
    OneAllyHP0 = 5,
    #[strum(to_string = "All Allies (HP 0)")]
    AllAlliesHP0 = 6,
    #[strum(to_string = "The User")]
    User = 7,
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
pub enum Occasion {
    #[default]
    Always = 0,
    #[strum(to_string = "Only in battle")]
    OnlyBattle = 1,
    #[strum(to_string = "Only from the menu")]
    OnlyMenu = 2,
    Never = 3,
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
pub enum ParameterType {
    #[default]
    None = 0,
    #[strum(to_string = "Max HP")]
    MaxHP = 1,
    #[strum(to_string = "Max SP")]
    MaxSP = 2,
    #[strum(to_string = "Strength")]
    Str = 3,
    #[strum(to_string = "Dexterity")]
    Dex = 4,
    #[strum(to_string = "Agility")]
    Agi = 5,
    #[strum(to_string = "Intelligence")]
    Int = 6,
}
