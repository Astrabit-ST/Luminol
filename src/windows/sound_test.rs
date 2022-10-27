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

use poll_promise::Promise;
use strum::IntoEnumIterator;

use crate::audio::Source;
use crate::UpdateInfo;

/// A tab for a sound (be it BGM, ME, SE, etc)
/// Optionally can be in 'picker' mode to pick a sound effect.
pub struct SoundTab {
    picker: bool,
    /// The source for this tab.
    pub source: Source,
    volume: u8,
    pitch: u8,
    selected_track: String,
    folder_children: Promise<Vec<String>>,
    play_promise: Option<Promise<()>>,
}

impl SoundTab {
    /// Create a new SoundTab
    pub fn new(source: Source, info: &'static UpdateInfo, picker: bool) -> Self {
        Self {
            picker,
            source,
            volume: 100,
            pitch: 100,
            selected_track: "".to_string(),
            folder_children: Promise::spawn_local(async move {
                info.filesystem
                    .dir_children(&format!("Audio/{source}"))
                    .await
                    .unwrap()
            }),
            play_promise: None,
        }
    }

    /// Display this SoundTab.
    pub fn ui(&mut self, info: &'static UpdateInfo, ui: &mut egui::Ui) {
        egui::SidePanel::right("sound_tab_controls")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() && !self.selected_track.is_empty() {
                            let path = format!("Audio/{}/{}", self.source, &self.selected_track);
                            let pitch = self.pitch;
                            let volume = self.volume;
                            let source = self.source;
                            // Play it.
                            self.play_promise = Some(Promise::spawn_local(async move {
                                if let Err(e) = info
                                    .audio
                                    .play(&info.filesystem, path, volume, pitch, source)
                                    .await
                                {
                                    info.toasts.error(e);
                                }
                            }));
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
            if let Some(folder_children) = self.folder_children.ready() {
                // Get row height.
                let row_height = ui.text_style_height(&egui::TextStyle::Body);
                // Group together so it looks nicer.
                ui.group(|ui| {
                    egui::ScrollArea::both()
                        .id_source(self.source)
                        .auto_shrink([false, false])
                        // Show only visible rows.
                        .show_rows(ui, row_height, folder_children.len(), |ui, row_range| {
                            for entry in &folder_children[row_range] {
                                // FIXME: Very hacky
                                // Did the user double click a sound?
                                if ui
                                    .selectable_value(
                                        &mut self.selected_track,
                                        entry.clone(),
                                        entry,
                                    )
                                    .double_clicked()
                                {
                                    // Play it if they did.
                                    let path =
                                        format!("Audio/{}/{}", self.source, &self.selected_track);
                                    let pitch = self.pitch;
                                    let volume = self.volume;
                                    let source = self.source;
                                    // Play it.
                                    self.play_promise = Some(Promise::spawn_local(async move {
                                        if let Err(e) = info
                                            .audio
                                            .play(&info.filesystem, path, volume, pitch, source)
                                            .await
                                        {
                                            info.toasts.error(e);
                                        }
                                    }));
                                };
                            }
                        });
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            }
        });
    }
}

/// A simple sound test window.
pub struct SoundTest {
    sources: Vec<SoundTab>,
    selected_source: Source,
}

impl SoundTest {
    /// Create a new sound test window.
    pub fn new(info: &'static UpdateInfo) -> Self {
        Self {
            // Create all sources.
            sources: Source::iter()
                .map(|s| SoundTab::new(s, info, false))
                .collect(),
            // By default, bgm is selected.
            selected_source: Source::BGM,
        }
    }
}

impl super::window::Window for SoundTest {
    fn name(&self) -> String {
        "Sound Test".to_string()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static UpdateInfo) {
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
