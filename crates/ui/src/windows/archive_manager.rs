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

/// The archive manager for creating and extracting RGSSAD archives.
#[derive(Default)]
pub struct Window {
    mode: Mode,
    save_promise: Option<
        poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
    >,
}

enum Mode {
    Extract {
        archive: Option<(luminol_filesystem::host::File, String)>,
        load_promise: Option<
            poll_promise::Promise<
                luminol_filesystem::Result<(luminol_filesystem::host::File, String)>,
            >,
        >,
    },
    Create {
        input_folder: Option<(luminol_filesystem::host::FileSystem, String)>,
        load_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
    },
}

impl Default for Mode {
    fn default() -> Self {
        Self::Extract {
            archive: None,
            load_promise: None,
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("RGSSAD Archive Manager")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut window_open = true;
        egui::Window::new("RGSSAD Archive Manager")
            .open(&mut window_open)
            .show(ctx, |ui| {
                ui.columns(2, |columns| {
                    if columns[0]
                        .add(egui::SelectableLabel::new(
                            matches!(self.mode, Mode::Extract { .. }),
                            "Extract from archive",
                        ))
                        .clicked()
                    {
                        self.mode = Mode::Extract {
                            archive: None,
                            load_promise: None,
                        };
                    }
                    if columns[1]
                        .add(egui::SelectableLabel::new(
                            matches!(self.mode, Mode::Create { .. }),
                            "Create new archive",
                        ))
                        .clicked()
                    {
                        self.mode = Mode::Create {
                            input_folder: None,
                            load_promise: None,
                        };
                    }
                });

                ui.separator();

                match &mut self.mode {
                    Mode::Extract {
                        archive,
                        load_promise,
                    } => {
                        if let Some(p) = load_promise.take() {
                            match p.try_take() {
                                Ok(Ok((handle, name))) => *archive = Some((handle, name)),
                                Ok(Err(e)) => {
                                    if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                        update_state.toasts.error(e.to_string())
                                    }
                                }
                                Err(p) => *load_promise = Some(p),
                            }
                        }

                        ui.with_layout(
                            egui::Layout {
                                cross_align: egui::Align::Center,
                                cross_justify: true,
                                ..Default::default()
                            },
                            |ui| {
                                if load_promise.is_none() && ui.button("Select archive").clicked() {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        *load_promise = Some(poll_promise::Promise::spawn_async(
                                            luminol_filesystem::host::File::from_file_picker(
                                                "RGSSAD archives",
                                                &["rgssad", "rgss2a", "rgss3a"],
                                            ),
                                        ));
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        *load_promise = Some(poll_promise::Promise::spawn_local(
                                            luminol_filesystem::host::File::from_file_picker(
                                                "RGSSAD archives",
                                                &["rgssad", "rgss2a", "rgss3a"],
                                            ),
                                        ));
                                    }
                                } else if load_promise.is_some() {
                                    ui.spinner();
                                }
                            },
                        );

                        ui.add(
                            egui::Label::new(if let Some((_, name)) = archive {
                                format!("Archive: {}", name)
                            } else {
                                "No archive selected".into()
                            })
                            .truncate(true),
                        );
                    }

                    _ => todo!("archive creation"),
                }

                ui.colored_label(egui::Color32::RED, "TODO: Put an archive viewer here");

                match &mut self.mode {
                    Mode::Extract {
                        archive,
                        load_promise,
                    } => {
                        if let Some(p) = self.save_promise.take() {
                            match p.try_take() {
                                Ok(Ok(_handle)) => todo!("extract files"),
                                Ok(Err(e)) => {
                                    if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                        update_state.toasts.error(e.to_string())
                                    }
                                }
                                Err(p) => self.save_promise = Some(p),
                            }
                        }

                        ui.with_layout(
                            egui::Layout {
                                cross_align: egui::Align::Center,
                                cross_justify: true,
                                ..Default::default()
                            },
                            |ui| {
                                if self.save_promise.is_none()
                                    && ui
                                        .add_enabled(
                                            archive.is_some() && load_promise.is_none(),
                                            egui::Button::new("Extract"),
                                        )
                                        .clicked()
                                {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    {
                                        self.save_promise = Some(poll_promise::Promise::spawn_async(
                                            luminol_filesystem::host::FileSystem::from_folder_picker(),
                                        ));
                                    }
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        self.save_promise = Some(poll_promise::Promise::spawn_local(
                                            luminol_filesystem::host::FileSystem::from_folder_picker(),
                                        ));
                                    }
                                } else if self.save_promise.is_some() {
                                    ui.spinner();
                                }
                            },
                        );
                    }

                    _ => todo!("archive creation"),
                }
            });
        *open = window_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}
