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
use crate::{
    id_alox, id_serde, id_vec_alox, id_vec_serde, optional_id_alox, optional_id_serde,
    optional_path_alox, optional_path_serde, Path,
};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Armor")]
pub struct Armor {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub icon_name: Path,
    pub description: String,
    pub kind: Kind,
    #[serde(with = "optional_id_serde")]
    #[marshal(with = "optional_id_alox")]
    pub auto_state_id: Option<usize>,
    pub price: i32,
    pub pdef: i32,
    pub mdef: i32,
    pub eva: i32,
    pub str_plus: i32,
    pub dex_plus: i32,
    pub agi_plus: i32,
    pub int_plus: i32,
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub guard_element_set: Vec<usize>,
    #[serde(with = "id_vec_serde")]
    #[marshal(with = "id_vec_alox")]
    pub guard_state_set: Vec<usize>,
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
pub enum Kind {
    #[default]
    Shield = 0,
    Helmet = 1,
    #[strum(to_string = "Body Armor")]
    BodyArmor = 2,
    Accessory = 3,
}
