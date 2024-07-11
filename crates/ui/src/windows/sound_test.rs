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

use strum::IntoEnumIterator;

/// A tab for a sound (be it BGM, ME, SE, etc)
/// Optionally can be in 'picker' mode to pick a sound effect.

/// A simple sound test window.
pub struct Window {
    sources: Vec<luminol_components::SoundTab>,
    selected_source: luminol_audio::Source,
}

impl Window {
    pub fn new(filesystem: &impl luminol_filesystem::FileSystem) -> Self {
        Self {
            // Create all sources.
            sources: luminol_audio::Source::iter()
                .map(|s| luminol_components::SoundTab::new(filesystem, s, Default::default()))
                .collect(),
            // By default, bgm is selected.
            selected_source: luminol_audio::Source::BGM,
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("Sound Test")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Sound Test").open(open).show(ctx, |ui| {
            egui::TopBottomPanel::top("sound_test_selector").show_inside(ui, |ui| {
                // Display the tab selector.
                ui.horizontal_wrapped(|ui| {
                    for source in &self.sources {
                        if ui
                            .selectable_label(
                                source.source == self.selected_source,
                                source.source.to_string(),
                            )
                            .clicked()
                        {
                            self.selected_source = source.source;
                        }
                    }
                })
            });

            // We should be finding something. The unwrap is safe here.
            self.sources
                .iter_mut()
                .find(|t| t.source == self.selected_source)
                .unwrap()
                .ui(ui, update_state);
        });
    }

    // Technically we don't need the cache, but we do rely on the project being open.
    fn requires_filesystem(&self) -> bool {
        true
    }
}
