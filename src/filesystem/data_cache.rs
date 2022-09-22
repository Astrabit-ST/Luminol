// Copyright (C) 2022 Lily Lyons
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

use crate::data::rmxp_structs::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use super::Filesystem;

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    tilesets: RefCell<Option<Vec<rpg::Tileset>>>,
    mapinfos: RefCell<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: RefCell<HashMap<i32, rpg::Map>>,
}

impl DataCache {
    pub fn load(&self, filesystem: &Filesystem) {
        *self.mapinfos.borrow_mut() = Some(
            filesystem
                .read_data("MapInfos.ron")
                .expect("Failed to load Map Infos"),
        );

        *self.tilesets.borrow_mut() = Some(
            filesystem
                .read_data("Tilesets.ron")
                .expect("Failed to load Tilesets"),
        );

        self.maps.borrow_mut().clear();
    }

    pub fn load_map(&self, filesystem: &Filesystem, id: i32) -> RefMut<'_, rpg::Map> {
        RefMut::map(self.maps.borrow_mut(), |maps| {
            maps.entry(id).or_insert_with(|| {
                filesystem
                    .read_data(&format!("Map{:0>3}.ron", id))
                    .expect("Failed to load map")
            })
        })
    }

    pub fn map_infos(&self) -> RefMut<'_, Option<HashMap<i32, rpg::MapInfo>>> {
        self.mapinfos.borrow_mut()
    }

    pub fn tilesets(&self) -> RefMut<'_, Option<Vec<rpg::Tileset>>> {
        self.tilesets.borrow_mut()
    }

    pub fn save(&self, filesystem: &Filesystem) {
        // Write map data and clear map cache.
        for (id, map) in self.maps.borrow_mut().drain() {
            filesystem
                .save_data(&format!("Map{:0>3}.ron", id), &map)
                .expect("Failed to write Map data");
        }
        if let Some(tilesets) = self.tilesets.borrow_mut().as_ref() {
            filesystem
                .save_data("Tilesets.ron", tilesets)
                .expect("Failed to write Tileset data");
        }
        if let Some(mapinfos) = self.mapinfos.borrow_mut().as_ref() {
            filesystem
                .save_data("MapInfos.ron", mapinfos)
                .expect("Failed to write MapInfos data");
        }
    }
}
