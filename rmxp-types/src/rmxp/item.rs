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
pub use crate::{optional_path, rpg::AudioFile, Path};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone)]
#[serde(rename = "RPG::Item")]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub icon_name: String,
    pub description: String,
    pub scope: i32,
    pub occasion: i32,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub menu_se: AudioFile,
    pub common_event_id: i32,
    pub price: i32,
    pub consumable: bool,
    pub parameter_type: i32,
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
    pub element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}
