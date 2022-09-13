use crate::UpdateInfo;
use rodio::{Decoder, OutputStream, Sink};
use strum::Display;
use strum::EnumIter;
use strum::IntoEnumIterator;

/// Different sound sources.
#[derive(EnumIter, Display, PartialEq, Eq, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub enum Source {
    BGM,
    BGS,
    ME,
    SE,
}

/// A tab for a sound (be it BGM, ME, SE, etc)
/// Optionally can be in 'picker' mode to pick a sound effect.
pub struct SoundTab {
    picker: bool,
    pub source: Source,
    volume: u8,
    pitch: u8,
    selected_track: String,
    sink: Sink,
}

impl SoundTab {
    pub fn new(source: Source, picker: bool) -> Self {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();
        Self {
            picker,
            source,
            volume: 100,
            pitch: 100,
            selected_track: "".to_string(),
            sink,
        }
    }

    pub fn ui(&mut self, info: &UpdateInfo<'_>, ui: &mut egui::Ui) {
        let source_str = self.source.to_string();
        egui::SidePanel::right("sound_tab_controls")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() && !self.selected_track.is_empty() {
                            let path = format!("Audio/{}/{}", source_str, &self.selected_track);
                            // Do we need to loop?
                            match self.source {
                                // If not, just play a regular sound:
                                Source::SE | Source::ME => self.sink.append(
                                    Decoder::new(info.filesystem.file(&path))
                                        .expect("Failed to create decoder"),
                                ),
                                // Stop the current track and loop music
                                _ => {
                                    self.sink.stop();
                                    self.sink.append(
                                        Decoder::new_looped(info.filesystem.file(&path))
                                            .expect("Failed to create decoder"),
                                    )
                                }
                            }
                            // Play it.
                            self.sink.play();
                        }

                        if ui.button("Stop").clicked() {
                            // Stop sound.
                            self.sink.stop();
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
                            self.sink.set_volume(self.volume as f32 / 100.0);
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
                            self.sink.set_speed(self.pitch as f32 / 100.0);
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
            let folder_children: Vec<_> = info
                .filesystem
                .dir_children(&format!("Audio/{}", source_str))
                .collect();
            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            ui.group(|ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show_rows(ui, row_height, folder_children.len(), |ui, row_range| {
                        for entry in &folder_children[row_range] {
                            let str = entry
                                .as_ref()
                                .expect("There should be an entry here.")
                                .file_name()
                                .into_string()
                                .expect("Failed to convert path into a UTF-8 string.");
                            ui.selectable_value(&mut self.selected_track, str.clone(), str);
                        }
                    });
            });
        });
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
            sources: Source::iter().map(|s| SoundTab::new(s, false)).collect(),
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
                // Display the
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
