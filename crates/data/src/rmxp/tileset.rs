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

use crate::{id_alox, id_serde, optional_path_alox, optional_path_serde, BlendMode, Path, Table1};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Tileset")]
pub struct Tileset {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub tileset_name: Path,
    pub autotile_names: Vec<String>,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub panorama_name: Path,
    pub panorama_hue: i32,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub fog_name: Path,
    pub fog_hue: i32,
    pub fog_opacity: i32,
    pub fog_blend_type: BlendMode,
    pub fog_zoom: i32,
    pub fog_sx: i32,
    pub fog_sy: i32,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub battleback_name: Path,
    pub passages: Table1,
    pub priorities: Table1,
    pub terrain_tags: Table1,
}
