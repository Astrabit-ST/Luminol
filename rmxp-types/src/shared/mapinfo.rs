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
use crate::id;

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[serde(rename = "RPG::MapInfo")]
pub struct MapInfo {
    pub name: String,
    #[serde(with = "id")]
    pub parent_id: usize,
    pub order: i32,
    pub expanded: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

impl PartialOrd for MapInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.order.partial_cmp(&other.order)
    }
}

impl Ord for MapInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}
