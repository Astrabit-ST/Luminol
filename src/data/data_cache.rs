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

use once_cell::sync::Lazy;
use ron::ser::{to_string_pretty, PrettyConfig};

use crate::data::rmxp_structs::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::filesystem::Filesystem;

use super::{
    config::LocalConfig,
    rgss_structs::{Table1, Table3},
    rmxp_structs::intermediate,
};

static CONFIG: Lazy<PrettyConfig> = Lazy::new(|| {
    PrettyConfig::new()
        .struct_names(true)
        .separate_tuple_members(true)
});

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    actors: RefCell<Option<Vec<rpg::Actor>>>,
    animations: RefCell<Option<Vec<rpg::animation::Animation>>>,
    system: RefCell<Option<rpg::system::System>>,
    tilesets: RefCell<Option<Vec<rpg::Tileset>>>,
    mapinfos: RefCell<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: RefCell<HashMap<i32, rpg::Map>>,
    common_events: RefCell<Option<Vec<rpg::CommonEvent>>>,
    scripts: RefCell<Option<Vec<intermediate::Script>>>,
    items: RefCell<Option<Vec<rpg::Item>>>,
    config: RefCell<Option<LocalConfig>>,
}

impl DataCache {
    /// Load all data required when opening a project.
    pub async fn load(&self, filesystem: &Filesystem) -> Result<(), String> {
        let config = filesystem
            .read_data(".luminol")
            .await
            .ok()
            .unwrap_or_default();

        *self.config.borrow_mut() = Some(config);

        *self.actors.borrow_mut() = Some(
            filesystem
                .read_data("Actors.ron")
                .await
                .map_err(|s| format!("Failed to read Actors: {}", s))?,
        );

        *self.animations.borrow_mut() = Some(
            filesystem
                .read_data("Animations.ron")
                .await
                .map_err(|s| format!("Failed to read Animations: {}", s))?,
        );

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

        *self.common_events.borrow_mut() = Some(
            filesystem
                .read_data("CommonEvents.ron")
                .await
                .map_err(|s| format!("Failed to read Common Events: {}", s))?,
        );

        *self.items.borrow_mut() = Some(
            filesystem
                .read_data("Items.ron")
                .await
                .map_err(|s| format!("Failed to read Items: {}", s))?,
        );

        let mut scripts = filesystem.read_data("xScripts.ron").await;

        if let Err(e) = scripts {
            println!("Attempted loading xScripts failed with {}", e);

            scripts = filesystem.read_data("Scripts.ron").await;
        } else {
            self.config.borrow_mut().as_mut().unwrap().scripts_path = "xScripts".to_string();
        }

        *self.scripts.borrow_mut() = Some(
            scripts.map_err(|s| format!("Failed to read Scripts (tried xScripts first): {}", s))?,
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
                .map_err(|e| format!("Failed to load map: {}", e))?;
            self.maps.borrow_mut().insert(id, map);
        }
        Ok(RefMut::map(self.maps.borrow_mut(), |m| {
            m.get_mut(&id).unwrap()
        }))
    }

    /// Get a map that has been loaded. This function is not async unlike [`Self::load_map`].
    /// #Panics
    /// Will panic if the map has not been loaded already.
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

