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
#[marshal(class = "RPG::Weapon")]
pub struct Weapon {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub icon_name: Path,
    pub description: String,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub animation1_id: Option<usize>,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub animation2_id: Option<usize>,
    pub price: i32,
    pub atk: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
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
