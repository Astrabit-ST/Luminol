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
use crate::{optional_path, Path, Table2};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Actor")]
pub struct Actor {
    pub id: i32,
    pub name: String,
    pub class_id: i32,
    pub initial_level: i32,
    pub final_level: i32,
    pub exp_basis: i32,
    pub exp_inflation: i32,
    #[serde(with = "optional_path")]
    pub character_name: Path,
    pub character_hue: i32,
    #[serde(with = "optional_path")]
    pub battler_name: Path,
    pub battler_hue: i32,
    pub parameters: Table2,
    pub weapon_id: i32,
    pub armor1_id: i32,
    pub armor2_id: i32,
    pub armor3_id: i32,
    pub armor4_id: i32,
    pub weapon_fix: bool,
    pub armor1_fix: bool,
    pub armor2_fix: bool,
    pub armor3_fix: bool,
    pub armor4_fix: bool,
}
