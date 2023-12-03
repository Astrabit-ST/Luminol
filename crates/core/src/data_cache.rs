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

impl Data {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(
        &mut self,
        filesystem: &impl luminol_filesystem::FileSystem,
        config: &mut luminol_config::project::Config,
    ) -> anyhow::Result<()> {
        let actors = RefCell::new(rpg::Actors {
            data: read_nil_padded(filesystem, "Actors.rxdata")
                .context("while reading actor data")?,
            ..Default::default()
        });
        let animations = RefCell::new(rpg::Animations {
            data: read_nil_padded(filesystem, "Animations.rxdata")
                .context("while reading animation data")?,
            ..Default::default()
        });
        let armors = RefCell::new(rpg::Armors {
            data: read_nil_padded(filesystem, "Armors.rxdata")
                .context("while reading armor data")?,
            ..Default::default()
        });
        let classes = RefCell::new(rpg::Classes {
            data: read_nil_padded(filesystem, "Classes.rxdata")
                .context("while reading class data")?,
            ..Default::default()
        });
        let common_events = RefCell::new(rpg::CommonEvents {
            data: read_nil_padded(filesystem, "CommonEvents.rxdata")
                .context("while reading common events")?,
            ..Default::default()
        });
        let enemies = RefCell::new(rpg::Enemies {
            data: read_nil_padded(filesystem, "Enemies.rxdata")
                .context("while reading enemy data")?,
            ..Default::default()
        });
        let items = RefCell::new(rpg::Items {
            data: read_nil_padded(filesystem, "Items.rxdata").context("while reading item data")?,
            ..Default::default()
        });
        let skills = RefCell::new(rpg::Skills {
            data: read_nil_padded(filesystem, "Skills.rxdata")
                .context("while reading skill data")?,
            ..Default::default()
        });
        let states = RefCell::new(rpg::States {
            data: read_nil_padded(filesystem, "States.rxdata")
                .context("while reading state data")?,
            ..Default::default()
        });
        let tilesets = RefCell::new(rpg::Tilesets {
            data: read_nil_padded(filesystem, "Tilesets.rxdata")
                .context("while reading tileset data")?,
            ..Default::default()
        });
        let troops = RefCell::new(rpg::Troops {
            data: read_nil_padded(filesystem, "Troops.rxdata")
                .context("while reading troop data")?,
            ..Default::default()
        });
        let weapons = RefCell::new(rpg::Weapons {
            data: read_nil_padded(filesystem, "Weapons.rxdata")
                .context("while reading weapon data")?,
            ..Default::default()
        });

        let map_infos = RefCell::new(rpg::MapInfos {
            data: read_data(filesystem, "MapInfos.rxdata").context("while reading map infos")?,
            ..Default::default()
        });

        let mut system = read_data::<rpg::System>(filesystem, "System.rxdata")
            .context("while reading system")?;
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
            system,
            tilesets,
            troops,
            weapons,
            maps,
        };

