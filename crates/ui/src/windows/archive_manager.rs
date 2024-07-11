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

use luminol_components::UiExt;
use luminol_filesystem::{File, FileSystem, OpenFlags};

static CREATE_DEFAULT_SELECTED_DIRS: once_cell::sync::Lazy<
    qp_trie::Trie<qp_trie::wrapper::BString, ()>,
> = once_cell::sync::Lazy::new(|| {
    let mut trie = qp_trie::Trie::new();
    trie.insert_str("Data", ());
    trie.insert_str("Graphics", ());
    trie
});

/// The archive manager for creating and extracting RGSSAD archives.
pub struct Window {
    mode: Mode,
    initialized: bool,
    progress: std::sync::Arc<std::sync::atomic::AtomicUsize>,
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
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        progress_total: usize,
    },
    Create {
        view: Option<luminol_components::FileSystemView<luminol_filesystem::host::FileSystem>>,
        load_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        version: u8,
        progress_total: usize,
    },
}

impl Default for Window {
    fn default() -> Self {
        Self {
            mode: Mode::Extract {
                view: None,
                load_promise: None,
                save_promise: None,
                progress_total: 0,
            },
            initialized: false,
            progress: std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(usize::MAX)),
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
        // Open the currently loaded project by default
        if !self.initialized {
            self.initialized = true;
            if let Some(host) = update_state.filesystem.host() {
                match &mut self.mode {
                    Mode::Extract { view, .. } => {
                        if let Ok(Some((entry, archive))) = (|| {
                            host.read_dir("")?
                                .into_iter()
                                .find(|entry| {
                                    entry.metadata.is_file
                                        && matches!(
                                            entry.path.extension(),
                                            Some("rgssad" | "rgss2a" | "rgss3a")
                                        )
                                })
                                .map(|entry| {
                                    host.open_file(&entry.path, OpenFlags::Read)
                                        .and_then(luminol_filesystem::archiver::FileSystem::new)
                                        .map(|archive| (entry, archive))
                                })
                                .transpose()
                        })() {
                            *view = Some(luminol_components::FileSystemView::new(
                                "luminol_archive_manager_extract_view".into(),
                                archive,
                                entry.path.to_string(),
                            ))
                        }
                    }

                    Mode::Create { view, .. } => {
                        let name = host.root_path().to_string();
                        *view = Some(luminol_components::FileSystemView::new(
                            "luminol_archive_manager_create_view".into(),
                            host,
                            name,
                        ));
                    }
                }
            }
        }

        let mut window_open = true;
        egui::Window::new("RGSSAD Archive Manager")
            .open(&mut window_open)
            .show(ctx, |ui| {
                let enabled = match &self.mode {
                    Mode::Extract {
                        load_promise,
                        save_promise,
                        ..
                    } => load_promise.is_none() && save_promise.is_none(),
                    Mode::Create {
                        load_promise,
                        save_promise,
                        ..
                    } => load_promise.is_none() && save_promise.is_none(),
                };
                ui.add_enabled_ui(enabled, |ui| {
                    ui.columns(2, |columns| {
                        if columns[0]
                            .add(egui::SelectableLabel::new(
                                matches!(self.mode, Mode::Extract { .. }),
                                "Extract from archive",
                            ))
                            .clicked()
                        {
                            self.initialized = false;
                            self.progress = std::sync::Arc::new(
                                std::sync::atomic::AtomicUsize::new(usize::MAX),
                            );
                            self.mode = Mode::Extract {
                                view: None,
                                load_promise: None,
                                save_promise: None,
                                progress_total: 0,
                            };
                        }
                        if columns[1]
                            .add(egui::SelectableLabel::new(
                                matches!(self.mode, Mode::Create { .. }),
                                "Create new archive",
                            ))
                            .clicked()
                        {
                            self.initialized = false;
                            self.progress = std::sync::Arc::new(
                                std::sync::atomic::AtomicUsize::new(usize::MAX),
                            );
                            self.mode = Mode::Create {
                                view: None,
                                load_promise: None,
                                save_promise: None,
                                version: 1,
                                progress_total: 0,
                            };
                        }
                    });

                    ui.separator();

                    self.show_inner(ui, update_state);

                    ui.with_cross_justify(|ui| {
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.set_height(ui.available_height());
                            egui::ScrollArea::both().show(ui, |ui| match &mut self.mode {
                                Mode::Extract { view, .. } => {
                                    if let Some(v) = view {
                                        v.ui(ui, update_state, None);
                                    } else {
                                        ui.add(egui::Label::new("No archive chosen"));
                                    }
                                }
                                Mode::Create { view, .. } => {
                                    if let Some(v) = view {
                                        v.ui(ui, update_state, Some(&CREATE_DEFAULT_SELECTED_DIRS));
                                    } else {
                                        ui.add(egui::Label::new("No source folder chosen"));
                                    }
                                }
                            });
                        });
                    });
                });
            });

        *open = window_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl Window {
    fn show_inner(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
        let progress = self.progress.clone();

        match &mut self.mode {
            Mode::Extract {
                view,
                load_promise,
                save_promise,
                progress_total,
            } => {
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
                                Err(e) => luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error parsing archive contents")
                                ),
                            }
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Unable to read archive file")
                                );
                            }
                        }
                        Err(p) => *load_promise = Some(p),
                    }
                }

                let progress_amount = progress.load(std::sync::atomic::Ordering::Relaxed);

                if progress_amount == usize::MAX
                    || progress_amount == *progress_total
                    || save_promise.is_none()
                {
                    ui.columns(2, |columns| {
                        columns[0].with_cross_justify_center(
                            |ui| {
                                if load_promise.is_none() && ui.button("Choose archive").clicked() {
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

                        columns[1].with_cross_justify_center(
                            |ui| {
                                if save_promise.is_none()
                                    && ui
                                        .add_enabled(
                                            view.as_ref()
                                                .is_some_and(|view| view.iter().next().is_some()),
                                            egui::Button::new("Extract selected files"),
                                        )
                                        .clicked()
                                {
                                    let view = view.as_ref().unwrap();
                                    match Self::find_files(view) {
                                        Ok(file_paths) => {
                                            let ctx = ui.ctx().clone();
                                            let progress = progress.clone();
                                            let view_filesystem = view.filesystem().clone();
                                            *progress_total = file_paths.len();
                                            progress.store(usize::MAX, std::sync::atomic::Ordering::Relaxed);

                                            *save_promise = Some(luminol_core::spawn_future(async move {
                                                let dest_fs = luminol_filesystem::host::FileSystem::from_folder_picker().await?;
                                                progress.store(0, std::sync::atomic::Ordering::Relaxed);
                                                ctx.request_repaint();

                                                for path in file_paths {
                                                    if let Some(parent) = path.parent() {
                                                        dest_fs.create_dir(parent)?;
                                                    }
                                                    let mut src_file = view_filesystem.open_file(&path, OpenFlags::Read)?;
                                                    let mut dest_file = dest_fs.open_file(&path, OpenFlags::Write | OpenFlags::Create | OpenFlags::Truncate)?;
                                                    async_std::io::copy(&mut src_file, &mut dest_file).await?;

                                                    progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                    ctx.request_repaint();
                                                }

                                                Ok(())
                                            }));
                                        }
                                        Err(e) => luminol_core::error!(update_state.toasts, e.wrap_err("Error enumerating files to extract from archive")),
                                    }
                                } else if save_promise.is_some() {
                                    ui.spinner();
                                }
                            },
                        );
                    });
                } else {
                    ui.add(
                        egui::ProgressBar::new(if *progress_total == 0 {
                            0.
                        } else {
                            (progress_amount as f64 / *progress_total as f64) as f32
                        })
                        .show_percentage(),
                    );
                }

                if let Some(p) = save_promise.take() {
                    match p.try_take() {
                        Ok(Ok(())) => {
                            luminol_core::info!(update_state.toasts, "Extracted successfully!")
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error extracting archive")
                                );
                            }
                        }
                        Err(p) => *save_promise = Some(p),
                    }
                }
            }

            Mode::Create {
                view,
                load_promise,
                save_promise,
                version,
                progress_total,
            } => {
                if let Some(p) = load_promise.take() {
                    match p.try_take() {
                        Ok(Ok(handle)) => {
                            let name = handle.root_path().to_string();
                            *view = Some(luminol_components::FileSystemView::new(
                                "luminol_archive_manager_create_view".into(),
                                handle,
                                name,
                            ));
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Unable to read contents of source directory"),
                                );
                            }
                        }
                        Err(p) => *load_promise = Some(p),
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Version:");
                    ui.columns(4, |columns| {
                        columns[1].radio_value(version, 1, "XP");
                        columns[2].radio_value(version, 2, "VX");
                        columns[3].radio_value(version, 3, "VX Ace");
                    });
                });

                ui.separator();

                let progress_amount = progress.load(std::sync::atomic::Ordering::Relaxed);

                if progress_amount == usize::MAX
                    || progress_amount == *progress_total
                    || save_promise.is_none()
                {
                    ui.columns(2, |columns| {
                        columns[0].with_cross_justify_center(
                            |ui| {
                                if load_promise.is_none() && ui.button("Choose source folder").clicked()
                                {
                                    *load_promise = Some(luminol_core::spawn_future(
                                        luminol_filesystem::host::FileSystem::from_folder_picker(),
                                    ));
                                } else if load_promise.is_some() {
                                    ui.spinner();
                                }
                            },
                        );

                        columns[1].with_cross_justify_center(
                            |ui| {
                                if save_promise.is_none()
                                    && ui
                                        .add_enabled(
                                            view.as_ref()
                                                .is_some_and(|view| view.iter().next().is_some()),
                                            egui::Button::new("Create from selected files"),
                                        )
                                        .clicked()
                                {
                                    if let Some(view) = view {
                                        let version = *version;
                                        match Self::find_files(view) {
                                            Ok(file_paths) => {
                                                let ctx = ui.ctx().clone();
                                                let progress = progress.clone();
                                                let view_filesystem = view.filesystem().clone();
                                                *progress_total = file_paths.len();
                                                progress.store(usize::MAX, std::sync::atomic::Ordering::Relaxed);

                                                *save_promise =
                                                    Some(luminol_core::spawn_future(async move {
                                                        let mut file = luminol_filesystem::host::File::new()?;

                                                        let mut is_first = true;

                                                        progress.store(0, std::sync::atomic::Ordering::Relaxed);
                                                        ctx.request_repaint();

                                                        let _ = luminol_filesystem::archiver::FileSystem::from_buffer_and_files(
                                                            &mut file,
                                                            if version == 2 {
                                                                1
                                                            } else {
                                                                version
                                                            },
                                                            file_paths.iter().map(|path| {
                                                                if is_first {
                                                                    is_first = false;
                                                                } else {
                                                                    progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                                    ctx.request_repaint();
                                                                }

                                                                let file = view_filesystem.open_file(path, OpenFlags::Read)?;
                                                                let size = file.metadata()?.size as u32;
                                                                Ok((path, size, file))
                                                            }),
                                                        ).await?;

                                                        progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                        ctx.request_repaint();

                                                        file.save(
                                                            match version {
                                                                1 => "Game.rgssad",
                                                                2 => "Game.rgss2a",
                                                                3 => "Game.rgss3a",
                                                                _ => unreachable!(),
                                                            },
                                                            "RGSSAD archives",
                                                        )
                                                        .await
                                                    }));
                                            }
                                            Err(e) => luminol_core::error!(update_state.toasts, e.wrap_err("Error enumerating files to create archive from")),
                                        }
                                    }
                                } else if save_promise.is_some() {
                                    ui.spinner();
                                }
                            },
                        );
                    });
                } else {
                    ui.add(
                        egui::ProgressBar::new(if *progress_total == 0 {
                            0.
                        } else {
                            (progress_amount as f64 / *progress_total as f64) as f32
                        })
                        .show_percentage(),
                    );
                }

                if let Some(p) = save_promise.take() {
                    match p.try_take() {
                        Ok(Ok(())) => {
                            luminol_core::info!(
                                update_state.toasts,
                                "Created archive successfully!"
                            );
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error creating archive")
                                );
                            }
                        }
                        Err(p) => *save_promise = Some(p),
                    }
                }
            }
        }
    }

    fn find_files(
        view: &luminol_components::FileSystemView<impl luminol_filesystem::ReadDir>,
    ) -> luminol_filesystem::Result<Vec<camino::Utf8PathBuf>> {
        let mut vec = Vec::new();
        for metadata in view {
            Self::find_files_recurse(
                &mut vec,
                view.filesystem(),
                metadata.path.as_str().into(),
                metadata.is_file,
            )?;
        }
        Ok(vec)
    }

    fn find_files_recurse(
        vec: &mut Vec<camino::Utf8PathBuf>,
        src_fs: &impl luminol_filesystem::ReadDir,
        path: &camino::Utf8Path,
        is_file: bool,
    ) -> luminol_filesystem::Result<()> {
        if is_file {
            vec.push(path.to_owned());
        } else {
            for entry in src_fs.read_dir(path)? {
                Self::find_files_recurse(vec, src_fs, &entry.path, entry.metadata.is_file)?;
            }
        }
        Ok(())
    }
}
