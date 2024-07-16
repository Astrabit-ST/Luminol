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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use color_eyre::eyre::WrapErr;
use luminol_data::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use crate::error;

pub mod data_formats;

// TODO convert this to an option like project config?
#[allow(clippy::large_enum_variant)]
#[derive(Default, Debug)]
pub enum Data {
    #[default]
    Unloaded,
    Loaded {
        actors: RefCell<rpg::Actors>,
        animations: RefCell<rpg::Animations>,
        armors: RefCell<rpg::Armors>,
        classes: RefCell<rpg::Classes>,
        common_events: RefCell<rpg::CommonEvents>,
        enemies: RefCell<rpg::Enemies>,
        items: RefCell<rpg::Items>,
        map_infos: RefCell<rpg::MapInfos>,
        scripts: RefCell<rpg::Scripts>,
        skills: RefCell<rpg::Skills>,
        states: RefCell<rpg::States>,
        system: RefCell<rpg::System>,
        tilesets: RefCell<rpg::Tilesets>,
        troops: RefCell<rpg::Troops>,
        weapons: RefCell<rpg::Weapons>,

        maps: RefCell<HashMap<usize, rpg::Map>>,
    },
}

macro_rules! load {
    ($fs:ident, $type:ident, $format_handler:ident) => {
        RefCell::new(rpg::$type {
            data: $format_handler
                .read_nil_padded($fs, format!("{}", stringify!($type)))
                .wrap_err_with(|| format!("While reading {}", stringify!($type)))?,
            ..Default::default()
        })
    };
}
macro_rules! from_defaults {
    ($parent:ident, $child:ident) => {
        RefCell::new(rpg::$parent {
            data: vec![rpg::$child::default()],
            ..Default::default()
        })
    };
}

macro_rules! save {
    ($fs:ident, $type:ident, $field:ident, $format_handler:ident) => {{
        let borrowed = $field.get_mut();
        if borrowed.modified {
            $format_handler
                .write_nil_padded(&borrowed.data, $fs, format!("{}", stringify!($type)))
                .wrap_err_with(|| format!("While saving {}", stringify!($type)))?;
        }
        borrowed.modified
    }};
}

impl Data {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(
        &mut self,
        filesystem: &impl luminol_filesystem::FileSystem,
        toasts: &mut crate::Toasts,
        config: &mut luminol_config::project::Config,
    ) -> color_eyre::Result<()> {
        let handler = data_formats::Handler::new(config.project.data_format);

        let map_infos = RefCell::new(rpg::MapInfos {
            data: handler
                .read_data(filesystem, "MapInfos")
                .wrap_err("While reading MapInfos")?,
            ..Default::default()
        });

        let mut system = handler
            .read_data::<rpg::System>(filesystem, "System")
            .wrap_err("While reading System")?;
        system.magic_number = rand::random();

        let system = RefCell::new(system);

        let mut scripts = None;
        let scripts_paths = [
            config.project.scripts_path.clone(),
            "xScripts".to_string(),
            "Scripts".to_string(),
        ];

        for script_path in scripts_paths {
            match handler.read_data(filesystem, format!("{script_path}")) {
                Ok(s) => {
                    config.project.scripts_path = script_path;
                    scripts = Some(rpg::Scripts {
                        data: s,
                        ..Default::default()
                    });
                    break;
                }
                Err(e) => {
                    error!(
                        *toasts,
                        e.wrap_err(format!(
                            "While attempting to read scripts from {script_path}"
                        ))
                    )
                }
            }
        }
        let Some(scripts) = scripts else {
            color_eyre::eyre::bail!(
                "Unable to load scripts (tried {}, xScripts, and Scripts first)",
                config.project.scripts_path
            );
        };
        let scripts = RefCell::new(scripts);

        let maps = RefCell::new(std::collections::HashMap::with_capacity(32));

        *self = Self::Loaded {
            actors: load!(filesystem, Actors, handler),
            animations: load!(filesystem, Animations, handler),
            armors: load!(filesystem, Armors, handler),
            classes: load!(filesystem, Classes, handler),
            common_events: load!(filesystem, CommonEvents, handler),
            enemies: load!(filesystem, Enemies, handler),
            items: load!(filesystem, Items, handler),
            skills: load!(filesystem, Skills, handler),
            states: load!(filesystem, States, handler),
            tilesets: load!(filesystem, Tilesets, handler),
            troops: load!(filesystem, Troops, handler),
            weapons: load!(filesystem, Weapons, handler),
            map_infos,
            system,
            scripts,
            maps,
        };

        Ok(())
    }

