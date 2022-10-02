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

use strum::IntoEnumIterator;

use crate::audio::Source;
use crate::UpdateInfo;

/// A tab for a sound (be it BGM, ME, SE, etc)
/// Optionally can be in 'picker' mode to pick a sound effect.
pub struct SoundTab {
    picker: bool,
    pub source: Source,
    volume: u8,
    pitch: u8,
    selected_track: String,
}

impl SoundTab {
    pub fn new(source: Source, picker: bool) -> Self {
        Self {
            picker,
            source,
            volume: 100,
            pitch: 100,
            selected_track: "".to_string(),
        }
    }

    pub fn ui(&mut self, info: &UpdateInfo<'_>, ui: &mut egui::Ui) {
        egui::SidePanel::right("sound_tab_controls")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() && !self.selected_track.is_empty() {
                            self.play(info);
                        }

                        if ui.button("Stop").clicked() {
                            // Stop sound.
                            info.audio.stop(&self.source);
                        }
                    });

                    ui.horizontal(|ui| {
                        // Add a slider.
                        // If it's changed, update the volume.
                        if ui
                            .add(
                                egui::Slider::new(&mut self.volume, 0..=100)
                                    .orientation(egui::SliderOrientation::Vertical)
                                    .text("Volume"),
                            )
                            .changed()
                        {
                            info.audio.set_volume(self.volume, &self.source);
                        };
                        // Add a pitch slider.
                        // If it's changed, update the pitch.
                        if ui
                            .add(
                                egui::Slider::new(&mut self.pitch, 50..=150)
                                    .orientation(egui::SliderOrientation::Vertical)
                                    .text("Pitch"),
                            )
                            .changed()
                        {
                            info.audio.set_pitch(self.pitch, &self.source);
                        };
                    });

                    if self.picker {
                        ui.horizontal(|ui| {
                            if ui.button("Cancel").clicked() {}
                            if ui.button("Ok").clicked() {}
                        });
                    }
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get folder children.
            let folder_children: Vec<_> = match info
                .filesystem
                .dir_children(&format!("Audio/{}", self.source))
            {
                Ok(d) => d.collect(),
                Err(e) => {
                    info.toasts.error(e);
                    return;
                }
            };

            // Get row height.
            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            // Group together so it looks nicer.
            ui.group(|ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    // Show only visible rows.
                    .show_rows(ui, row_height, folder_children.len(), |ui, row_range| {
                        for entry in &folder_children[row_range] {
                            // FIXME: Very hacky
                            let str = entry
                                .as_ref()
                                .expect("There should be an entry here.")
                                .file_name()
                                .into_string()
                                .expect("Failed to convert path into a UTF-8 string.");
                            // Did the user double click a sound?
                            if ui
                                .selectable_value(&mut self.selected_track, str.clone(), str)
                                .double_clicked()
                            {
                                // Play it if they did.
                                self.play(info);
                            };
                        }
                    });
            });
        });
    }

    fn play(&self, info: &UpdateInfo<'_>) {
        // Get path.
        let path = format!("Audio/{}/{}", self.source, &self.selected_track);
        // Play it.
        info.audio
            .play(info, &path, self.volume, self.pitch, &self.source);
    }
}

/// A simple sound test window.
pub struct SoundTest {
    sources: Vec<SoundTab>,
    selected_source: Source,
}

impl SoundTest {
    pub fn new() -> Self {
        Self {
            // Create all sources.
            sources: Source::iter().map(|s| SoundTab::new(s, false)).collect(),
            // By default, bgm is selected.
            selected_source: Source::BGM,
        }
    }
}

impl super::window::Window for SoundTest {
    fn name(&self) -> String {
        "Sound Test".to_string()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &UpdateInfo<'_>) {
        egui::Window::new("Sound Test").open(open).show(ctx, |ui| {
            egui::TopBottomPanel::top("sound_test_selector").show_inside(ui, |ui| {
                // Display the tab selector.
                ui.horizontal_wrapped(|ui| {
                    for source in self.sources.iter() {
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
                .ui(info, ui);
        });
    }

    // Technically we don't need the cache, but we do rely on the project being open.
    fn requires_filesystem(&self) -> bool {
        true
    }
}
