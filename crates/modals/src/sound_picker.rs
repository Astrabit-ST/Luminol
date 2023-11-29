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

pub struct Modal {
    tab: luminol_components::SoundTab,
}

impl luminol_core::Modal for Modal {
    type Data = luminol_data::rpg::AudioFile;

    fn button(
        _this: &mut Option<Self>,
        _ui: &mut egui::Ui,
        _data: &mut Self::Data,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        todo!()
    }

    fn show(
        _this: &mut Option<Self>,
        _ctx: &egui::Context,
        _data: &mut Self::Data,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        todo!()
    }
}
