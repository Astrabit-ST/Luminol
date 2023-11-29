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



/// The event editor window.
pub struct Window {
    id: usize,
    map_id: usize,
    selected_page: usize,
    name: String,
    viewed_tab: u8,

    switch_modal_1: Option<luminol_modals::switch::Modal>,
    switch_modal_2: Option<luminol_modals::switch::Modal>,
    variable_modal: Option<luminol_modals::variable::Modal>,
}

impl Window {
    /// Create a new event editor.
    pub fn new(id: usize, map_id: usize) -> Self {
        Self {
            id,
            map_id,
            selected_page: 0,
            name: String::from("(unknown)"),
            viewed_tab: 2,

            switch_modal_1: None,
            switch_modal_2: None,
            variable_modal: None,
        }
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        format!("Event: {}, {} in Map {}", self.name, self.id, self.map_id)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_event_edit")
            .with(self.map_id)
            .with(self.id)
    }

    // This needs an overhaul
    fn show(
        &mut self,
        _ctx: &egui::Context,
        _open: &mut bool,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        todo!()
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
