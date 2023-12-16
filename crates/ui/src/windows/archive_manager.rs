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
pub struct Window {
    mode: Mode,
    save_promise: Option<
        poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
    >,
}

enum Mode {
    Extract {
        view: Option<
            luminol_components::FileSystemView<
                luminol_filesystem::archiver::FileSystem<luminol_filesystem::host::File>,
            >,
        >,
        load_promise: Option<
            poll_promise::Promise<
                luminol_filesystem::Result<(luminol_filesystem::host::File, String)>,
            >,
        >,
    },
    Create {
        view: Option<luminol_components::FileSystemView<luminol_filesystem::host::FileSystem>>,
        load_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
    },
}

impl Default for Window {
    fn default() -> Self {
        Self {
            mode: Mode::Extract {
                view: None,
                load_promise: None,
            },
            save_promise: None,
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
                            view: None,
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
                            view: None,
                            load_promise: None,
                        };
                    }
                });

                ui.separator();

                match &mut self.mode {
                    Mode::Extract { view, load_promise } => {
                        if let Some(p) = load_promise.take() {
                            match p.try_take() {
                                Ok(Ok((handle, name))) => {
                                    match luminol_filesystem::archiver::FileSystem::new(handle) {
                                        Ok(archiver) => {
                                            *view = Some(luminol_components::FileSystemView::new(
                                                "luminol_archive_manager_extract_view".into(),
                                                archiver,
                                                name,
                                            ))
                                        }
                                        Err(e) => update_state.toasts.error(e.to_string()),
                                    }
                                }
                                Ok(Err(e)) => {
                                    if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                        update_state.toasts.error(e.to_string())
                                    }
                                }
                                Err(p) => *load_promise = Some(p),
                            }
                        }

                        ui.columns(2, |columns| {
                            columns[0].with_layout(
                                egui::Layout {
                                    cross_align: egui::Align::Center,
                                    cross_justify: true,
                                    ..Default::default()
                                },
                                |ui| {
                                    if load_promise.is_none() && ui.button("Select archive").clicked() {
                                        *load_promise = Some(luminol_core::spawn_future(
                                            luminol_filesystem::host::File::from_file_picker(
                                                "RGSSAD archives",
                                                &["rgssad", "rgss2a", "rgss3a"],
                                            ),
                                        ));
                                    } else if load_promise.is_some() {
                                        ui.spinner();
                                    }
                                },
                            );

                            columns[1].with_layout(
                                egui::Layout {
                                    cross_align: egui::Align::Center,
                                    cross_justify: true,
                                    ..Default::default()
                                },
                                |ui| {
                                    if self.save_promise.is_none()
                                        && ui
                                            .add_enabled(
                                                view.is_some() && load_promise.is_none(),
                                                egui::Button::new("Extract"),
                                            )
                                            .clicked()
                                    {
                                        self.save_promise = Some(luminol_core::spawn_future(
                                            luminol_filesystem::host::FileSystem::from_folder_picker(),
                                        ));
                                    } else if self.save_promise.is_some() {
                                        ui.spinner();
                                    }
                                },
                            );
                        });
                    }

                    _ => todo!("archive creation"),
                }

                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        ..Default::default()
                    },
                    |ui| {
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());
                            egui::ScrollArea::both().show(ui, |ui| match &mut self.mode {
                                Mode::Extract { view, .. } => {
                                    if let Some(v) = view {
                                        if let Err(e) = v.ui(ui) {
                                            update_state.toasts.error(e.to_string());
                                            *view = None
                                        }
                                    } else {
                                        ui.add(egui::Label::new("No archive selected").wrap(false));
                                    }
                                }
                                Mode::Create { .. } => todo!("archive creation"),
                            });
                        });
                    },
                );
            });

        if let Some(p) = self.save_promise.take() {
            match p.try_take() {
                Ok(Ok(filesystem)) => {
                    if let Err(e) = self.copy_files(&filesystem) {
                        update_state.toasts.error(e.to_string());
                    } else {
                        update_state.toasts.info(match &self.mode {
                            Mode::Extract { .. } => "Extracted successfully!",
                            Mode::Create { .. } => "Created archive successfully!",
                        });
                    }
                }
                Ok(Err(e)) => {
                    if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                        update_state.toasts.error(e.to_string())
                    }
                }
                Err(p) => self.save_promise = Some(p),
            }
        }

        *open = window_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl Window {
    fn copy_files(
        &self,
        dest_fs: &impl luminol_filesystem::FileSystem,
    ) -> luminol_filesystem::Result<()> {
        match &self.mode {
            Mode::Extract {
                view: Some(view), ..
            } => {
                for metadata in view {
                    let path: &camino::Utf8Path = metadata.path.as_str().into();
                    if metadata.is_file {
                        if let Some(parent) = path.parent() {
                            let _ = dest_fs.create_dir(parent);
                        }
                    }
                    Self::copy_files_recurse(
                        view.filesystem(),
                        dest_fs,
                        metadata.path.as_str().into(),
                        metadata.is_file,
                    )?;
                }
                Ok(())
            }

            _ => todo!("archive creation"),
        }
    }

    fn copy_files_recurse(
        src_fs: &impl luminol_filesystem::FileSystem,
        dest_fs: &impl luminol_filesystem::FileSystem,
        path: &camino::Utf8Path,
        is_file: bool,
    ) -> luminol_filesystem::Result<()> {
        if is_file {
            let mut src_file = src_fs.open_file(path, luminol_filesystem::OpenFlags::Read)?;
            let mut dest_file = dest_fs.open_file(
                path,
                luminol_filesystem::OpenFlags::Read
                    | luminol_filesystem::OpenFlags::Write
                    | luminol_filesystem::OpenFlags::Create
                    | luminol_filesystem::OpenFlags::Truncate,
            )?;
            std::io::copy(&mut src_file, &mut dest_file)
                .map_err(|e| luminol_filesystem::Error::IoError(e))?;
        } else {
            dest_fs.create_dir(path)?;
            for entry in src_fs.read_dir(path)? {
                Self::copy_files_recurse(src_fs, dest_fs, &entry.path, entry.metadata.is_file)?;
            }
        }
        Ok(())
    }
}