    pub fn unload(&mut self) {
        *self = Self::Unloaded;
    }

    pub fn from_defaults() -> Self {
        let mut map_infos = std::collections::HashMap::with_capacity(16);
        map_infos.insert(1, rpg::MapInfo::default());
        let map_infos = RefCell::new(rpg::MapInfos {
            data: map_infos,
            ..Default::default()
        });

        let system = rpg::System {
            magic_number: rand::random(),
            ..Default::default()
        };
        let system = RefCell::new(system);

        let scripts = vec![]; // FIXME legality of providing defualt scripts is unclear
        let scripts = RefCell::new(rpg::Scripts {
            data: scripts,
            ..Default::default()
        });

        let mut maps = std::collections::HashMap::with_capacity(32);
        maps.insert(1, rpg::Map::default());
        let maps = RefCell::new(maps);

        Self::Loaded {
            actors: from_defaults!(Actors, Actor),
            animations: from_defaults!(Animations, Animation),
            armors: from_defaults!(Armors, Armor),
            classes: from_defaults!(Classes, Class),
            common_events: from_defaults!(CommonEvents, CommonEvent),
            enemies: from_defaults!(Enemies, Enemy),
            items: from_defaults!(Items, Item),
            skills: from_defaults!(Skills, Skill),
            states: from_defaults!(States, State),
            tilesets: from_defaults!(Tilesets, Tileset),
            troops: from_defaults!(Troops, Troop),
            weapons: from_defaults!(Weapons, Weapon),
            map_infos,
            system,
            scripts,
            maps,
        }
    }

    pub fn rxdata_ext(&self) -> &'static str {
        todo!()
    }

    /// Save all cached data to disk.
    // we take an &mut self to ensure no outsanding borrows of the cache exist.
    pub fn save(
        &mut self,
        filesystem: &impl luminol_filesystem::FileSystem,
        config: &luminol_config::project::Config,
    ) -> color_eyre::Result<()> {
        let handler = data_formats::Handler::new(config.project.data_format);

        let Self::Loaded {
            actors,
            animations,
            armors,
            classes,
            common_events,
            enemies,
            items,
            map_infos,
            scripts,
            skills,
            states,
            tilesets,
            troops,
            weapons,
            system,
            maps,
        } = self
        else {
            panic!("project not loaded")
        };

        let mut modified = false;

        modified |= save!(filesystem, Actors, actors, handler);
        modified |= save!(filesystem, Animations, animations, handler);
        modified |= save!(filesystem, Armors, armors, handler);
        modified |= save!(filesystem, Classes, classes, handler);
        modified |= save!(filesystem, CommonEvents, common_events, handler);
        modified |= save!(filesystem, Enemies, enemies, handler);
        modified |= save!(filesystem, Items, items, handler);
        modified |= save!(filesystem, Skills, skills, handler);
        modified |= save!(filesystem, States, states, handler);
        modified |= save!(filesystem, Tilesets, tilesets, handler);
        modified |= save!(filesystem, Troops, troops, handler);
        modified |= save!(filesystem, Weapons, weapons, handler);

        {
            let map_infos = map_infos.get_mut();
            if map_infos.modified {
                modified = true;
                handler
                    .write_data(&map_infos.data, filesystem, "MapInfos")
                    .wrap_err("While saving MapInfos")?;
            }
        }

        {
            let scripts = scripts.get_mut();
            if scripts.modified {
                modified = true;
                handler.write_data(&scripts.data, filesystem, &config.project.scripts_path)?;
            }
        }

        {
            let maps = maps.get_mut();
            maps.iter().try_for_each(|(id, map)| {
                if map.modified {
                    modified = true;
                    handler
                        .write_data(map, filesystem, format!("Map{id:0>3}"))
                        .wrap_err_with(|| format!("While saving map {id:0>3}"))
                } else {
                    Ok(())
                }
            })?
        }

        {
            let system = system.get_mut();
            if system.modified || modified {
                system.magic_number = rand::random();
                handler
                    .write_data(system, filesystem, "System")
                    .wrap_err("While saving System")?;
                system.modified = false;
            }
        }

        let pretty_config = ron::ser::PrettyConfig::new()
            .struct_names(true)
            .enumerate_arrays(true);

        let project_config = ron::ser::to_string_pretty(&config.project, pretty_config.clone())
            .wrap_err("While serializing .luminol/config")?;
        filesystem
            .write(".luminol/config", project_config)
            .wrap_err("While writing .luminol/config")?;

        let command_db = ron::ser::to_string_pretty(&config.command_db, pretty_config.clone())
            .wrap_err("While serializing .luminol/commands")?;
        filesystem
            .write(".luminol/commands", command_db)
            .wrap_err("While writing .luminol/config")?;

        // even though Ini uses fmt::write internally, it provides no easy way to write to a string.
        // so we need to open a file instead
        let mut ini_file = filesystem
            .open_file(
                "Game.ini",
                luminol_filesystem::OpenFlags::Create
                    | luminol_filesystem::OpenFlags::Write
                    | luminol_filesystem::OpenFlags::Truncate,
            )
            .wrap_err("While opening Game.ini")?;
        config
            .game_ini
            .write_to(&mut ini_file)
            .wrap_err("While serializing Game.ini")?;

        actors.borrow_mut().modified = false;
        animations.borrow_mut().modified = false;
        armors.borrow_mut().modified = false;
        classes.borrow_mut().modified = false;
        common_events.borrow_mut().modified = false;
        enemies.borrow_mut().modified = false;
        items.borrow_mut().modified = false;
        skills.borrow_mut().modified = false;
        states.borrow_mut().modified = false;
        tilesets.borrow_mut().modified = false;
        troops.borrow_mut().modified = false;
        weapons.borrow_mut().modified = false;
        map_infos.borrow_mut().modified = false;
        scripts.borrow_mut().modified = false;
        for (_, map) in maps.borrow_mut().iter_mut() {
            map.modified = false;
        }
        Ok(())
    }
}

