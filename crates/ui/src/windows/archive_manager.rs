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

use std::io::Write;

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
        save_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
    },
    Create {
        view: Option<luminol_components::FileSystemView<luminol_filesystem::host::FileSystem>>,
        load_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        version: u8,
    },
}

impl Default for Window {
    fn default() -> Self {
        Self {
            mode: Mode::Extract {
                view: None,
                load_promise: None,
                save_promise: None,
            },
            initialized: false,
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
                            self.mode = Mode::Extract {
                                view: None,
                                load_promise: None,
                                save_promise: None,
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
                            self.mode = Mode::Create {
                                view: None,
                                load_promise: None,
                                save_promise: None,
                                version: 1,
                            };
                        }
                    });

                    ui.separator();

                    self.show_inner(ui, update_state);

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
                                            v.ui(ui, update_state, None);
                                        } else {
                                            ui.add(
                                                egui::Label::new("No archive chosen").wrap(false),
                                            );
                                        }
                                    }
                                    Mode::Create { view, .. } => {
                                        if let Some(v) = view {
                                            v.ui(
                                                ui,
                                                update_state,
                                                Some(&CREATE_DEFAULT_SELECTED_DIRS),
                                            );
                                        } else {
                                            ui.add(
                                                egui::Label::new("No source folder chosen")
                                                    .wrap(false),
                                            );
                                        }
                                    }
                                });
                            });
                        },
                    );
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
        match &mut self.mode {
            Mode::Extract {
                view,
                load_promise,
                save_promise,
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

                    columns[1].with_layout(
                        egui::Layout {
                            cross_align: egui::Align::Center,
                            cross_justify: true,
                            ..Default::default()
                        },
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
                                *save_promise = Some(luminol_core::spawn_future(
                                    luminol_filesystem::host::FileSystem::from_folder_picker(),
                                ));
                            } else if save_promise.is_some() {
                                ui.spinner();
                            }
                        },
                    );
                });

                if let Some(p) = save_promise.take() {
                    match p.try_take() {
                        Ok(Ok(filesystem)) => {
                            if let Err(e) =
                                Self::copy_files(view.as_ref().unwrap(), &filesystem, false)
                            {
                                update_state.toasts.error(e.to_string());
                            } else {
                                update_state.toasts.info("Extracted successfully!");
                            }
                        }
                        Ok(Err(e)) => {
                            if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                update_state.toasts.error(e.to_string())
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
                            if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                update_state.toasts.error(e.to_string())
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

                ui.columns(2, |columns| {
                    columns[0].with_layout(
                        egui::Layout {
                            cross_align: egui::Align::Center,
                            cross_justify: true,
                            ..Default::default()
                        },
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

                    columns[1].with_layout(
                        egui::Layout {
                            cross_align: egui::Align::Center,
                            cross_justify: true,
                            ..Default::default()
                        },
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
                                            let view_filesystem = view.filesystem().clone();
                                            *save_promise =
                                                Some(luminol_core::spawn_future(async move {
                                                    let mut file = luminol_filesystem::host::File::new()?;

                                                    let _ = luminol_filesystem::archiver::FileSystem::from_buffer_and_files(
                                                        &mut file,
                                                        version,
                                                        file_paths.iter().map(|path| {
                                                            let file = view_filesystem.open_file(path, OpenFlags::Read)?;
                                                            let size = file.metadata()?.size as u32;
                                                            Ok((path, size, file))
                                                        }),
                                                    ).await?;

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
                                                }))
                                        }
                                        Err(e) => update_state.toasts.error(e.to_string()),
                                    }
                                }
                            } else if save_promise.is_some() {
                                ui.spinner();
                            }
                        },
                    );
                });

                if let Some(p) = save_promise.take() {
                    match p.try_take() {
                        Ok(Ok(())) => {
                            update_state.toasts.info("Created archive successfully!");
                        }
                        Ok(Err(e)) => {
                            if !matches!(e, luminol_filesystem::Error::CancelledLoading) {
                                update_state.toasts.error(e.to_string())
                            }
                        }
                        Err(p) => *save_promise = Some(p),
                    }
                }
            }
        }
    }

    fn copy_files(
        view: &luminol_components::FileSystemView<impl luminol_filesystem::FileSystem>,
        dest_fs: &impl luminol_filesystem::FileSystem,
        create_only: bool,
    ) -> luminol_filesystem::Result<()> {
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
                create_only,
            )?;
        }
        Ok(())
    }

    fn copy_files_recurse(
        src_fs: &impl luminol_filesystem::FileSystem,
        dest_fs: &impl luminol_filesystem::FileSystem,
        path: &camino::Utf8Path,
        is_file: bool,
        create_only: bool,
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
            if !create_only {
                std::io::copy(&mut src_file, &mut dest_file)
                    .map_err(luminol_filesystem::Error::IoError)?;
                dest_file.flush()?;
            }
        } else {
            dest_fs.create_dir(path)?;
            for entry in src_fs.read_dir(path)? {
                Self::copy_files_recurse(
                    src_fs,
                    dest_fs,
                    &entry.path,
                    entry.metadata.is_file,
                    create_only,
                )?;
            }
        }
        Ok(())
    }

    fn find_files(
        view: &luminol_components::FileSystemView<impl luminol_filesystem::FileSystem>,
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
        src_fs: &impl luminol_filesystem::FileSystem,
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