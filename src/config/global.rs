// Copyright (C) 2022 Lily Lyons
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
use std::collections::VecDeque;

/// The state saved by Luminol between sessions.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
#[serde(default)]
pub struct Config {
    /// Recently open projects.
    pub recent_projects: VecDeque<String>,
    /// The current code theme
    pub theme: syntax_highlighting::CodeTheme,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub const fn new() -> Self {
        Self {
            recent_projects: VecDeque::new(),
            theme: syntax_highlighting::CodeTheme::dark(),
        }
    }
}
