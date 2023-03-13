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

use crate::data::nil_padded::NilPadded;
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

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct Cache {
    actors: RefCell<Option<NilPadded<rpg::Actor>>>,
    animations: RefCell<Option<NilPadded<rpg::animation::Animation>>>,
    armors: RefCell<Option<NilPadded<rpg::Armor>>>,
    classes: RefCell<Option<NilPadded<rpg::class::Class>>>,
    commonevents: RefCell<Option<NilPadded<rpg::CommonEvent>>>,
    enemies: RefCell<Option<NilPadded<rpg::enemy::Enemy>>>,
    items: RefCell<Option<NilPadded<rpg::Item>>>,
    mapinfos: RefCell<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: RefCell<HashMap<i32, rpg::Map>>,
    scripts: RefCell<Option<Vec<intermediate::Script>>>,
    skills: RefCell<Option<NilPadded<rpg::Skill>>>,
    states: RefCell<Option<NilPadded<rpg::State>>>,
    system: RefCell<Option<rpg::system::System>>,
    tilesets: RefCell<Option<NilPadded<rpg::Tileset>>>,
    troops: RefCell<Option<NilPadded<rpg::troop::Troop>>>,
    weapons: RefCell<Option<NilPadded<rpg::Weapon>>>,

    config: RefCell<Option<LocalConfig>>,
}

macro_rules! save_data {
    ($this:ident, $filesystem:ident, $($name:ident),*) => {
        $(
            paste::paste! {
                let _bytes = $this
                    .[< $name:lower >]
                    .borrow()
                    .as_ref()
                    .map(|t| alox_48::to_bytes(t).map_err(|e| format!(concat!("Saving ", stringify!($name), ": {}"), e)));

                if let Some(_bytes) = _bytes {
                    $filesystem
                        .save_data(concat!("Data/", stringify!($name), ".rxdata"), _bytes?)
                        .await
                        .map_err(|_| concat!("Failed to write", stringify!($name), "data"))?;
                }
            }
        )*
    };
}

macro_rules! load_data {
    ($this:ident, $filesystem:ident, $($name:ident),*) => {
        $(
            paste::paste! {
                *$this.[< $name:lower >].borrow_mut() = Some(
                    $filesystem
                        .read_data(concat!("Data/", stringify!($name), ".rxdata"))
                        .await
                        .map_err(|s| format!(concat!("Failed to load ", stringify!($name) ,": {}"), s))?,
                );
            }
        )*
    };
}

impl Cache {
    /// Load all data required when opening a project.
    pub async fn load(&self, filesystem: &impl Filesystem) -> Result<(), String> {
        // FIXME: keep errors?
        let config = filesystem
            .read_bytes(".luminol")
            .await
            .ok()
            .and_then(|v| {
                String::from_utf8(v)
                    .ok()
                    .and_then(|s| ron::from_str::<LocalConfig>(&s).ok())
            })
            .unwrap_or_default();

        *self.config.borrow_mut() = Some(config);

        load_data! {
            self, filesystem,
            Actors, Animations, Armors,
            Classes, CommonEvents, Enemies,
            Items, MapInfos,
            Skills, States, System,
            Tilesets, Troops, Weapons
        }

        let mut scripts = filesystem.read_data("Data/xScripts.rxdata").await;

        if let Err(e) = scripts {
            println!("Attempted loading xScripts failed with {e}");

            scripts = filesystem.read_data("Data/Scripts.rxdata").await;
        } else {
            self.config.borrow_mut().as_mut().unwrap().scripts_path = "xScripts".to_string();
        }

        *self.scripts.borrow_mut() = Some(
            scripts.map_err(|s| format!("Failed to read Scripts (tried xScripts first): {s}"))?,
        );

        self.maps.borrow_mut().clear();
        Ok(())
    }

    /// Load a map.
    pub async fn load_map(
        &self,
        filesystem: &'static impl Filesystem,
        id: i32,
    ) -> Result<RefMut<'_, rpg::Map>, String> {
        let has_map = self.maps.borrow().contains_key(&id);
        if !has_map {
            let map = filesystem
                .read_data(format!("Data/Map{id:0>3}.rxdata",))
                .await
                .map_err(|e| format!("Failed to load map: {e}"))?;
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

    // FIXME: make these into a macro
    /// Get MapInfos.
    pub fn map_infos(&self) -> RefMut<'_, Option<HashMap<i32, rpg::MapInfo>>> {
        self.mapinfos.borrow_mut()
    }

