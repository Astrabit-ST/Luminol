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
use strum::EnumIter;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
/// Local luminol project config
#[allow(missing_docs)]
pub struct LocalConfig {
    pub project_name: String,
    pub scripts_path: String,
    pub use_ron: bool,
    pub rgss_ver: RGSSVer,
    pub editor_ver: RMVer,
    pub playtest_exe: String,
    pub prefer_rgssad: bool,
}

impl Default for LocalConfig {
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

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, strum::Display, Debug)]
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
    Clone, Copy, PartialEq, Eq, Serialize, Deserialize, EnumIter, strum::Display, Default, Debug,
)]
#[allow(missing_docs)]
pub enum RMVer {
    #[default]
    #[strum(to_string = "RPG Maker XP")]
    XP,
    #[strum(to_string = "RPG Maker VX")]
    VX,
    #[strum(to_string = "RPG Maker VX Ace")]
    Ace,
}
