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

pub struct Variable;

impl super::DatabaseModalHandler for Variable {
    fn button_format(id: &mut usize, update_state: &mut luminol_core::UpdateState<'_>) -> String {
        let system = update_state.data.system();
        *id = system.variables.len().min(*id);
        format!("{:0>3}: {}", *id + 1, system.variables[*id])
    }

    fn window_title() -> &'static str {
        "Variables"
    }

    fn iter(
        update_state: &mut luminol_core::UpdateState<'_>,
        f: impl FnOnce(&mut dyn Iterator<Item = (usize, String)>),
    ) {
        let system = update_state.data.system();
        let mut iter = system
            .variables
            .iter()
            .enumerate()
            .map(|(id, name)| (id, format!("{:0>3}: {name}", id + 1)));
        f(&mut iter);
    }

    fn current_size(update_state: &luminol_core::UpdateState<'_>) -> Option<usize> {
        Some(update_state.data.system().variables.len())
    }

    fn resize(update_state: &mut luminol_core::UpdateState<'_>, new_size: usize) {
        let system = &mut update_state.data.system();
        system.variables.resize_with(new_size, String::new);
    }
}
