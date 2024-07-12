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
use serde::{Deserialize, Serialize};

use super::{command_db, DataFormat, RGSSVer, RMVer};

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
    pub data_format: DataFormat,
    pub rgss_ver: RGSSVer,
    pub editor_ver: RMVer,
    pub playtest_exe: String,
    pub prefer_rgssad: bool,
    pub persistence_id: u64,
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project_name: String::new(),
            scripts_path: "Scripts".to_string(),
            data_format: DataFormat::Marshal,
            rgss_ver: RGSSVer::RGSS1,
            editor_ver: RMVer::XP,
            playtest_exe: "game".to_string(),
            prefer_rgssad: false,
            persistence_id: 0,
        }
    }
}

impl Config {
    pub fn from_project(project: Project) -> Self {
        let mut game_ini = ini::Ini::new();
        game_ini
            .with_section(Some("Game"))
            .set("Library", "RGSS104E.dll")
            .set("Scripts", format!("Data/{}", project.scripts_path))
            .set("Title", &project.project_name)
            .set("RTP1", "")
            .set("RTP2", "")
            .set("RTP3", "");

        let command_db = command_db::CommandDB::new(project.editor_ver);

        Self {
            project,
            command_db,
            game_ini,
        }
    }
}
