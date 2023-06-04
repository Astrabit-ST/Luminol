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

use crate::prelude::*;
use core::ops::{Deref, DerefMut};

#[derive(Default, Debug)]
pub struct Cache {
    state: AtomicRefCell<State>,
}

// Loaded is used 99% of the time. The size discrepancy is okay.
#[allow(clippy::large_enum_variant)]
#[derive(Default, Debug)]
enum State {
    #[default]
    Unloaded,
    Loaded {
        actors: AtomicRefCell<rpg::Actors>,
        animations: AtomicRefCell<rpg::Animations>,
        armors: AtomicRefCell<rpg::Armors>,
        classes: AtomicRefCell<rpg::Classes>,
        common_events: AtomicRefCell<rpg::CommonEvents>,
        enemies: AtomicRefCell<rpg::Enemies>,
        items: AtomicRefCell<rpg::Items>,
        mapinfos: AtomicRefCell<rpg::MapInfos>,
        scripts: AtomicRefCell<Vec<rpg::Script>>,
        skills: AtomicRefCell<rpg::Skills>,
        states: AtomicRefCell<rpg::States>,
        system: AtomicRefCell<rpg::System>,
        tilesets: AtomicRefCell<rpg::Tilesets>,
        troops: AtomicRefCell<rpg::Troops>,
        weapons: AtomicRefCell<rpg::Weapons>,

        maps: dashmap::DashMap<i32, rpg::Map>,
    },
}

impl Cache {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(&self) -> Result<(), String> {
        macro_rules! load {
            ($this:ident, $($field:ident, $file:literal),+) => {
                let filesystem = &state!().filesystem;

                let mut scripts = None;
                for file in [project_config!().scripts_path.clone(), "xScripts".to_string(), "Scripts".to_string()] {
                    match filesystem.read_data(format!("Data/{file}.{}", self.rxdata_ext())) {
                        Ok(s) => {
                            scripts = Some(s);
                            project_config!().scripts_path = file;
                            break;
                        },
                        Err(e) => {
                            eprintln!("Error loading scripts: {e:#?}");
                        }
                    }
                }
                let Some(scripts) = scripts else {
                    return Err("Unable to load scripts, tried scripts file from config, then xScripts, then Scripts".to_string());
                };

                *self.state.borrow_mut() = State::Loaded {
                    $(
                        $field: AtomicRefCell::new(
                            filesystem
                                .read_data(format!("Data/{}.{}", $file, self.rxdata_ext()))
                                .map_err(|e| {
                                    format!(concat!("Failed to load ", stringify!($field), ": {}"), e)
                                })?
                        ),
                    )+

                    maps: Default::default(),
                    scripts: AtomicRefCell::new(scripts),
                };
            };
        }

        load! {
            self,
            actors, "Actors",
            animations, "Animations",
            armors, "Armors",
            classes, "Classes",
            common_events, "CommonEvents",
            enemies, "Enemies",
            items, "Items",
            mapinfos, "MapInfos",
            skills, "Skills",
            states, "States",
            system, "System",
            tilesets, "Tilesets",
            troops, "Troops",
            weapons, "Weapons"
        }

        Ok(())
    }

