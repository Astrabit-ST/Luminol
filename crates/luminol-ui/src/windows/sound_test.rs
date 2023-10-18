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

use crate::prelude::*;

/// A tab for a sound (be it BGM, ME, SE, etc)
/// Optionally can be in 'picker' mode to pick a sound effect.

/// A simple sound test window.
pub struct Window {
    sources: Vec<SoundTab>,
    selected_source: audio::Source,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            // Create all sources.
            sources: audio::Source::iter()
                .map(|s| SoundTab::new(s, false))
                .collect(),
            // By default, bgm is selected.
            selected_source: audio::Source::BGM,
        }
    }
}

impl super::window::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("Sound Test")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
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
                .ui(ui);
        });
    }

    // Technically we don't need the cache, but we do rely on the project being open.
    fn requires_filesystem(&self) -> bool {
        true
    }
}