    /// Get Tilesets.
    pub fn tilesets(&self) -> RefMut<'_, Option<NilPadded<rpg::Tileset>>> {
        self.tilesets.borrow_mut()
    }

    /// Get system.
    pub fn system(&self) -> RefMut<'_, Option<rpg::system::System>> {
        self.system.borrow_mut()
    }

    /// Get Animations.
    pub fn animations(&self) -> RefMut<'_, Option<NilPadded<rpg::animation::Animation>>> {
        self.animations.borrow_mut()
    }

    /// Get Actors.
    pub fn actors(&self) -> RefMut<'_, Option<NilPadded<rpg::Actor>>> {
        self.actors.borrow_mut()
    }

    /// Get Common Events.
    pub fn common_events(&self) -> RefMut<'_, Option<NilPadded<rpg::CommonEvent>>> {
        self.commonevents.borrow_mut()
    }

    /// Get Scripts.
    pub fn scripts(&self) -> RefMut<'_, Option<Vec<intermediate::Script>>> {
        self.scripts.borrow_mut()
    }

    /// Get items.
    pub fn items(&self) -> RefMut<'_, Option<NilPadded<rpg::Item>>> {
        self.items.borrow_mut()
    }

    /// Get Config.
    pub fn config(&self) -> RefMut<'_, Option<LocalConfig>> {
        self.config.borrow_mut()
    }

    /// Save the local config.
    pub async fn save_config(&self, filesystem: &impl Filesystem) -> Result<(), String> {
        let config_bytes = self.config.borrow().as_ref().map(|c| {
            ron::to_string(c).map_err(|e| format!("Failed to serialize config data: {e}"))
        });

        if let Some(config_bytes) = config_bytes {
            filesystem
                .save_data(".luminol", &config_bytes?)
                .await
                .map_err(|_| "Failed to write Config data")?;
        }

        Ok(())
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub async fn save(&self, filesystem: &impl Filesystem) -> Result<(), String> {
        self.system().as_mut().unwrap().magic_number = rand::random();

        // Write map data and clear map cache.
        // We serialize all of these first before writing them to the disk to avoid bringing a refcell across an await.
        // A RwLock may be used in the future to solve this, though.
        let maps_bytes: HashMap<_, _> = {
            let maps = self.maps.borrow();
            maps.iter()
                .map(|(id, map)| (*id, alox_48::to_bytes(map).map_err(|e| e.to_string())))
                .collect()
        };

        for (id, map) in maps_bytes {
            filesystem
                .save_data(format!("Data/Map{id:0>3}.rxdata",), map?)
                .await
                .map_err(|e| format!("Failed to write Map data {e}"))?;
        }

        save_data! {
            self, filesystem,
            Actors, Animations, Armors,
            Classes, CommonEvents, Enemies,
            Items, MapInfos, Scripts,
            Skills, States, System, // FIXME: save to xScripts too!
            Tilesets, Troops, Weapons
        };

        self.save_config(filesystem).await
    }

    /// Setup default values
    pub fn setup_defaults(&self) {
        // FIXME: make macro
        *self.actors() = Some(vec![rpg::Actor::default()].into());
        *self.animations() = Some(vec![rpg::animation::Animation::default()].into());
        *self.armors.borrow_mut() = Some(vec![rpg::Armor::default()].into());
        *self.classes.borrow_mut() = Some(vec![rpg::class::Class::default()].into());
        *self.common_events() = Some(vec![rpg::CommonEvent::default()].into());
        *self.enemies.borrow_mut() = Some(vec![rpg::enemy::Enemy::default()].into());
        *self.items() = Some(NilPadded::default());

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

        *self.scripts() = Some(alox_48::from_bytes(include_bytes!("Scripts.rxdata")).unwrap()); // FIXME: make this static somehow?
        *self.skills.borrow_mut() = Some(vec![rpg::Skill::default()].into());
        *self.states.borrow_mut() = Some(vec![rpg::State::default()].into());

        *self.system() = Some(rpg::system::System {
            magic_number: rand::random(),
            ..Default::default()
        });

        *self.tilesets() = Some(
            vec![rpg::Tileset {
                id: 1,
                passages: Table1::new(8),
                priorities: Table1::new(8),
                terrain_tags: Table1::new(8),
                ..Default::default()
            }]
            .into(),
        );

        *self.troops.borrow_mut() = Some(vec![rpg::troop::Troop::default()].into());
        *self.weapons.borrow_mut() = Some(vec![rpg::Weapon::default()].into());

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

        *self.config() = Some(LocalConfig::default());
    }
}