    pub fn rxdata_ext(&self) -> &'static str {
        match project_config!().editor_ver {
            config::RMVer::XP => "rxdata",
            config::RMVer::VX => "rvdata",
            config::RMVer::Ace => "rvdata2",
        }
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub fn save(&self) -> Result<(), String> {
        config::Project::save()?;

        let filesystem = &state!().filesystem;

        let state = self.state.borrow();
        let State::Loaded {
            actors,
            animations,
            armors,
            classes,
            common_events,
            enemies,
            items,
            mapinfos,
            scripts,
            skills,
            states,
            system,
            tilesets,
            troops,
            weapons,
            maps,
        } = &*state else {
            return Err("Project not loaded".to_string());
        };

        let ext = self.rxdata_ext();

        filesystem.save_data(format!("Data/Actors.{ext}"), &*actors.borrow())?;
        filesystem.save_data(format!("Data/Animations.{ext}"), &*animations.borrow())?;
        filesystem.save_data(format!("Data/Armors.{ext}"), &*armors.borrow())?;
        filesystem.save_data(format!("Data/Classes.{ext}"), &*classes.borrow())?;
        filesystem.save_data(format!("Data/CommonEvents.{ext}"), &*common_events.borrow())?;
        filesystem.save_data(format!("Data/Enemies.{ext}"), &*enemies.borrow())?;
        filesystem.save_data(format!("Data/Items.{ext}"), &*items.borrow())?;
        filesystem.save_data(format!("Data/MapInfos.{ext}"), &*mapinfos.borrow())?;
        filesystem.save_data(format!("Data/Scripts.{ext}"), &*scripts.borrow())?;
        filesystem.save_data(format!("Data/Skills.{ext}"), &*skills.borrow())?;
        filesystem.save_data(format!("Data/States.{ext}"), &*states.borrow())?;
        filesystem.save_data(format!("Data/System.{ext}"), &*system.borrow())?;
        filesystem.save_data(format!("Data/Tilesets.{ext}"), &*tilesets.borrow())?;
        filesystem.save_data(format!("Data/Troops.{ext}"), &*troops.borrow())?;
        filesystem.save_data(format!("Data/Weapons.{ext}"), &*weapons.borrow())?;

        for entry in maps.iter() {
            filesystem.save_data(format!("Data/Map{:0>3}.{ext}", entry.key()), entry.value())?
        }
        maps.clear();

        Ok(())
    }

    pub async fn create_project(&self, config: config::project::Config) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path = path.path().join(&config.project_name);
            std::fs::create_dir(&path).map_err(|e| e.to_string())?;

            let filesystem = &state!().filesystem;
            filesystem.start_loading(path); // FIXME: this is lazy, we're telling the filesystem that "hey the project is loaded now lol pls believe us"

            *config::PROJECT.borrow_mut() = config::Project::Loaded {
                command_db: config::CommandDB::new(config.editor_ver),
                config,
            };

            self.setup_defaults();
            self.save()?;
        } else {
            return Err("Cancelled picking a project directory".to_string());
        }

        Ok(())
    }

    /// Setup default values
    // FIXME: Code jank
    pub fn setup_defaults(&self) {
        let mut mapinfos = rpg::MapInfos::new();
        mapinfos.insert(
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
        let mapinfos = mapinfos.into();

        let maps = dashmap::DashMap::new();
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

        let system = rpg::System {
            magic_number: rand::random(),
            ..Default::default()
        }
        .into();

        let scripts = match project_config!().editor_ver {
            config::RMVer::XP => alox_48::from_bytes(include_bytes!("Scripts.rxdata")).unwrap(),
            config::RMVer::VX => todo!(),
            config::RMVer::Ace => todo!(),
        };
        let scripts = AtomicRefCell::new(scripts);

        *self.state.borrow_mut() = State::Loaded {
            actors: AtomicRefCell::new(NilPadded::from(vec![rpg::Actor::default()])),
            animations: AtomicRefCell::new(NilPadded::from(vec![rpg::Animation::default()])),
            armors: AtomicRefCell::new(NilPadded::from(vec![rpg::Armor::default()])),
            classes: AtomicRefCell::new(NilPadded::from(vec![rpg::Class::default()])),
            common_events: AtomicRefCell::new(NilPadded::from(vec![rpg::CommonEvent::default()])),
            enemies: AtomicRefCell::new(NilPadded::from(vec![rpg::Enemy::default()])),
            items: AtomicRefCell::new(NilPadded::from(vec![rpg::Item::default()])),
            skills: AtomicRefCell::new(NilPadded::from(vec![rpg::Skill::default()])),
            states: AtomicRefCell::new(NilPadded::from(vec![rpg::State::default()])),
            tilesets: AtomicRefCell::new(NilPadded::from(vec![rpg::Tileset::default()])),
            troops: AtomicRefCell::new(NilPadded::from(vec![rpg::Troop::default()])),
            weapons: AtomicRefCell::new(NilPadded::from(vec![rpg::Weapon::default()])),

            mapinfos,
            maps,
            system,
            scripts,
        };
    }
}

