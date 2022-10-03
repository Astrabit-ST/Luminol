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
use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};
use std::{collections::HashMap, sync::Arc};

use crate::filesystem::Filesystem;

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    tilesets: Mutex<Option<Vec<rpg::Tileset>>>,
    mapinfos: Mutex<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: Mutex<HashMap<i32, rpg::Map>>,
}

impl DataCache {
    pub fn load(&self, filesystem: &Filesystem) -> Result<(), String> {
        *self.mapinfos.lock() = Some(
            filesystem
                .read_data("MapInfos.ron")
                .map_err(|_| "Failed to read MapInfos")?,
        );

        *self.tilesets.lock() = Some(
            filesystem
                .read_data("Tilesets.ron")
                .map_err(|_| "Failed to read Tilesets")?,
        );

        self.maps.lock().clear();
        Ok(())
    }

    pub fn load_map(
        &self,
        filesystem: Arc<Filesystem>,
        id: i32,
    ) -> Result<MappedMutexGuard<'_, rpg::Map>, String> {
        let mut map_lock = self.maps.lock();
        let has_map = map_lock.contains_key(&id);
        if !has_map {
            let map = filesystem
                .read_data(&format!("Map{:0>3}.ron", id))
                .map_err(|_| "Failed to load map")?;
            map_lock.insert(id, map);
        }
        Ok(MutexGuard::map(map_lock, |m| m.get_mut(&id).unwrap()))
    }

    pub fn map_infos(&self) -> MutexGuard<'_, Option<HashMap<i32, rpg::MapInfo>>> {
        self.mapinfos.lock()
    }

    pub fn tilesets(&self) -> MutexGuard<'_, Option<Vec<rpg::Tileset>>> {
        self.tilesets.lock()
    }

    pub fn save(&self, filesystem: &Filesystem) -> Result<(), String> {
        // Write map data and clear map cache.
        for (id, map) in self.maps.lock().drain() {
            filesystem
                .save_data(&format!("Map{:0>3}.ron", id), &map)
                .map_err(|_| "Failed to write Map data")?
        }
        if let Some(tilesets) = self.tilesets.lock().as_ref() {
            filesystem
                .save_data("Tilesets.ron", tilesets)
                .map_err(|_| "Failed to write Tileset data")?;
        }
        if let Some(mapinfos) = self.mapinfos.lock().as_ref() {
            filesystem
                .save_data("MapInfos.ron", mapinfos)
                .map_err(|_| "Failed to write MapInfos data")?;
        }
        Ok(())
    }
}
