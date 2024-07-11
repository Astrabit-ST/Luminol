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

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Skill")]
pub struct Skill {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub icon_name: Path,
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
    pub sp_cost: i32,
    pub power: i32,
    pub atk_f: i32,
    pub eva_f: i32,
    pub str_f: i32,
    pub dex_f: i32,
    pub agi_f: i32,
    pub int_f: i32,
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
