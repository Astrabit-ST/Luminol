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

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Weapon")]
pub struct Weapon {
    pub id: i32,
    pub name: String,
    #[serde(with = "optional_path")]
    pub icon_name: Path,
    pub description: String,
    pub animation1_id: i32,
    pub animation2_id: i32,
    pub price: i32,
    pub atk: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
    pub element_set: Vec<i32>,
    pub plus_state_set: Vec<i32>,
    pub minus_state_set: Vec<i32>,
}
