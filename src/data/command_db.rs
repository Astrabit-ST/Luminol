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

use command_lib::CommandDescription;
use once_cell::sync::Lazy;

use crate::filesystem::Filesystem;

use super::config::RMVer;

static XP_DEFAULT: Lazy<Vec<CommandDescription>> = Lazy::new(|| {
    ron::from_str(include_str!("xp_default.ron")).expect(
        "failed to statically load the default commands for rpg maker xp. please report this bug",
    )
});

#[derive(Default)]
pub struct CommandDB {
    /// Default commands
    default: Vec<CommandDescription>,
    /// User defined commands
    user: Vec<CommandDescription>,
}

impl CommandDB {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, code: u16) -> Option<&CommandDescription> {
        self.user
            .iter()
            .find(|c| c.code == code)
            .or_else(|| self.default.iter().find(|c| c.code == code))
    }

    pub(super) async fn load(&mut self, filesystem: &impl Filesystem, version: RMVer) {}
}
