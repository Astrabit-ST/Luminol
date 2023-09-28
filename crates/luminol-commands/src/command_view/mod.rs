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

#[macro_use]
mod macros;
mod command_ui;
mod parameter_ui;
mod ui;

pub use crate::prelude::*;
use std::collections::HashMap;

pub struct CommandView {
    selected_index: usize,
    window_state: WindowState,
    id: egui::Id,
    modals: HashMap<u64, bool>, // todo find a better way to handle modals
}

enum WindowState {
    Insert { index: usize, tab: usize },
    Edit { index: usize },
    None,
}

impl Default for CommandView {
    fn default() -> Self {
        Self {
            selected_index: 0,
            window_state: WindowState::None,
            id: egui::Id::new("command_view"),
            modals: HashMap::new(),
        }
    }
}

impl CommandView {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            ..Default::default()
        }
    }
}
