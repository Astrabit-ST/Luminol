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
use luminol_core::prelude::*;

pub struct Modal {
    state: State,
    id_source: egui::Id,
    source: Source,
}

enum State {
    Closed,
    Open { tab: luminol_components::SoundTab },
}

impl Modal {
    pub fn new(source: Source, id_source: impl Into<egui::Id>) -> Self {
        Self {
            state: State::Closed,
            id_source: id_source.into(),
            source,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data<'m> = &'m mut luminol_data::rpg::AudioFile;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| {
            let button_text = if let Some(track) = &data.name {
                format!("Audio/{}/{}", self.source, track)
            } else {
                "(None)".to_string()
            };

            let mut button_response = ui.button(button_text);

            if button_response.clicked() {
                let tab = luminol_components::SoundTab::new(
                    update_state.filesystem,
                    self.source,
                    data.clone(),
                );
                self.state = State::Open { tab };
            }
            if self.show_window(update_state, ui.ctx(), data) {
                button_response.mark_changed()
            }

            button_response
        }
    }

    fn reset(&mut self, _: &mut luminol_core::UpdateState<'_>, _data: Self::Data<'_>) {
        // we don't need to do much here
        self.state = State::Closed;
    }
}

impl Modal {
    pub fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut luminol_data::rpg::AudioFile,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open { tab } = &mut self.state else {
            return false;
        };

        egui::Window::new("Sound Picker")
            .open(&mut win_open)
            .id(self.id_source.with("window"))
            .show(ctx, |ui| {
                egui::TopBottomPanel::bottom(self.id_source.with("bottom_panel")).show_inside(
                    ui,
                    |ui| {
                        ui.add_space(1.0);
                        luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                    },
                );

                tab.ui(ui, update_state);
            });

        if needs_save {
            *data = tab.audio_file.clone();
        }

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }
        needs_save
    }
}
