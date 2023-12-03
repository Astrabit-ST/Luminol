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

pub struct SoundTab {
    /// The source for this tab.
    pub source: luminol_audio::Source,
    pub volume: u8,
    pub pitch: u8,
    pub selected_track: String,
    folder_children: Vec<luminol_filesystem::DirEntry>,
}

impl SoundTab {
    /// Create a new SoundTab
    pub fn new(
        filesystem: &impl luminol_filesystem::FileSystem,
        source: luminol_audio::Source,
    ) -> Self {
        let folder_children = filesystem.read_dir(format!("Audio/{source}")).unwrap();
        Self {
            source,
            volume: 100,
            pitch: 100,
            selected_track: String::new(),
            folder_children,
        }
    }

    /// Display this SoundTab.
    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
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

                            if let Err(e) = update_state.audio.play(
                                path,
                                update_state.filesystem,
                                volume,
                                pitch,
                                source,
                            ) {
                                update_state.toasts.error(e.to_string());
                            }
                        }

                        if ui.button("Stop").clicked() {
                            // Stop sound.
                            update_state.audio.stop(&self.source);
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
                            update_state.audio.set_volume(self.volume, &self.source);
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
                            update_state.audio.set_pitch(self.pitch, &self.source);
                        };
                    });
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get row height.
            let row_height = ui.text_style_height(&egui::TextStyle::Body);
            // Group together so it looks nicer.
            ui.group(|ui| {
                egui::ScrollArea::both()
                    .id_source(self.source)
                    .auto_shrink([false, false])
                    // Show only visible rows.
                    .show_rows(
                        ui,
                        row_height,
                        self.folder_children.len(),
                        |ui, row_range| {
                            for entry in &self.folder_children[row_range] {
                                // FIXME: Very hacky
                                // Did the user double click a sound?
                                if ui
                                    .selectable_value(
                                        &mut self.selected_track,
                                        entry.file_name().to_string(),
                                        entry.file_name(),
                                    )
                                    .double_clicked()
                                {
                                    // Play it if they did.
                                    let path =
                                        format!("Audio/{}/{}", self.source, &self.selected_track);
                                    let pitch = self.pitch;
                                    let volume = self.volume;
                                    let source = self.source;

                                    if let Err(e) = update_state.audio.play(
                                        path,
                                        update_state.filesystem,
                                        volume,
                                        pitch,
                                        source,
                                    ) {
                                        update_state.toasts.error(e.to_string());
                                    }
                                };
                            }
                        },
                    );
            });
        });
    }
}