macro_rules! nested_ref_getter {
    ($($typ:ty, $name:ident),* $(,)?) => {
        $(
            #[allow(unsafe_code, dead_code)]
            pub fn $name(&self) -> RefMut<'_, $typ> {
                match self {
                    Self::Unloaded => panic!("data cache unloaded"),
                    Self::Loaded { $name, ..} => $name.borrow_mut(),
                }
            }
        )+
    };

}

impl Data {
    nested_ref_getter! {
        rpg::Actors, actors,
        rpg::Animations, animations,
        rpg::Armors, armors,
        rpg::Classes, classes,
        rpg::CommonEvents, common_events,
        rpg::Enemies, enemies,
        rpg::Items, items,
        rpg::MapInfos, map_infos,
        rpg::Scripts, scripts,
        rpg::Skills, skills,
        rpg::States, states,
        rpg::System, system,
        rpg::Tilesets, tilesets,
        rpg::Troops, troops,
        rpg::Weapons, weapons,
    }

    /// Load a map.
    #[allow(clippy::panic)]
    pub fn get_or_load_map(
        &self,
        id: usize,
        filesystem: &impl luminol_filesystem::FileSystem,
        config: &luminol_config::project::Config,
    ) -> RefMut<'_, rpg::Map> {
        let maps_ref = match self {
            Self::Loaded { maps, .. } => maps.borrow_mut(),
            Self::Unloaded => panic!("project not loaded"),
        };
        RefMut::map(maps_ref, |maps| {
            // FIXME
            maps.entry(id).or_insert_with(|| {
                let handler = data_formats::Handler::new(config.project.data_format);
                handler
                    .read_data(filesystem, format!("Map{id:0>3}"))
                    .expect("failed to load map")
            })
        })
    }

    pub fn get_map(&self, id: usize) -> RefMut<'_, rpg::Map> {
        let maps_ref = match self {
            Self::Loaded { maps, .. } => maps.borrow_mut(),
            Self::Unloaded => panic!("project not loaded"),
        };
        RefMut::map(maps_ref, |maps| maps.get_mut(&id).expect("map not loaded"))
    }
}
