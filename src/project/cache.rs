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

use crate::prelude::*;
use core::ops::{Deref, DerefMut};
/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct Cache {
    actors: AtomicRefCell<Option<NilPadded<rpg::Actor>>>,
    animations: AtomicRefCell<Option<NilPadded<rpg::Animation>>>,
    armors: AtomicRefCell<Option<NilPadded<rpg::Armor>>>,
    classes: AtomicRefCell<Option<NilPadded<rpg::Class>>>,
    commonevents: AtomicRefCell<Option<NilPadded<rpg::CommonEvent>>>,
    enemies: AtomicRefCell<Option<NilPadded<rpg::Enemy>>>,
    items: AtomicRefCell<Option<NilPadded<rpg::Item>>>,
    mapinfos: AtomicRefCell<Option<HashMap<i32, rpg::MapInfo>>>,
    maps: AtomicRefCell<HashMap<i32, rpg::Map>>,
    scripts: AtomicRefCell<Option<Vec<rpg::Script>>>,
    skills: AtomicRefCell<Option<NilPadded<rpg::Skill>>>,
    states: AtomicRefCell<Option<NilPadded<rpg::State>>>,
    system: AtomicRefCell<Option<rpg::System>>,
    tilesets: AtomicRefCell<Option<NilPadded<rpg::Tileset>>>,
    troops: AtomicRefCell<Option<NilPadded<rpg::Troop>>>,
    weapons: AtomicRefCell<Option<NilPadded<rpg::Weapon>>>,

    config: AtomicRefCell<Option<LocalConfig>>,
    commanddb: AtomicRefCell<Option<CommandDB>>,
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
                        .map_err(|s| format!(concat!("Failed to load ", stringify!($name) ,": {}"), s))?,
                );
            }
        )*
    };
}

macro_rules! getter {
    ($($name:ident, $type:ty),*) => {
        $(
            paste::paste! {
                #[doc = "Get `" $name "` from the data cache. Panics if the project was not loaded."]
                pub fn [< $name:lower>](&self) -> impl Deref<Target = $type> + DerefMut + '_ {
                    AtomicRefMut::map(self.[< $name:lower >].borrow_mut(), |o| Option::as_mut(o).expect(concat!("Grabbing ", stringify!($name), " from the data cache failed because the project was not loaded. Please file this as an issue")))
                }

                #[doc = "Try getting `" $name "` from the data cache."]
                pub fn [<try_ $name:lower>](&self) -> Option<impl Deref<Target = $type> + DerefMut + '_ > {
                    AtomicRefMut::filter_map(self.[< $name:lower >].borrow_mut(), |o| Option::as_mut(o))
                }

                #[doc = "Get the raw optional `" $name "` from the data cache."]
                pub fn [<raw_ $name:lower>](&self) -> impl Deref<Target = Option<$type>> + DerefMut + '_  {
                    self.[< $name:lower >].borrow_mut()
                }
            }
        )*
    };
}

macro_rules! setup_default {
    ($this:ident, $($name:ident),*) => {
        $(
            paste::paste! {
                *$this.[< $name:lower >].borrow_mut() = Some(
                    // This is a pretty dirty hack to make rustc assume that it's a vec of the type we're storing
                    NilPadded::from(vec![None, Some(Default::default())])
                );
            }
        )*
    };
}

