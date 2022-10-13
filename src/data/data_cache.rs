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

use ron::ser::{to_string_pretty, PrettyConfig};

use crate::data::rmxp_structs::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::filesystem::Filesystem;

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    system: RefCell<Option<rpg::system::System>>,
    tilesets: RefCell<Option<Vec<rpg::Tileset>>>,
    mapinfos: RefCell<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: RefCell<HashMap<i32, rpg::Map>>,
}

impl DataCache {
    /// Load all data required when opening a project.
    pub async fn load(&self, filesystem: &Filesystem) -> Result<(), String> {
        *self.system.borrow_mut() = Some(
            filesystem
                .read_data("System.ron")
                .await
                .map_err(|s| format!("Failed to read System: {}", s))?,
        );

        *self.mapinfos.borrow_mut() = Some(
            filesystem
                .read_data("MapInfos.ron")
                .await
                .map_err(|s| format!("Failed to read MapInfos: {}", s))?,
        );

        *self.tilesets.borrow_mut() = Some(
            filesystem
                .read_data("Tilesets.ron")
                .await
                .map_err(|s| format!("Failed to read Tilesets: {}", s))?,
        );

        self.maps.borrow_mut().clear();
        Ok(())
    }

    /// Load a map.
    pub async fn load_map(
        &self,
        filesystem: &'static Filesystem,
        id: i32,
    ) -> Result<RefMut<'_, rpg::Map>, String> {
        let has_map = self.maps.borrow().contains_key(&id);
        if !has_map {
            let map = filesystem
                .read_data(&format!("Map{:0>3}.ron", id))
                .await
                .map_err(|_| "Failed to load map")?;
            self.maps.borrow_mut().insert(id, map);
        }
        Ok(RefMut::map(self.maps.borrow_mut(), |m| {
            m.get_mut(&id).unwrap()
        }))
    }

    /// Get a map that has been loaded. This function is not async unlike [`Self::load_map`].
    /// #Panics
    /// Will panic iftthe map has not been loaded already.
    pub fn get_map(&self, id: i32) -> RefMut<'_, rpg::Map> {
        RefMut::map(self.maps.borrow_mut(), |maps| maps.get_mut(&id).unwrap())
    }

    /// Get MapInfos.
    pub fn map_infos(&self) -> RefMut<'_, Option<HashMap<i32, rpg::MapInfo>>> {
        self.mapinfos.borrow_mut()
    }

    /// Get Tilesets.
    pub fn tilesets(&self) -> RefMut<'_, Option<Vec<rpg::Tileset>>> {
        self.tilesets.borrow_mut()
    }

    /// Get system.
    pub fn system(&self) -> RefMut<'_, Option<rpg::system::System>> {
        self.system.borrow_mut()
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub async fn save(&self, filesystem: &Filesystem) -> Result<(), String> {
        // Write map data and clear map cache.
        // We serialize all of these first before writing them to the disk to avoid bringing a refcell across an await.
        // A RwLock may be used in the future to solve this, though.
        let maps_strs: HashMap<_, _> = self
            .maps
            .borrow_mut()
            .drain()
            .map(|(id, map)| {
                (
                    id,
                    to_string_pretty(&map, PrettyConfig::default()).map_err(|e| e.to_string()),
                )
            })
            .collect();

        for (id, map) in maps_strs {
            filesystem
                .save_data(&format!("Map{:0>3}.ron", id), &map?)
                .await
                .map_err(|_| "Failed to write Map data")?
        }

        let tilesets_str = self
            .tilesets
            .borrow()
            .as_ref()
            .map(|t| to_string_pretty(&t, PrettyConfig::default()).map_err(|e| e.to_string()));
        if let Some(tilesets_str) = tilesets_str {
            filesystem
                .save_data("Tilesets.ron", &tilesets_str?)
                .await
                .map_err(|_| "Failed to write Tileset data")?;
        }

        let mapinfos_str = self
            .mapinfos
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, PrettyConfig::default()).map_err(|e| e.to_string()));
        if let Some(mapinfos_str) = mapinfos_str {
            filesystem
                .save_data("MapInfos.ron", &mapinfos_str?)
                .await
                .map_err(|_| "Failed to write MapInfos data")?;
        }
        Ok(())
    }
}
