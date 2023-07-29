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
use crate::prelude::*;
use once_cell::sync::Lazy;

mod command_db;
pub mod global;
pub mod project;

pub use command_db::CommandDB;
pub use global::Config;

#[derive(Default, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Project {
    #[default]
    Unloaded,
    Loaded {
        config: project::Config,
        command_db: CommandDB,
        game_ini: ini::Ini,
    },
}

impl Project {
    pub fn load() -> Result<(), String> {
        let filesystem = &state!().filesystem;

        if !filesystem.exists(".luminol").map_err(|e| e.to_string())? {
            filesystem
                .create_dir(".luminol")
                .map_err(|e| e.to_string())?;
        }

        let config = match filesystem
            .read_to_string(".luminol/config")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let Some(editor_ver) = filesystem.detect_rm_ver() else {
                    return Err("Unable to detect RPG Maker version".to_string());
                };
                let config = project::Config {
                    editor_ver,
                    ..Default::default()
                };
                filesystem
                    .write(".luminol/config", ron::to_string(&config).unwrap())
                    .map_err(|e| e.to_string())?;
                config
            }
        };

        let command_db = match filesystem
            .read_to_string(".luminol/commands")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let command_db = CommandDB::new(config.editor_ver);
                filesystem
                    .write(".luminol/commands", ron::to_string(&command_db).unwrap())
                    .map_err(|e| e.to_string())?;
                command_db
            }
        };

        let game_ini = match filesystem
            .read_to_string("Game.ini")
            .ok()
            .and_then(|i| ini::Ini::load_from_str_noescape(&i).ok())
        {
            Some(i) => i,
            None => {
                let mut ini = ini::Ini::new();
                ini.with_section(Some("Game"))
                    .set("Library", "RGSS104E.dll")
                    .set("Scripts", &config.scripts_path)
                    .set("Title", &config.project_name)
                    .set("RTP1", "")
                    .set("RTP2", "")
                    .set("RTP3", "");

                ini
            }
        };

        *PROJECT.borrow_mut() = Project::Loaded {
            config,
            command_db,
            game_ini,
        };

        Ok(())
    }

    pub fn save() -> Result<(), String> {
        match &*PROJECT.borrow() {
            Project::Unloaded => return Err("Project not loaded".to_string()),
            Project::Loaded {
                config,
                command_db,
                game_ini,
            } => {
                state!()
                    .filesystem
                    .write(
                        ".luminol/commands",
                        ron::to_string(command_db).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;

                state!()
                    .filesystem
                    .write(
                        ".luminol/config",
                        ron::to_string(config).map_err(|e| e.to_string())?,
                    )
                    .map_err(|e| e.to_string())?;

                let mut ini_file = state!()
                    .filesystem
                    .open_file("Game.ini", filesystem::OpenFlags::Create)
                    .map_err(|e| e.to_string())?;
                game_ini
                    .write_to(&mut ini_file)
                    .map_err(|e| e.to_string())?;
            }
        }

        Ok(())
    }
}

pub static PROJECT: AtomicRefCell<Project> = AtomicRefCell::new(Project::Unloaded);

#[macro_export]
macro_rules! project_config {
    () => {{
        AtomicRefMut::map($crate::config::PROJECT.borrow_mut(), |c| match c {
            $crate::config::Project::Unloaded => panic!("Project not loaded"),
            $crate::config::Project::Loaded { config, .. } => config,
        })
    }};
}

#[macro_export]
macro_rules! command_db {
    () => {
        AtomicRefMut::map($crate::config::PROJECT.borrow_mut(), |c| match c {
            $crate::config::Project::Unloaded => panic!("Project not loaded"),
            $crate::config::Project::Loaded { command_db, .. } => command_db,
        })
    };
}

#[macro_export]
macro_rules! game_ini {
    () => {
        AtomicRefMut::map($crate::config::PROJECT.borrow_mut(), |c| match c {
            $crate::config::Project::Unloaded => panic!("Project not loaded"),
            $crate::config::Project::Loaded { game_ini, .. } => game_ini,
        })
    };
}

pub static GLOBAL: Lazy<AtomicRefCell<global::Config>> =
    Lazy::new(|| AtomicRefCell::new(global::Config::new()));

#[macro_export]
macro_rules! global_config {
    () => {
        $crate::config::GLOBAL.borrow_mut()
    };
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
    strum::Display,
    Debug
)]
#[allow(missing_docs)]
pub enum RGSSVer {
    #[strum(to_string = "ModShot")]
    ModShot,
    #[strum(to_string = "mkxp-oneshot")]
    MKXPOneShot,
    #[strum(to_string = "rsgss")]
    RSGSS,
    #[strum(to_string = "mkxp")]
    MKXP,
    #[strum(to_string = "mkxp-freebird")]
    MKXPFreebird,
    #[strum(to_string = "mkxp-z")]
    MKXPZ,
    #[strum(to_string = "Stock RGSS1")]
    RGSS1,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::EnumIter,
    strum::Display,
    Default,
    Debug
)]
#[allow(missing_docs)]
pub enum RMVer {
    #[default]
    #[strum(to_string = "RPG Maker XP")]
    XP = 1,
    #[strum(to_string = "RPG Maker VX")]
    VX = 2,
    #[strum(to_string = "RPG Maker VX Ace")]
    Ace = 3,
}