        Ok(())
    }

    pub fn unload(&mut self) {
        *self = Self::Unloaded;
    }

    pub fn from_defaults() -> Self {
        let actors = RefCell::new(rpg::Actors {
            data: vec![rpg::Actor::default()],
            ..Default::default()
        });
        let animations = RefCell::new(rpg::Animations {
            data: vec![rpg::Animation::default()],
            ..Default::default()
        });
        let armors = RefCell::new(rpg::Armors {
            data: vec![rpg::Armor::default()],
            ..Default::default()
        });
        let classes = RefCell::new(rpg::Classes {
            data: vec![rpg::Class::default()],
            ..Default::default()
        });
        let common_events = RefCell::new(rpg::CommonEvents {
            data: vec![rpg::CommonEvent::default()],
            ..Default::default()
        });
        let enemies = RefCell::new(rpg::Enemies {
            data: vec![rpg::Enemy::default()],
            ..Default::default()
        });
        let items = RefCell::new(rpg::Items {
            data: vec![rpg::Item::default()],
            ..Default::default()
        });
        let skills = RefCell::new(rpg::Skills {
            data: vec![rpg::Skill::default()],
            ..Default::default()
        });
        let states = RefCell::new(rpg::States {
            data: vec![rpg::State::default()],
            ..Default::default()
        });
        let tilesets = RefCell::new(rpg::Tilesets {
            data: vec![rpg::Tileset::default()],
            ..Default::default()
        });
        let troops = RefCell::new(rpg::Troops {
            data: vec![rpg::Troop::default()],
            ..Default::default()
        });
        let weapons = RefCell::new(rpg::Weapons {
            data: vec![rpg::Weapon::default()],
            ..Default::default()
        });

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
            system,
            tilesets,
            troops,
            weapons,
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
            system,
            tilesets,
            troops,
            weapons,
            maps,
        } = self
        else {
            panic!("project not loaded")
        };

        let mut actors = actors.borrow_mut();
        if actors.modified {
            actors.modified = false;
            write_nil_padded(&actors.data, filesystem, "Actors.rxdata")
                .context("while saving actor data")?;
        }
        let mut animations = animations.borrow_mut();
        if animations.modified {
            animations.modified = false;
            write_nil_padded(&animations.data, filesystem, "Animations.rxdata")
                .context("while saving animation data")?;
        }
        let mut armors = armors.borrow_mut();
        if armors.modified {
            armors.modified = false;
            write_nil_padded(&armors.data, filesystem, "Armors.rxdata")
                .context("while saving armor data")?;
        }
        let mut classes = classes.borrow_mut();
        if classes.modified {
            classes.modified = false;
            write_nil_padded(&classes.data, filesystem, "Classes.rxdata")
                .context("while saving class data")?;
        }
        let mut common_events = common_events.borrow_mut();
        if common_events.modified {
            common_events.modified = false;
            write_nil_padded(&common_events.data, filesystem, "CommonEvents.rxdata")
                .context("while saving common event data")?;
        }
        let mut enemies = enemies.borrow_mut();
        if enemies.modified {
            enemies.modified = false;
            write_nil_padded(&enemies.data, filesystem, "Enemies.rxdata")
                .context("while saving enemy data")?;
        }
        let mut items = items.borrow_mut();
        if items.modified {
            items.modified = false;
            write_nil_padded(&items.data, filesystem, "Items.rxdata")
                .context("while saving item data")?;
        }
        let mut skills = skills.borrow_mut();
        if skills.modified {
            skills.modified = false;
            write_nil_padded(&skills.data, filesystem, "Skills.rxdata")
                .context("while saving skill data")?;
        }
        let mut states = states.borrow_mut();
        if states.modified {
            states.modified = false;
            write_nil_padded(&states.data, filesystem, "States.rxdata")
                .context("while saving state data")?;
        }
        let mut tilesets = tilesets.borrow_mut();
        if tilesets.modified {
            tilesets.modified = false;
            write_nil_padded(&tilesets.data, filesystem, "Tilesets.rxdata")
                .context("while saving tileset data")?;
        }
        let mut troops = troops.borrow_mut();
        if troops.modified {
            troops.modified = false;
            write_nil_padded(&troops.data, filesystem, "Troops.rxdata")
                .context("while saving troop data")?;
        }
        let mut weapons = weapons.borrow_mut();
        if weapons.modified {
            weapons.modified = false;
            write_nil_padded(&weapons.data, filesystem, "Weapons.rxdata")
                .context("while saving weapons data")?;
        }

        let mut map_infos = map_infos.borrow_mut();
        if map_infos.modified {
            map_infos.modified = false;
            write_data(&map_infos.data, filesystem, "MapInfos.rxdata")
                .context("while saving map infos")?;
        }

        let system = system.get_mut();
        system.magic_number = rand::random();
        write_data(system, filesystem, "System.rxdata").context("while saving system")?;

        let mut scripts = scripts.borrow_mut();
        if scripts.modified {
            scripts.modified = false;
            write_data(
                &scripts.data,
                filesystem,
                format!("{}.rxdata", config.project.scripts_path),
            )?;
        }

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
