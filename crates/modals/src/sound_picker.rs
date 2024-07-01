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
use luminol_core::prelude::*;

pub struct Modal {
    tab: luminol_components::SoundTab,
    open: bool,
}

impl Modal {
    pub fn new(
        filesystem: &impl FileSystem,
        source: Source,
        selected_track: luminol_data::rpg::AudioFile,
    ) -> Self {
        let tab = luminol_components::SoundTab::new(filesystem, source, selected_track);
        Self { tab, open: false }
    }
}

impl luminol_core::Modal for Modal {
    type Data = luminol_data::rpg::AudioFile;

    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| {
            let button_text = if let Some(track) = &self.tab.audio_file.name {
                format!("Audio/{}/{}", self.tab.source, track)
            } else {
                "(None)".to_string()
            };

            let mut button_response = ui.button(button_text);

            if button_response.clicked() {
                self.open = true;
            }
            if self.show_window(update_state, ui.ctx(), data) {
                button_response.mark_changed()
            }

            button_response
        }
    }

    fn reset(&mut self) {
        self.open = false;
    }
}

impl Modal {
    pub fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut luminol_data::rpg::AudioFile,
    ) -> bool {
        let mut win_open = self.open;
        let mut keep_open = true;
        let mut needs_save = false;

        egui::Window::new("Graphic Picker")
            .open(&mut win_open)
            .show(ctx, |ui| {
                self.tab.ui(ui, update_state);
                ui.separator();

                luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
            });

        if needs_save {
            *data = self.tab.audio_file.clone();
        }

        self.open = win_open && keep_open;
        needs_save
    }
}
