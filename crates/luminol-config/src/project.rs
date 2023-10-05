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
use serde::{Deserialize, Serialize};

use super::{command_db, RGSSVer, RMVer};

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub struct Config {
    pub project: Project,
    pub command_db: command_db::CommandDB,
    pub game_ini: ini::Ini,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
/// Local luminol project config
#[allow(missing_docs)]
pub struct Project {
    pub project_name: String,
    pub scripts_path: String,
    pub use_ron: bool,
    pub rgss_ver: RGSSVer,
    pub editor_ver: RMVer,
    pub playtest_exe: String,
    pub prefer_rgssad: bool,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            scripts_path: "Scripts".to_string(),
            use_ron: false,
            rgss_ver: RGSSVer::RGSS1,
            editor_ver: RMVer::XP,
            playtest_exe: "game".to_string(),
            prefer_rgssad: false,
        }
    }
}

impl Config {
    pub fn load(filesystem: &impl luminol_core::filesystem::FileSystem) -> Result<Self, String> {
        if !filesystem.exists(".luminol").map_err(|e| e.to_string())? {
            filesystem
                .create_dir(".luminol")
                .map_err(|e| e.to_string())?;
        }

        let project = match filesystem
            .read_to_string(".luminol/config")
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
        {
            Some(c) => c,
            None => {
                let Some(editor_ver) = RMVer::detect_from_filesystem(filesystem) else {
                    return Err("Unable to detect RPG Maker version".to_string());
                };
                let config = Project {
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
                let command_db = command_db::CommandDB::new(project.editor_ver);
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
                    .set("Scripts", &project.scripts_path)
                    .set("Title", &project.project_name)
                    .set("RTP1", "")
                    .set("RTP2", "")
                    .set("RTP3", "");

                ini
            }
        };

        Ok(Self {
            project,
            command_db,
            game_ini,
        })
    }

    pub fn save(
        &self,
        filesystem: &impl luminol_core::filesystem::FileSystem,
    ) -> Result<(), String> {
        filesystem
            .write(
                ".luminol/commands",
                ron::to_string(&self.command_db).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        filesystem
            .write(
                ".luminol/config",
                ron::to_string(&self.project).map_err(|e| e.to_string())?,
            )
            .map_err(|e| e.to_string())?;

        let mut ini_file = filesystem
            .open_file("Game.ini", luminol_core::filesystem::OpenFlags::Write | luminol_core::filesystem::OpenFlags::Create)
            .map_err(|e| e.to_string())?;
        self.game_ini
            .write_to(&mut ini_file)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
