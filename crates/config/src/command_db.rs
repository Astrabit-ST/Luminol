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

use luminol_data::commands::CommandDescription;
use once_cell::sync::Lazy;

use serde::{Deserialize, Serialize};

use super::RMVer;

static XP_DEFAULT: Lazy<Vec<CommandDescription>> = Lazy::new(|| {
    ron::from_str(include_str!("commands/xp.ron")).expect(
        "failed to statically load the default commands for rpg maker xp. please report this bug",
    )
});

static VX_DEFAULT: Lazy<Vec<CommandDescription>> = Lazy::new(|| {
    ron::from_str(include_str!("commands/vx.ron")).expect(
        "failed to statically load the default commands for rpg maker vx. please report this bug",
    )
});

static ACE_DEFAULT: Lazy<Vec<CommandDescription>> = Lazy::new(|| {
    ron::from_str(include_str!("commands/ace.ron")).expect(
        "failed to statically load the default commands for rpg maker vx ace. please report this bug",
    )
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommandDB {
    /// Default commands
    default: Vec<CommandDescription>,
    /// User defined commands
    // FIXME: visible to user?
    pub user: Vec<CommandDescription>,
}

impl CommandDB {
    pub fn new(ver: RMVer) -> Self {
        Self {
            default: match ver {
                RMVer::XP => &*XP_DEFAULT,
                RMVer::VX => &*VX_DEFAULT,
                RMVer::Ace => &*ACE_DEFAULT,
            }
            .clone(),
            user: vec![],
        }
    }

    pub fn get(&self, code: u16) -> Option<&CommandDescription> {
        self.user
            .iter()
            .find(|c| c.code == code)
            .or_else(|| self.default.iter().find(|c| c.code == code))
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandDescription> {
        self.default.iter().chain(self.user.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut CommandDescription> {
        self.default.iter_mut().chain(self.user.iter_mut())
    }

    pub fn len(&self) -> usize {
        self.default.len() + self.user.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
