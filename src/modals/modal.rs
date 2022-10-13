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

use crate::UpdateInfo;

/// A basic trait describind a modal that edits some value.
/// Modals can be open and will set their open state.
/// They should (generally) respect the open bool passed to them and only show if it is true.
// TODO: Make more featureful and general
pub trait Modal {
    // The output type for this modal.
    type Data;

    // Set the modal Id
    fn id(self, id: egui::Id) -> Self;

    // Display a button to show this modal.
    // It should call show.
    fn button(
        self,
        ui: &mut egui::Ui,
        state: &mut bool,
        data: &mut Self::Data,
        info: &'static UpdateInfo,
    ) -> Self;

    // Show this modal.
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data: &mut Self::Data,
        info: &'static UpdateInfo,
    );
}
