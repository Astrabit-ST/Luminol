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
use crate::{id, id_vec, optional_path, Path};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Armor")]
pub struct Armor {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path")]
    pub icon_name: Path,
    pub description: String,
    pub kind: i32,
    #[serde(with = "id")]
    pub auto_state_id: usize,
    pub price: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub eva: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
    #[serde(with = "id_vec")]
    pub guard_element_set: Vec<usize>,
    #[serde(with = "id_vec")]
    pub guard_state_set: Vec<usize>,
}