macro_rules! nested_ref_getter {
    ($(
        $typ:ty, $name:ident, $($enum_type:ident :: $variant:ident),+
    );*) => {
        $(
            #[allow(unsafe_code, dead_code)]
            pub fn $name<'a>(&'a self) -> impl core::ops::Deref<Target = $typ> + core::ops::DerefMut + 'a {
                struct _Ref<'b> {
                    _this_ref: atomic_refcell::AtomicRef<'b, State>,
                    _other_ref: atomic_refcell::AtomicRefMut<'b, $typ>,
                }
                impl<'b> core::ops::Deref for _Ref<'b> {
                    type Target = $typ;

                    fn deref(&self) -> &Self::Target {
                        &self._other_ref
                    }
                }
                impl<'b> core::ops::DerefMut for _Ref<'b> {
                    fn deref_mut(&mut self) -> &mut Self::Target {
                        &mut self._other_ref
                    }
                }

                let _this_ref = self.state.borrow();
                let _other_ref: atomic_refcell::AtomicRefMut<'a, $typ> = unsafe {
                    // See Self::map for safety
                    match &*(&*_this_ref as *const _) {
                        $(
                            $enum_type::$variant { $name, .. } => $name.borrow_mut(),
                        )+
                        _ => panic!("Project not loaded"),
                    }
                };

                _Ref {
                    _this_ref,
                    _other_ref,
                }
            }
        )+
    };

}

impl Cache {
    nested_ref_getter! {
        rpg::Actors, actors, State::Loaded;
        rpg::Animations, animations, State::Loaded;
        rpg::Armors, armors, State::Loaded;
        rpg::Classes, classes, State::Loaded;
        rpg::CommonEvents, common_events, State::Loaded;
        rpg::Enemies, enemies, State::Loaded;
        rpg::Items, items, State::Loaded;
        rpg::MapInfos, mapinfos, State::Loaded;
        Vec<rpg::Script>, scripts, State::Loaded;
        rpg::Skills, skills, State::Loaded;
        rpg::States, states, State::Loaded;
        rpg::System, system, State::Loaded;
        rpg::Tilesets, tilesets, State::Loaded;
        rpg::Troops, troops, State::Loaded;
        rpg::Weapons, weapons, State::Loaded
    }

    /// Load a map.
    #[allow(unsafe_code)]
    #[allow(clippy::panic)]
    pub fn map<'a>(&'a self, id: i32) -> impl Deref<Target = rpg::Map> + DerefMut + 'a {
        struct Ref<'b> {
            _state: atomic_refcell::AtomicRef<'b, State>,
            map_ref: dashmap::mapref::one::RefMut<'b, i32, rpg::Map>,
        }
        impl<'b> Deref for Ref<'b> {
            type Target = rpg::Map;

            fn deref(&self) -> &Self::Target {
                &self.map_ref
            }
        }
        impl<'b> DerefMut for Ref<'b> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.map_ref
            }
        }

        let state = self.state.borrow();
        let State::Loaded { ref maps, ..} = &*state else {
                panic!("project not loaded")
            };
        //? # SAFETY
        // For starters, this has been tested against miri. Miri is okay with it.
        // Ref is self referential- map_ref borrows from _state. We need to store _state so it gets dropped at the same time as map_ref.
        // If it didn't, map_ref would invalidate the refcell. We could unload the project, changing State while map_ref is live. Storing _state prevents this.
        // Because the rust borrow checker isn't smart enough for this, we need to create an unbounded reference to maps to get a map out. We're not actually using this reference
        // for any longer than it would be valid for (notice the fact that we assign map_ref a lifetime of 'a, which is the lifetime it should have anyway) so this is okay.
        let map_ref: dashmap::mapref::one::RefMut<'a, _, _> = unsafe {
            let unsafe_maps_ref: &dashmap::DashMap<i32, rpg::Map> = &*(maps as *const _);
            unsafe_maps_ref
                .entry(id)
                .or_try_insert_with(|| {
                    state!()
                        .filesystem
                        .read_data(format!("Data/Map{id:0>3}.{}", self.rxdata_ext()))
                })
                .expect("failed to load map") // FIXME
        };

        Ref {
            _state: state,
            map_ref,
        }
    }
}
