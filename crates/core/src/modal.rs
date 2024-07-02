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

/// A basic trait describing a modal that edits some value.
pub trait Modal: Sized {
    /// The output type for this modal.
    type Data;

    /// Return a widget that displays a button for this modal.
    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut crate::UpdateState<'_>,
    ) -> impl egui::Widget + 'm; // woah rpitit (so cool)

    fn reset(&mut self, update_state: &mut crate::UpdateState<'_>, data: &Self::Data);
}
