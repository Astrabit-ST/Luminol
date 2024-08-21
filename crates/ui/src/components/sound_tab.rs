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

use super::UiExt;

pub struct SoundTab {
    /// The source for this tab.
    pub source: luminol_audio::Source,
    pub audio_file: luminol_data::rpg::AudioFile,

    search_text: String,
    folder_children: Vec<luminol_filesystem::DirEntry>,
    filtered_children: Vec<luminol_filesystem::DirEntry>,

    scrolled_on_first_open: bool,
}

impl SoundTab {
    /// Create a new SoundTab
    pub fn new(
        filesystem: &impl luminol_filesystem::FileSystem,
        source: luminol_audio::Source,
        audio_file: luminol_data::rpg::AudioFile,
    ) -> Self {
        let mut folder_children = filesystem
            .read_dir(format!("Audio/{source}"))
            .unwrap_or_default();
        folder_children.sort_unstable_by(|a, b| {
            lexical_sort::natural_lexical_cmp(a.file_name(), b.file_name())
        });
        Self {
            source,
            audio_file,

            filtered_children: folder_children.clone(),
            search_text: String::new(),
            folder_children,

            scrolled_on_first_open: false,
        }
    }

    fn play(&self, update_state: &mut luminol_core::UpdateState<'_>) {
        if let Some(track) = &self.audio_file.name {
            let path = camino::Utf8Path::new("Audio")
                .join(self.source.as_path())
                .join(track);
            let pitch = self.audio_file.pitch;
            let volume = self.audio_file.volume;
            let source = self.source;

            if let Err(e) = update_state.audio.play(
                path,
                update_state.filesystem,
                volume,
                pitch,
                Some(source),
                update_state
                    .project_config
                    .as_ref()
                    .expect("project not loaded")
                    .project
                    .volume_scale,
            ) {
                luminol_core::error!(
                    update_state.toasts,
                    e.wrap_err("Error playing from audio file")
                );
            }
        } else {
            update_state.audio.stop(self.source);
        }
    }

    /// Display this SoundTab.
    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
        egui::SidePanel::right("sound_tab_controls")
            .resizable(false)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() {
                            self.play(update_state);
                        }

                        if ui.button("Stop").clicked() {
                            // Stop sound.
                            update_state.audio.stop(self.source);
                        }
                    });

                    ui.horizontal(|ui| {
                        let step = ui
                            .input(|i| i.modifiers.shift)
                            .then_some(5.0)
                            .unwrap_or_default();

                        let slider = egui::Slider::new(&mut self.audio_file.volume, 0..=100)
                            .orientation(egui::SliderOrientation::Vertical)
                            .step_by(step)
                            .text("Volume");
                        // Add a slider.
                        // If it's changed, update the volume.
                        if ui.add(slider).changed() {
                            update_state.audio.set_volume(
                                self.audio_file.volume,
                                self.source,
                                update_state
                                    .project_config
                                    .as_ref()
                                    .expect("project not loaded")
                                    .project
                                    .volume_scale,
                            );
                        };

                        let slider = egui::Slider::new(&mut self.audio_file.pitch, 50..=150)
                            .orientation(egui::SliderOrientation::Vertical)
                            .step_by(step)
                            .text("Pitch");
                        // Add a pitch slider.
                        // If it's changed, update the pitch.
                        if ui.add(slider).changed() {
                            update_state
                                .audio
                                .set_pitch(self.audio_file.pitch, self.source);
                        };
                    });
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // Get row height.
            let row_height = ui.spacing().interact_size.y.max(
                ui.text_style_height(&egui::TextStyle::Button) + 2. * ui.spacing().button_padding.y,
            );

            let persistence_id = update_state
                .project_config
                .as_ref()
                .expect("project not loaded")
                .project
                .persistence_id;

            // Group together so it looks nicer.
            ui.group(|ui| {
                let out = egui::TextEdit::singleline(&mut self.search_text)
                    .hint_text("Search 🔎")
                    .show(ui);
                if out.response.changed() {
                    let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                    self.filtered_children = self
                        .folder_children
                        .iter()
                        .filter(|entry| {
                            matcher
                                .fuzzy(entry.file_name(), &self.search_text, false)
                                .is_some()
                        })
                        .cloned()
                        .collect();
                }
                ui.separator();

                let audio_file_name = self.audio_file.name.as_ref().and_then(|name| {
                    update_state
                        .filesystem
                        .desensitize(
                            camino::Utf8Path::new("Audio")
                                .join(self.source.as_path())
                                .join(name),
                        )
                        .ok()
                        .map(|path| camino::Utf8PathBuf::from(path.file_name().unwrap()))
                });

                let mut scroll_area_output = egui::ScrollArea::vertical()
                    .id_source((persistence_id, self.source))
                    .auto_shrink([false, true])
                    // Show only visible rows.
                    .show_rows(
                        ui,
                        row_height,
                        self.filtered_children.len() + 1, // +1 for (None)
                        |ui, mut row_range| {
                            ui.with_cross_justify(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                                // we really want to only show (None) if it's in range, we can collapse this but itd rely on short circuiting
                                #[allow(clippy::collapsible_if)]
                                if row_range.contains(&0) {
                                    if ui
                                        .selectable_value(&mut self.audio_file.name, None, "(None)")
                                        .double_clicked()
                                    {
                                        self.play(update_state);
                                    }
                                }
                                // subtract 1 to account for (None)
                                row_range.start = row_range.start.saturating_sub(1);
                                row_range.end = row_range.end.saturating_sub(1);
                                for (i, entry) in
                                    self.filtered_children[row_range.clone()].iter().enumerate()
                                {
                                    let faint = (i + row_range.start) % 2 == 0;
                                    let res = ui.with_stripe(faint, |ui| {
                                        let entry_name = camino::Utf8Path::new(entry.file_name());
                                        let res = ui.add(egui::SelectableLabel::new(
                                            audio_file_name.as_deref() == Some(entry_name),
                                            entry_name.as_str(),
                                        ));
                                        if res.clicked() {
                                            self.audio_file.name = Some(
                                                entry_name
                                                    .file_stem()
                                                    .unwrap_or(entry_name.as_str())
                                                    .into(),
                                            );
                                        }
                                        res
                                    });
                                    // need to move this out because the borrow checker isn't smart enough
                                    // Did the user double click a sound?
                                    if res.inner.double_clicked() {
                                        // Play it if they did.
                                        self.play(update_state);
                                    };
                                }
                            });
                        },
                    );

                // Scroll the selected item into view
                if !self.scrolled_on_first_open {
                    let row = if self.audio_file.name.is_none() {
                        Some(0)
                    } else {
                        self.filtered_children
                            .iter()
                            .enumerate()
                            .find_map(|(i, entry)| {
                                (audio_file_name.as_deref() == Some(entry.file_name().into()))
                                    .then_some(i + 1)
                            })
                    };
                    if let Some(row) = row {
                        let spacing = ui.spacing().item_spacing.y;
                        let max = row as f32 * (row_height + spacing) + spacing;
                        let min = row as f32 * (row_height + spacing) + row_height
                            - spacing
                            - scroll_area_output.inner_rect.height();
                        if scroll_area_output.state.offset.y > max {
                            scroll_area_output.state.offset.y = max;
                            scroll_area_output
                                .state
                                .store(ui.ctx(), scroll_area_output.id);
                        } else if scroll_area_output.state.offset.y < min {
                            scroll_area_output.state.offset.y = min;
                            scroll_area_output
                                .state
                                .store(ui.ctx(), scroll_area_output.id);
                        }
                    }
                    self.scrolled_on_first_open = true;
                }
            });
        });
    }
}
