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
        scripts: RefCell<Vec<rpg::Script>>,
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
) -> Result<T, String>
where
    T: serde::de::DeserializeOwned,
{
    let path = camino::Utf8PathBuf::from("Data").join(filename);
    let data = filesystem.read(path).map_err(|e| e.to_string())?;

    alox_48::from_bytes(&data).map_err(|e| e.to_string())
}

fn write_data(
    data: &impl serde::Serialize,
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> Result<(), String> {
    let path = camino::Utf8PathBuf::from("Data").join(filename);

    let bytes = alox_48::to_bytes(data).map_err(|e| e.to_string())?;
    filesystem.write(path, bytes).map_err(|e| e.to_string())
}

fn read_nil_padded<T>(
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> Result<Vec<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    let path = camino::Utf8PathBuf::from("Data").join(filename);
    let data = filesystem.read(path).map_err(|e| e.to_string())?;

    let mut de = alox_48::Deserializer::new(&data).map_err(|e| e.to_string())?;

    luminol_data::helpers::nil_padded::deserialize(&mut de).map_err(|e| e.to_string())
}

fn write_nil_padded(
    data: &[impl serde::Serialize],
    filesystem: &impl luminol_filesystem::FileSystem,
    filename: impl AsRef<camino::Utf8Path>,
) -> Result<(), String> {
    let path = camino::Utf8PathBuf::from("Data").join(filename);

    let mut ser = alox_48::Serializer::new();

    luminol_data::helpers::nil_padded::serialize(data, &mut ser).map_err(|e| e.to_string())?;
    filesystem
        .write(path, ser.output)
        .map_err(|e| e.to_string())
}

impl Data {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(
        &mut self,
        filesystem: &impl luminol_filesystem::FileSystem,
        config: &mut luminol_config::project::Config,
    ) -> Result<(), String> {
        let actors = RefCell::new(read_nil_padded(filesystem, "Actors.rxdata")?);
        let animations = RefCell::new(read_nil_padded(filesystem, "Animations.rxdata")?);
        let armors = RefCell::new(read_nil_padded(filesystem, "Armors.rxdata")?);
        let classes = RefCell::new(read_nil_padded(filesystem, "Classes.rxdata")?);
        let common_events = RefCell::new(read_nil_padded(filesystem, "CommonEvents.rxdata")?);
        let enemies = RefCell::new(read_nil_padded(filesystem, "Enemies.rxdata")?);
        let items = RefCell::new(read_nil_padded(filesystem, "Items.rxdata")?);
        let skills = RefCell::new(read_nil_padded(filesystem, "Skills.rxdata")?);
        let states = RefCell::new(read_nil_padded(filesystem, "States.rxdata")?);
        let tilesets = RefCell::new(read_nil_padded(filesystem, "Tilesets.rxdata")?);
        let troops = RefCell::new(read_nil_padded(filesystem, "Troops.rxdata")?);
        let weapons = RefCell::new(read_nil_padded(filesystem, "Weapons.rxdata")?);

        let map_infos = RefCell::new(read_data(filesystem, "MapInfos.rxdata")?);

        let mut system = read_data::<rpg::System>(filesystem, "System.rxdata")?;
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
                    scripts = Some(s);
                    break;
                }
                Err(e) => {
                    eprintln!("error loading scripts from {script_path}: {e}")
                }
            }
        }
        let Some(scripts) = scripts else {
            return Err("failed to load scripts".to_string());
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

    // TODO dependency cycle
    pub fn from_defaults() -> Self {
        let actors = RefCell::new(vec![rpg::Actor::default()]);
        let animations = RefCell::new(vec![rpg::Animation::default()]);
        let armors = RefCell::new(vec![rpg::Armor::default()]);
        let classes = RefCell::new(vec![rpg::Class::default()]);
        let common_events = RefCell::new(vec![rpg::CommonEvent::default()]);
        let enemies = RefCell::new(vec![rpg::Enemy::default()]);
        let items = RefCell::new(vec![rpg::Item::default()]);
        let skills = RefCell::new(vec![rpg::Skill::default()]);
        let states = RefCell::new(vec![rpg::State::default()]);
        let tilesets = RefCell::new(vec![rpg::Tileset::default()]);
        let troops = RefCell::new(vec![rpg::Troop::default()]);
        let weapons = RefCell::new(vec![rpg::Weapon::default()]);

        let mut map_infos = std::collections::HashMap::with_capacity(16);
        map_infos.insert(1, rpg::MapInfo::default());
        let map_infos = RefCell::new(map_infos);

        let system = rpg::System {
            magic_number: rand::random(),
            ..Default::default()
        };
        let system = RefCell::new(system);

        let scripts = vec![]; // FIXME legality of providing defualt scripts is unclear
        let scripts = RefCell::new(scripts);

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
    ) -> Result<(), String> {
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

        write_nil_padded(actors.get_mut(), filesystem, "Actors.rxdata")?;
        write_nil_padded(animations.get_mut(), filesystem, "Animations.rxdata")?;
        write_nil_padded(armors.get_mut(), filesystem, "Armors.rxdata")?;
        write_nil_padded(classes.get_mut(), filesystem, "Classes.rxdata")?;
        write_nil_padded(common_events.get_mut(), filesystem, "CommonEvents.rxdata")?;
        write_nil_padded(enemies.get_mut(), filesystem, "Enemies.rxdata")?;
        write_nil_padded(items.get_mut(), filesystem, "Items.rxdata")?;
        write_nil_padded(skills.get_mut(), filesystem, "Skills.rxdata")?;
        write_nil_padded(states.get_mut(), filesystem, "States.rxdata")?;
        write_nil_padded(tilesets.get_mut(), filesystem, "Tilesets.rxdata")?;
        write_nil_padded(troops.get_mut(), filesystem, "Troops.rxdata")?;
        write_nil_padded(weapons.get_mut(), filesystem, "Weapons.rxdata")?;

        write_data(map_infos.get_mut(), filesystem, "MapInfos.rxdata")?;

        let system = system.get_mut();
        system.magic_number = rand::random();
        write_data(system, filesystem, "System.rxdata")?;

        write_data(
            scripts.get_mut(),
            filesystem,
            format!("{}.rxdata", config.project.scripts_path),
        )?;

        maps.get_mut()
            .iter()
            .try_for_each(|(id, map)| write_data(map, filesystem, format!("Map{id:0>3}.rxdata")))
    }

    /// Setup default values
    // FIXME: Code jank
    pub fn setup_defaults(&mut self) {
        todo!()
    }
}

macro_rules! nested_ref_getter {
    ($($typ:ty, $name:ident),* $(,)?) => {
        $(
            #[allow(unsafe_code, dead_code)]
            pub fn $name(&self) -> RefMut<$typ> {
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
        Vec<rpg::Script>, scripts,
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
    ) -> RefMut<rpg::Map> {
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

    pub fn get_map(&self, id: usize) -> RefMut<rpg::Map> {
        let maps_ref = match self {
            Self::Loaded { maps, .. } => maps.borrow_mut(),
            Self::Unloaded => panic!("project not loaded"),
        };
        RefMut::map(maps_ref, |maps| maps.get_mut(&id).expect("map not loaded"))
    }
}