    /// Get Animations.
    pub fn animations(&self) -> RefMut<'_, Option<Vec<rpg::animation::Animation>>> {
        self.animations.borrow_mut()
    }

    /// Get Actors.
    pub fn actors(&self) -> RefMut<'_, Option<Vec<rpg::Actor>>> {
        self.actors.borrow_mut()
    }

    /// Get Common Events.
    pub fn common_events(&self) -> RefMut<'_, Option<Vec<rpg::CommonEvent>>> {
        self.common_events.borrow_mut()
    }

    /// Get Scripts.
    pub fn scripts(&self) -> RefMut<'_, Option<Vec<intermediate::Script>>> {
        self.scripts.borrow_mut()
    }

    /// Get items.
    pub fn items(&self) -> RefMut<'_, Option<Vec<rpg::Item>>> {
        self.items.borrow_mut()
    }

    /// Get Config.
    pub fn config(&self) -> RefMut<'_, Option<LocalConfig>> {
        self.config.borrow_mut()
    }

    /// Save the local config.
    pub async fn save_config(&self, filesystem: &Filesystem) -> Result<(), String> {
        let config_str = self
            .config
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(config_str) = config_str {
            filesystem
                .save_data_at(".luminol", &config_str?)
                .await
                .map_err(|_| "Failed to write Config data")?;
        }

        Ok(())
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub async fn save(&self, filesystem: &Filesystem) -> Result<(), String> {
        self.system().as_mut().unwrap().magic_number = rand::random();

        // Write map data and clear map cache.
        // We serialize all of these first before writing them to the disk to avoid bringing a refcell across an await.
        // A RwLock may be used in the future to solve this, though.
        let maps_strs: HashMap<_, _> = {
            let maps = self.maps.borrow();
            maps.iter()
                .map(|(id, map)| {
                    (
                        *id,
                        to_string_pretty(&map, CONFIG.clone()).map_err(|e| e.to_string()),
                    )
                })
                .collect()
        };

        for (id, map) in maps_strs {
            filesystem
                .save_data(&format!("Map{:0>3}.ron", id), &map?)
                .await
                .map_err(|e| format!("Failed to write Map data {e}"))?
        }

        let tilesets_str = self
            .tilesets
            .borrow()
            .as_ref()
            .map(|t| to_string_pretty(&t, CONFIG.clone()).map_err(|e| e.to_string()));
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
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));
        if let Some(mapinfos_str) = mapinfos_str {
            filesystem
                .save_data("MapInfos.ron", &mapinfos_str?)
                .await
                .map_err(|_| "Failed to write MapInfos data")?;
        }

        let system_str = self
            .system
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(system_str) = system_str {
            filesystem
                .save_data("System.ron", &system_str?)
                .await
                .map_err(|_| "Failed to write System data")?;
        }

        let actors_str = self
            .actors
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(actors_str) = actors_str {
            filesystem
                .save_data("Actors.ron", &actors_str?)
                .await
                .map_err(|_| "Failed to write Actor data")?;
        }

        let animations_str = self
            .animations
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(animations_str) = animations_str {
            filesystem
                .save_data("Animations.ron", &animations_str?)
                .await
                .map_err(|_| "Failed to write Animation data")?;
        }

        let common_events_str = self
            .common_events
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(common_events_str) = common_events_str {
            filesystem
                .save_data("CommonEvents.ron", &common_events_str?)
                .await
                .map_err(|_| "Failed to write Common Event data")?;
        }

        let scripts_str = self
            .scripts
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        let script_path = self
            .config()
            .as_ref()
            .map(|c| c.scripts_path.clone())
            .unwrap_or_else(|| "Scripts".to_string());
        if let Some(scripts_str) = scripts_str {
            filesystem
                .save_data(&format!("{script_path}.ron"), &scripts_str?)
                .await
                .map_err(|_| "Failed to write Script data")?;
        }

        let items_str = self
            .items
            .borrow()
            .as_ref()
            .map(|m| to_string_pretty(&m, CONFIG.clone()).map_err(|e| e.to_string()));

        if let Some(items_str) = items_str {
            filesystem
                .save_data("Items.ron", &items_str?)
                .await
                .map_err(|_| "Failed to write Item data")?;
        }

        self.save_config(filesystem).await
    }

    /// Setup default values
    pub fn setup_defaults(&self) {
        *self.actors() = Some(vec![rpg::Actor::default()]);
        *self.animations() = Some(vec![rpg::animation::Animation::default()]);
        *self.common_events() = Some(vec![rpg::CommonEvent::default()]);
        *self.scripts() = Some(vec![]);
        *self.items() = Some(vec![]);

        let mut map_infos = HashMap::new();
        map_infos.insert(
            1,
            rpg::MapInfo {
                parent_id: 0,
                name: "Map 001".to_string(),
                order: 0,
                expanded: false,
                scroll_x: 0,
                scroll_y: 0,
            },
        );

        *self.map_infos() = Some(map_infos);
        let mut maps = HashMap::new();
        maps.insert(
            1,
            rpg::Map {
                tileset_id: 1,
                width: 20,
                height: 15,
                data: Table3::new(20, 15, 3),
                ..Default::default()
            },
        );
        *self.maps.borrow_mut() = maps;

        *self.tilesets() = Some(vec![rpg::Tileset {
            id: 1,
            passages: Table1::new(8),
            priorities: Table1::new(8),
            terrain_tags: Table1::new(8),
            ..Default::default()
        }]);

        *self.system() = Some(rpg::system::System {
            magic_number: rand::random(),
            ..Default::default()
        });

        *self.config() = Some(Default::default());
    }
}
