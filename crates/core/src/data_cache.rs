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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use anyhow::Context;
use luminol_data::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

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

fn read_data<T>(
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let path = camino::Utf8PathBuf::from("Data").join(filename);
    let data = filesystem.read(path)?;

    alox_48::from_bytes(&data).map_err(anyhow::Error::from)
}

fn write_data(
    data: &impl serde::Serialize,
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> anyhow::Result<()> {
    let path = camino::Utf8PathBuf::from("Data").join(filename);

    let bytes = alox_48::to_bytes(data)?;
    filesystem.write(path, bytes).map_err(anyhow::Error::from)
}

fn read_nil_padded<T>(
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> anyhow::Result<Vec<T>>
where
    T: serde::de::DeserializeOwned,
{
    let path = camino::Utf8PathBuf::from("Data").join(filename);
    let data = filesystem.read(path)?;

    let mut de = alox_48::Deserializer::new(&data)?;

    luminol_data::helpers::nil_padded::deserialize(&mut de).map_err(anyhow::Error::from)
}

fn write_nil_padded(
    data: &[impl serde::Serialize],
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> anyhow::Result<()> {
    let path = camino::Utf8PathBuf::from("Data").join(filename);

    let mut ser = alox_48::Serializer::new();

    luminol_data::helpers::nil_padded::serialize(data, &mut ser)?;
    filesystem
        .write(path, ser.output)
        .map_err(anyhow::Error::from)
}

macro_rules! load {
    ($fs:ident, $type:ident) => {
        RefCell::new(rpg::$type {
            data: read_nil_padded($fs, format!("{}.rxdata", stringify!($type)))
                .context(format!("while reading {}.rxdata", stringify!($type)))?,
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
    ($fs:ident, $type:ident, $field:ident) => {
        let mut borrowed = $field.borrow_mut();
        if borrowed.modified {
            borrowed.modified = false;
            write_nil_padded(&borrowed.data, $fs, format!("{}.rxdata", stringify!($type)))
                .context(format!("while saving {}.rxdata", stringify!($type)))?;
        }
    };
}
impl Data {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(
        &mut self,
        filesystem: &impl luminol_filesystem::FileSystem,
        config: &mut luminol_config::project::Config,
    ) -> anyhow::Result<()> {
        let map_infos = RefCell::new(rpg::MapInfos {
            data: read_data(filesystem, "MapInfos.rxdata")
                .context("while reading MapInfos.rxdata")?,
            ..Default::default()
        });

        let mut system = read_data::<rpg::System>(filesystem, "System.rxdata")
            .context("while reading System.rxdata")?;
        system.magic_number = rand::random();

        let system = RefCell::new(system);

        let mut scripts = None;
        let scripts_paths = [
            std::mem::take(&mut config.project.scripts_path),
            "xScripts".to_string(),
            "Scripts".to_string(),
        ];

        for script_path in scripts_paths {
            match read_data(filesystem, format!("{script_path}.rxdata")) {
                Ok(s) => {
                    config.project.scripts_path = script_path;
                    scripts = Some(rpg::Scripts {
                        data: s,
                        ..Default::default()
                    });
                    break;
                }
                Err(e) => eprintln!("error loading scripts from {script_path}: {e}"),
            }
        }
        let Some(scripts) = scripts else {
            anyhow::bail!(
                "Unable to load scripts (tried {}, xScripts, and Scripts first)",
                config.project.scripts_path
            );
        };
        let scripts = RefCell::new(scripts);

        let maps = RefCell::new(std::collections::HashMap::with_capacity(32));

        *self = Self::Loaded {
            actors: load!(filesystem, Actors),
            animations: load!(filesystem, Animations),
            armors: load!(filesystem, Armors),
            classes: load!(filesystem, Classes),
            common_events: load!(filesystem, CommonEvents),
            enemies: load!(filesystem, Enemies),
            items: load!(filesystem, Items),
            skills: load!(filesystem, Skills),
            states: load!(filesystem, States),
            tilesets: load!(filesystem, Tilesets),
            troops: load!(filesystem, Troops),
            weapons: load!(filesystem, Weapons),
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
    ) -> anyhow::Result<()> {
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

        save!(filesystem, Actors, actors);
        save!(filesystem, Animations, animations);
        save!(filesystem, Armors, armors);
        save!(filesystem, Classes, classes);
        save!(filesystem, CommonEvents, common_events);
        save!(filesystem, Enemies, enemies);
        save!(filesystem, Items, items);
        save!(filesystem, Scripts, scripts);
        save!(filesystem, Skills, skills);
        save!(filesystem, States, states);
        save!(filesystem, Tilesets, tilesets);
        save!(filesystem, Troops, troops);
        save!(filesystem, Weapons, weapons);

        {
            let mut map_infos = map_infos.borrow_mut();
            if map_infos.modified {
                map_infos.modified = false;
                write_data(&map_infos.data, filesystem, "MapInfos.rxdata")
                    .context("while saving MapInfos.rxdata")?;
            }
        }

        {
            let system = system.get_mut();
            system.magic_number = rand::random();
            write_data(system, filesystem, "System.rxdata")
                .context("while saving System.rxdata")?;
        }

        {
            let mut scripts = scripts.borrow_mut();
            if scripts.modified {
                scripts.modified = false;
                write_data(
                    &scripts.data,
                    filesystem,
                    format!("{}.rxdata", config.project.scripts_path),
                )?;
            }
        }

        {
            let mut maps = maps.borrow_mut();
            maps.iter_mut().try_for_each(|(id, map)| {
                if map.modified {
                    map.modified = false;
                    write_data(map, filesystem, format!("Map{id:0>3}.rxdata"))
                        .with_context(|| format!("while saving map {id:0>3}"))?
                }
                Ok(())
            })
        }
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
    ) -> RefMut<'_, rpg::Map> {
        let maps_ref = match self {
            Self::Loaded { maps, .. } => maps.borrow_mut(),
            Self::Unloaded => panic!("project not loaded"),
        };
        RefMut::map(maps_ref, |maps| {
            // FIXME
            maps.entry(id).or_insert_with(|| {
                read_data(filesystem, format!("Map{id:0>3}.rxdata")).expect("failed to load map")
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