impl Cache {
    /// Load all data required when opening a project.
    pub fn load(&self) -> Result<(), String> {
        let filesystem = &state!().filesystem;

        if !filesystem.path_exists(".luminol") {
            filesystem.create_directory(".luminol")?;
        }

        let config = match filesystem
            .read_bytes(".luminol/config")
            .ok()
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let config = LocalConfig::default();
                filesystem
                    .save_data(
                        ".luminol/config",
                        ron::ser::to_string_pretty(
                            &config,
                            ron::ser::PrettyConfig::default().struct_names(true),
                        )
                        .expect("Failed to serialize config"),
                    )
                    .expect("Failed to write config data after failing to load config data");
                config
            }
        };

        let commanddb = match filesystem
            .read_bytes(".luminol/commands")
            .ok()
            .and_then(|v| String::from_utf8(v).ok())
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let config = CommandDB::new(config.editor_ver);
                filesystem
                    .save_data(
                        ".luminol/commands",
                        ron::ser::to_string_pretty(
                            &config,
                            ron::ser::PrettyConfig::default().struct_names(true),
                        )
                        .expect("Failed to serialize commands"),
                    )
                    .expect("Failed to write config data after failing to load command data");
                config
            }
        };

        *self.config.borrow_mut() = Some(config);
        *self.commanddb.borrow_mut() = Some(commanddb);

        load_data! {
            self, filesystem,
            Actors, Animations, Armors,
            Classes, CommonEvents, Enemies,
            Items, MapInfos,
            Skills, States, System,
            Tilesets, Troops, Weapons
        }

        let mut scripts = filesystem.read_data("Data/xScripts.rxdata");

        if let Err(e) = scripts {
            eprintln!("Attempted loading xScripts failed with {e}");

            scripts = filesystem.read_data("Data/Scripts.rxdata");
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
    pub fn load_map(
        &self,
        id: i32,
    ) -> Result<impl Deref<Target = rpg::Map> + DerefMut + '_, String> {
        let has_map = self.maps.borrow().contains_key(&id);
        if !has_map {
            let map = state!()
                .filesystem
                .read_data(format!("Data/Map{id:0>3}.rxdata",))
                .map_err(|e| format!("Failed to load map: {e}"))?;
            self.maps.borrow_mut().insert(id, map);
        }
        Ok(AtomicRefMut::map(self.maps.borrow_mut(), |m| {
            m.get_mut(&id).unwrap()
        }))
    }

    /// Get a map that has been loaded. This function is not async unlike [`Self::load_map`].
    /// # Panics
    /// Will panic if the map has not been loaded already.
    pub fn get_map(&self, id: i32) -> impl Deref<Target = rpg::Map> + DerefMut + '_ {
        AtomicRefMut::map(self.maps.borrow_mut(), |maps| maps.get_mut(&id).unwrap())
    }

    getter! {
        Actors, NilPadded<rpg::Actor>,
        Animations, NilPadded<rpg::Animation>,
        Armors, NilPadded<rpg::Armor>,
        Classes, NilPadded<rpg::Class>,
        CommonEvents, NilPadded<rpg::CommonEvent>,
        Enemies, NilPadded<rpg::Enemy>,
        Items, NilPadded<rpg::Item>,
        MapInfos, HashMap<i32, rpg::MapInfo>,
        Scripts, Vec<rpg::Script>,
        Skills, NilPadded<rpg::Skill>,
        States, NilPadded<rpg::State>,
        System, rpg::System,
        Tilesets, NilPadded<rpg::Tileset>,
        Troops, NilPadded<rpg::Troop>,
        Weapons, NilPadded<rpg::Weapon>,

        Config, LocalConfig,
        CommandDB, CommandDB
    }

    /// Save the local config.
    pub fn save_config(&self, filesystem: &Filesystem) -> Result<(), String> {
        if !filesystem.path_exists(".luminol") {
            filesystem.create_directory(".luminol")?;
        }

        let config_str = ron::ser::to_string_pretty(
            &*self.config(),
            ron::ser::PrettyConfig::default().struct_names(true),
        )
        .map_err(|e| format!("Failed to serialize config data: {e}"))?;

        filesystem
            .save_data(".luminol/config", config_str)
            .map_err(|_| "Failed to write Config data")?;

        let commands_str = ron::ser::to_string_pretty(
            &*self.commanddb(),
            ron::ser::PrettyConfig::default().struct_names(true),
        )
        .map_err(|e| format!("Failed to serialize command data: {e}"))?;

        filesystem
            .save_data(".luminol/commands", commands_str)
            .map_err(|_| "Failed to write Config data")?;

        Ok(())
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub fn save(&self, filesystem: &Filesystem) -> Result<(), String> {
        self.system().magic_number = rand::random();

        // Write map data and clear map cache.
        // We serialize all of these first before writing them to the disk to avoid bringing a AtomicRefCell across an await.
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
                .map_err(|e| format!("Failed to write Map data {e}"))?;
        }

        let scripts_bytes =
            alox_48::to_bytes(&*self.scripts()).map_err(|e| format!("Saving Scripts: {e}"))?;
        filesystem
            .save_data(
                format!("Data/{}.rxdata", self.config().scripts_path),
                scripts_bytes,
            )
            .map_err(|e| format!("Failed to write Script data {e}"))?;

        save_data! {
            self, filesystem,
            Actors, Animations, Armors,
            Classes, CommonEvents, Enemies,
            Items, MapInfos,
            Skills, States, System, // FIXME: save to xScripts too!
            Tilesets, Troops, Weapons
        };

        self.save_config(filesystem)
    }

    /// Setup default values
    pub fn setup_defaults(&self) {
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
        *self.mapinfos.borrow_mut() = Some(map_infos);

        // FIXME: make this static somehow?
        *self.scripts.borrow_mut() =
            Some(alox_48::from_bytes(include_bytes!("Scripts.rxdata")).unwrap());

        *self.system.borrow_mut() = Some(rpg::System {
            magic_number: rand::random(),
            ..Default::default()
        });

        let mut maps = HashMap::new();
        maps.insert(
            1,
            rpg::Map {
                tileset_id: 1,
                width: 20,
                height: 15,
                data: rmxp_types::Table3::new(20, 15, 3),
                ..Default::default()
            },
        );
        *self.maps.borrow_mut() = maps;

        *self.config.borrow_mut() = Some(LocalConfig::default());
        *self.commanddb.borrow_mut() = Some(CommandDB::default());

        setup_default! {
            self,
            Actors, Animations, Armors,
            Classes, CommonEvents, Enemies,
            Items, Skills, States,
            Tilesets, Troops, Weapons
        }
    }
}
