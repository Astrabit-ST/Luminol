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

#[cfg(not(target_arch = "wasm32"))]
use crate::terminal;
use crate::CodeTheme;
use std::collections::VecDeque;

/// The state saved by Luminol between sessions.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    #[cfg(not(target_arch = "wasm32"))]
    /// Recently open projects.
    pub recent_projects: VecDeque<String>,
    #[cfg(target_arch = "wasm32")]
    /// Recently open projects.
    pub recent_projects: VecDeque<(String, String)>,
    #[cfg(not(target_arch = "wasm32"))]
    pub terminal: terminal::Config,

    /// The current code theme
    pub theme: CodeTheme,
    #[cfg(not(target_arch = "wasm32"))]
    pub rtp_paths: indexmap::IndexMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            recent_projects: VecDeque::new(),
            theme: CodeTheme::dark(),
            #[cfg(not(target_arch = "wasm32"))]
            rtp_paths: indexmap::IndexMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            terminal: terminal::Config::default(),
        }
    }
}
