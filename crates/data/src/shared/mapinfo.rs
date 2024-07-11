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

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::MapInfo")]
pub struct MapInfo {
    pub name: String,
    // because mapinfos is stored in a hash, we dont actually need to modify values! this can just stay as a usize.
    // it would be slightly more accurate to store this as an option, but no other values (off the top of my head) are like this. maybe event tile ids.
    // I'll need to think on this a bit.
    pub parent_id: usize,
    pub order: i32,
    pub expanded: bool,
    pub scroll_x: i32,
    pub scroll_y: i32,
}

impl PartialOrd for MapInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for MapInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}
