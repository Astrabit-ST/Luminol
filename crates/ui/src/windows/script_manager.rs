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

use futures_lite::{AsyncReadExt, AsyncWriteExt, StreamExt};
use luminol_components::UiExt;
use luminol_filesystem::{File, FileSystem, OpenFlags};

/// The script manager for creating and extracting Scripts.rxdata.
pub struct Window {
    mode: Mode,
    initialized: bool,
    progress: std::sync::Arc<std::sync::atomic::AtomicUsize>,
}

type Scripts = Vec<luminol_data::rpg::Script>;

enum Mode {
    Extract {
        view: Option<luminol_components::FileSystemView<ScriptsFileSystem>>,
        load_promise:
            Option<poll_promise::Promise<luminol_filesystem::Result<(ScriptsFileSystem, String)>>>,
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        progress_total: usize,
    },
    Create {
        view: Option<luminol_components::FileSystemView<luminol_filesystem::host::FileSystem>>,
        load_promise: Option<
            poll_promise::Promise<luminol_filesystem::Result<luminol_filesystem::host::FileSystem>>,
        >,
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        format: ScriptsFormat,
        progress_total: usize,
    },
    Convert {
        scripts: Option<(std::sync::Arc<parking_lot::Mutex<Scripts>>, String)>,
        load_promise: Option<poll_promise::Promise<luminol_filesystem::Result<(Scripts, String)>>>,
        save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
        format: ScriptsFormat,
    },
}

#[derive(Clone, Copy, strum::Display, strum::EnumIter)]
enum ScriptsFormat {
    #[strum(to_string = "RPG Maker XP")]
    Rxdata,
    #[strum(to_string = "RPG Maker VX")]
    Rvdata,
    #[strum(to_string = "RPG Maker VX Ace")]
    Rvdata2,
    #[strum(to_string = "JSON")]
    Json,
    #[strum(to_string = "YAML")]
    Yaml,
    #[strum(to_string = "RON")]
    Ron,
}

struct ScriptsFileSystem(std::sync::Arc<parking_lot::Mutex<ScriptsFileSystemInner>>);

struct ScriptsFileSystemInner {
    trie: luminol_filesystem::FileSystemTrie<luminol_data::rpg::Script>,
    names: Vec<String>,
}

impl ScriptsFileSystem {
    fn new(scripts: impl Iterator<Item = luminol_data::rpg::Script>) -> Self {
        let mut trie = luminol_filesystem::FileSystemTrie::new();
        let (_, hint) = scripts.size_hint();
        let mut names = Vec::with_capacity(hint.unwrap_or_default());
        for script in scripts {
            let mut name = script.name.replace('\\', "/");
            loop {
                let new_name = name.replace("//", "/");
                if new_name == name {
                    break;
                } else {
                    name = new_name;
                }
            }
            if let Some(stripped) = name.strip_prefix('/') {
                name = stripped.to_string();
            }
            if let Some(stripped) = name.strip_suffix('/') {
                name = stripped.to_string();
            }
            if !name.is_empty() && !script.script_text.is_empty() {
                trie.create_file(&name, script);
                names.push(name);
            }
        }
        Self(std::sync::Arc::new(parking_lot::Mutex::new(
            ScriptsFileSystemInner { trie, names },
        )))
    }
}

impl luminol_filesystem::ReadDir for ScriptsFileSystem {
    fn read_dir(
        &self,
        path: impl AsRef<camino::Utf8Path>,
    ) -> luminol_filesystem::Result<Vec<luminol_filesystem::DirEntry>> {
        let path = path.as_ref();
        Ok(self
            .0
            .lock()
            .trie
            .iter_dir(path)
            .map_or_else(Default::default, |iter| {
                iter.map(|(name, maybe_script)| luminol_filesystem::DirEntry {
                    path: if path.as_str().is_empty() {
                        name.into()
                    } else {
                        format!("{path}/{name}").into()
                    },
                    metadata: if let Some(script) = maybe_script {
                        luminol_filesystem::Metadata {
                            is_file: true,
                            size: script.script_text.as_bytes().len() as u64,
                        }
                    } else {
                        luminol_filesystem::Metadata {
                            is_file: false,
                            size: 0,
                        }
                    },
                })
                .collect()
            }))
    }
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

fn get_scripts_from_filesystem<T>(
    scripts_path: Option<&str>,
    filesystem: &T,
) -> Option<(String, Scripts)>
where
    T: luminol_filesystem::FileSystem,
{
    type InnerOk = Option<(String, Scripts)>;
    type InnerErr<T> = (String, luminol_filesystem::archiver::FileSystem<T>);
    fn inner<T>(scripts_path: Option<&str>, filesystem: &T) -> Result<InnerOk, InnerErr<T::File>>
    where
        T: luminol_filesystem::FileSystem,
    {
        let mut archive = None;
        let output = scripts_path
            .map(|path| format!("Data/{path}"))
            .as_deref()
            .into_iter()
            .chain([
                "Data/xScripts.rxdata",
                "Data/xScripts.rvdata",
                "Data/xScripts.rvdata2",
                "Data/xScripts.json",
                "Data/xScripts.yaml",
                "Data/xScripts.yml",
                "Data/xScripts.ron",
                "Data/Scripts.rxdata",
                "Data/Scripts.rvdata",
                "Data/Scripts.rvdata2",
                "Data/Scripts.json",
                "Data/Scripts.yaml",
                "Data/Scripts.yml",
                "Data/Scripts.ron",
                "Game.rgssad",
                "Game.rgss2a",
                "Game.rgss3a",
            ])
            .find_map(|path| {
                let path = camino::Utf8PathBuf::from(path);
                let mut file = filesystem.open_file(&path, OpenFlags::Read).ok()?;
                let vec: Vec<_> = match path.extension() {
                    Some("json") => serde_json::from_reader(std::io::BufReader::new(file)).ok()?,

                    Some("yaml" | "yml") => {
                        serde_yml::from_reader(std::io::BufReader::new(file)).ok()?
                    }

                    Some("ron") => ron::de::from_reader(std::io::BufReader::new(file)).ok()?,

                    Some("rgssad" | "rgss2a" | "rgss3a") => {
                        archive = Some((
                            path.into(),
                            luminol_filesystem::archiver::FileSystem::new(file).ok()?,
                        ));
                        return Default::default();
                    }

                    _ => {
                        use std::io::Read;
                        let mut data = Vec::with_capacity(file.metadata().ok()?.size as usize);
                        file.read_to_end(&mut data).ok()?;
                        let mut de = luminol_core::alox_48::Deserializer::new(&data).ok()?;
                        de.deserialize_value().ok()?
                    }
                };
                Some((path.to_string().replace('\\', "/"), vec))
            });
        if let Some(archive) = archive {
            Err(archive)
        } else {
            Ok(output)
        }
    }

    match inner(scripts_path, filesystem) {
        Ok(output) => output,
        Err((prefix, archive)) => match inner(scripts_path, &archive) {
            Ok(output) => output.map(|(suffix, scripts)| (format!("{prefix}/{suffix}"), scripts)),
            Err(_) => None,
        },
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("Script Manager")
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
                let scripts_path = update_state
                    .project_config
                    .as_ref()
                    .map(|config| config.project.scripts_path.as_str());

                match &mut self.mode {
                    Mode::Extract { view, .. } => {
                        if let Some((path, vec)) = get_scripts_from_filesystem(scripts_path, &host)
                        {
                            *view = Some(luminol_components::FileSystemView::new(
                                "luminol_script_manager_extract_view".into(),
                                ScriptsFileSystem::new(vec.into_iter()),
                                path,
                            ))
                        }
                    }

                    Mode::Create { view, .. } => {
                        let name = host.root_path().to_string();
                        *view = Some(luminol_components::FileSystemView::new(
                            "luminol_script_manager_create_view".into(),
                            host,
                            name,
                        ));
                    }

                    Mode::Convert { scripts, .. } => {
                        if let Some((path, vec)) = get_scripts_from_filesystem(scripts_path, &host)
                        {
                            *scripts =
                                Some((std::sync::Arc::new(parking_lot::Mutex::new(vec)), path));
                        }
                    }
                }
            }
        }

        let mut window_open = true;
        egui::Window::new("Script Manager")
            .open(&mut window_open)
            .default_width(500.)
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
                    Mode::Convert {
                        load_promise,
                        save_promise,
                        ..
                    } => load_promise.is_none() && save_promise.is_none(),
                };
                ui.add_enabled_ui(enabled, |ui| {
                    ui.columns(3, |columns| {
                        if columns[0]
                            .add(egui::SelectableLabel::new(
                                matches!(self.mode, Mode::Extract { .. }),
                                "Extract from Scripts file",
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
                                "Create new Scripts file",
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
                                format: ScriptsFormat::Rxdata,
                                progress_total: 0,
                            };
                        }
                        if columns[2]
                            .add(egui::SelectableLabel::new(
                                matches!(self.mode, Mode::Convert { .. }),
                                "Convert Scripts file",
                            ))
                            .clicked()
                        {
                            self.initialized = false;
                            self.progress = std::sync::Arc::new(
                                std::sync::atomic::AtomicUsize::new(usize::MAX),
                            );
                            self.mode = Mode::Convert {
                                scripts: None,
                                load_promise: None,
                                save_promise: None,
                                format: ScriptsFormat::Rxdata,
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
                                        ui.add(egui::Label::new("No Scripts file chosen"));
                                    }
                                }
                                Mode::Create { view, .. } => {
                                    if let Some(v) = view {
                                        v.ui(ui, update_state, None);
                                    } else {
                                        ui.add(egui::Label::new("No source folder chosen"));
                                    }
                                }
                                Mode::Convert { scripts, .. } => {
                                    ui.add(if let Some((_, path)) = scripts {
                                        egui::Label::new(format!("Scripts file: {path:?}"))
                                    } else {
                                        egui::Label::new("No Scripts file chosen")
                                    });
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
                        Ok(Ok((fs, name))) => {
                            *view = Some(luminol_components::FileSystemView::new(
                                "luminol_script_manager_extract_view".into(),
                                fs,
                                name,
                            ))
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Unable to read Scripts file")
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
                                if load_promise.is_none() && ui.button("Choose Scripts file").clicked() {
                                    let scripts_path = update_state
                                        .project_config
                                        .as_ref()
                                        .map(|config| config.project.scripts_path.clone());

                                    *load_promise = Some(luminol_core::spawn_future(async move {
                                        let (mut file, filename) = luminol_filesystem::host::File::from_file_picker(
                                            "RPG Maker data",
                                            &["rxdata", "rvdata", "rvdata2", "json", "yaml", "yml", "ron", "rgssad", "rgss2a", "rgss3a"],
                                        ).await?;
                                        let (vec, path): (Vec<_>, _) = match filename.to_lowercase().rsplit_once('.').map(|(_, ext)| ext) {
                                            Some("json") => {
                                                (serde_json::from_reader(std::io::BufReader::new(file))?, filename)
                                            }

                                            Some("yaml" | "yml") => {
                                                (serde_yml::from_reader(std::io::BufReader::new(file))?, filename)
                                            }

                                            Some("ron") => {
                                                (ron::de::from_reader(std::io::BufReader::new(file))?, filename)
                                            }

                                            Some("rgssad" | "rgss2a" | "rgss3a") => {
                                                let archive = luminol_filesystem::archiver::FileSystem::new(file)?;
                                                let (path, scripts) = get_scripts_from_filesystem(scripts_path.as_deref(), &archive)
                                                    .ok_or(color_eyre::eyre::eyre!("No Scripts file found in the archive"))?;
                                                (scripts, format!("{filename}/{path}"))
                                            }

                                            _ => {
                                                let mut buf = Vec::with_capacity(file.metadata()?.size as usize);
                                                file.read_to_end(&mut buf).await?;
                                                let mut de = luminol_core::alox_48::Deserializer::new(&buf)?;
                                                (luminol_core::alox_48::path_to_error::deserialize(&mut de)
                                                    .map_err(|(error, trace)| luminol_core::format_traced_error(error, trace))?, filename)
                                            }
                                        };
                                        Ok((ScriptsFileSystem::new(vec.into_iter()), path))
                                    }));
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
                                            view.is_some(),
                                            egui::Button::new("Extract selected files"),
                                        )
                                        .clicked()
                                {
                                    let view = view.as_ref().unwrap();
                                    match Self::find_files(view) {
                                        Ok(file_paths) => {
                                            let ctx = ui.ctx().clone();
                                            let progress = progress.clone();
                                            let scripts = view.filesystem().0.clone();
                                            *progress_total = file_paths.len();
                                            progress.store(usize::MAX, std::sync::atomic::Ordering::Relaxed);

                                            *save_promise = Some(luminol_core::spawn_future(async move {
                                                let dest_fs = luminol_filesystem::host::FileSystem::from_folder_picker().await?;

                                                progress.store(0, std::sync::atomic::Ordering::Relaxed);
                                                ctx.request_repaint();

                                                let mut names_file = dest_fs.open_file("_scripts.txt", OpenFlags::Write | OpenFlags::Create | OpenFlags::Truncate)?;
                                                let names_len = scripts.lock().names.len();
                                                for i in 0..names_len {
                                                    let name = scripts.lock().names[i].clone();
                                                    names_file.write_all(name.as_bytes()).await?;
                                                    names_file.write_all(b"\n").await?;
                                                }
                                                names_file.flush().await?;
                                                drop(names_file);

                                                for mut path in file_paths {
                                                    if let Some(parent) = path.parent() {
                                                        dest_fs.create_dir(parent)?;
                                                    }
                                                    let src = scripts.lock().trie.get_file(path.as_str()).ok_or(luminol_filesystem::Error::NotExist)?.script_text.clone();
                                                    let src_data = src.as_bytes();
                                                    if let Some(filename) = path.file_name() {
                                                        path.set_file_name(format!("{filename}.rb"));
                                                    }
                                                    let mut dest_file = dest_fs.open_file(&path, OpenFlags::Write | OpenFlags::Create | OpenFlags::Truncate)?;
                                                    async_std::io::copy(&mut async_std::io::Cursor::new(src_data), &mut dest_file).await?;

                                                    progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                    ctx.request_repaint();
                                                }

                                                Ok(())
                                            }));
                                        }
                                        Err(e) => luminol_core::error!(update_state.toasts, e.wrap_err("Error enumerating files to extract from Scripts file")),
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
                                    e.wrap_err("Error extracting from Scripts file")
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
                format,
                progress_total,
            } => {
                if let Some(p) = load_promise.take() {
                    match p.try_take() {
                        Ok(Ok(handle)) => {
                            let name = handle.root_path().to_string();
                            *view = Some(luminol_components::FileSystemView::new(
                                "luminol_script_manager_create_view".into(),
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
                    ui.label("Output Format:");
                    ui.add(luminol_components::EnumComboBox::new(
                        "luminol_script_manager_create_format",
                        format,
                    ));
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
                                        let format = *format;
                                        match Self::find_files(view) {
                                            Ok(file_paths) => {
                                                let ctx = ui.ctx().clone();
                                                let progress = progress.clone();
                                                let view_filesystem = view.filesystem().clone();
                                                *progress_total = file_paths.len();
                                                progress.store(usize::MAX, std::sync::atomic::Ordering::Relaxed);

                                                *save_promise =
                                                    Some(luminol_core::spawn_future(async move {
                                                        use async_std::io::prelude::BufReadExt;

                                                        let mut is_first = true;

                                                        progress.store(0, std::sync::atomic::Ordering::Relaxed);
                                                        ctx.request_repaint();

                                                        let mut lines = view_filesystem.exists("_scripts.txt")?
                                                            .then(|| view_filesystem.open_file("_scripts.txt", OpenFlags::Read))
                                                            .transpose()?
                                                            .map(|names_file| async_std::io::BufReader::new(names_file).lines());

                                                        let mut scripts = Vec::with_capacity(file_paths.len());

                                                        let mut file_path_trie = qp_trie::Trie::new();
                                                        for path in file_paths.iter() {
                                                            let name = path.as_str().replace('\\', "/");
                                                            let name = if [".rb", ".ru"].iter().any(|suffix| name.to_lowercase().ends_with(suffix)) {
                                                                name.rsplit_once('.').unwrap().0.to_string()
                                                            } else {
                                                                continue;
                                                            };
                                                            file_path_trie.insert_str(&name, ());
                                                        }
                                                        let mut file_path_iter = file_paths.iter();

                                                        while let Some(path) =
                                                            if let Some(mut l) = lines.take() {
                                                                let line = loop {
                                                                    let line = l.next().await;
                                                                    if !line.as_ref().is_some_and(|line| line.as_ref().is_ok_and(|line| line.is_empty())) {
                                                                        break line;
                                                                    }
                                                                };
                                                                if line.is_some() {
                                                                    lines = Some(l);
                                                                    line.transpose().map(|o| o.map(|line| format!("{line}.rb")))
                                                                } else {
                                                                    Ok(file_path_iter.next().map(ToString::to_string))
                                                                }
                                                            } else {
                                                                Ok(file_path_iter.next().map(ToString::to_string))
                                                            }?
                                                        {
                                                            let name = path.to_string().replace('\\', "/");
                                                            let name = if [".rb", ".ru"].iter().any(|suffix| name.to_lowercase().ends_with(suffix)) {
                                                                name.rsplit_once('.').unwrap().0.to_string()
                                                            } else {
                                                                continue;
                                                            };

                                                            if !file_path_trie.contains_key_str(&name) {
                                                                continue;
                                                            }
                                                            file_path_trie.remove_str(&name);

                                                            if is_first {
                                                                is_first = false;
                                                            } else {
                                                                progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                                ctx.request_repaint();
                                                            }

                                                            let mut script_text = String::new();
                                                            view_filesystem.open_file(&path, OpenFlags::Read)?.read_to_string(&mut script_text).await?;

                                                            let script = luminol_data::rpg::Script::new(
                                                                name,
                                                                script_text,
                                                            );
                                                            scripts.push(script);
                                                        }

                                                        progress.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                                                        ctx.request_repaint();

                                                        let mut file = luminol_filesystem::host::File::new()?;

                                                        match format {
                                                            ScriptsFormat::Json => {
                                                                serde_json::to_writer_pretty(std::io::BufWriter::new(&mut file), &scripts)?;
                                                            }

                                                            ScriptsFormat::Yaml => {
                                                                serde_yml::to_writer(std::io::BufWriter::new(&mut file), &scripts)?;
                                                            }

                                                            ScriptsFormat::Ron => {
                                                                ron::ser::to_writer_pretty(std::io::BufWriter::new(&mut file), &scripts, ron::ser::PrettyConfig::new().indentor("  ".into()))?;
                                                            }

                                                            ScriptsFormat::Rxdata | ScriptsFormat::Rvdata | ScriptsFormat::Rvdata2 => {
                                                                let mut serializer = luminol_core::alox_48::Serializer::new();
                                                                luminol_core::alox_48::path_to_error::serialize(scripts, &mut serializer)
                                                                    .map_err(|(error, trace)| luminol_core::format_traced_error(error, trace))?;
                                                                file.write_all(&serializer.output).await?;
                                                            }
                                                        }

                                                        file.flush().await?;
                                                        file.save(
                                                            match format {
                                                                ScriptsFormat::Rxdata => "Scripts.rxdata",
                                                                ScriptsFormat::Rvdata => "Scripts.rvdata",
                                                                ScriptsFormat::Rvdata2 => "Scripts.rvdata2",
                                                                ScriptsFormat::Json => "Scripts.json",
                                                                ScriptsFormat::Yaml => "Scripts.yaml",
                                                                ScriptsFormat::Ron => "Scripts.ron",
                                                            },
                                                            "RPG Maker data",
                                                        )
                                                        .await
                                                    }));
                                            }
                                            Err(e) => luminol_core::error!(update_state.toasts, e.wrap_err("Error enumerating files to create Scripts file from")),
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
                                "Created Scripts file successfully!"
                            );
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error creating Scripts file")
                                );
                            }
                        }
                        Err(p) => *save_promise = Some(p),
                    }
                }
            }

            Mode::Convert {
                scripts,
                load_promise,
                save_promise,
                format,
            } => {
                if let Some(p) = load_promise.take() {
                    match p.try_take() {
                        Ok(Ok((vec, name))) => {
                            *scripts =
                                Some((std::sync::Arc::new(parking_lot::Mutex::new(vec)), name));
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Unable to read Scripts file")
                                );
                            }
                        }
                        Err(p) => *load_promise = Some(p),
                    }
                }

                ui.horizontal(|ui| {
                    ui.label("Output Format:");
                    ui.add(luminol_components::EnumComboBox::new(
                        "luminol_script_manager_convert_format",
                        format,
                    ));
                });

                ui.separator();

                ui.columns(2, |columns| {
                    columns[0].with_cross_justify_center(|ui| {
                        if load_promise.is_none() && ui.button("Choose Scripts file").clicked() {
                            let scripts_path = update_state
                                .project_config
                                .as_ref()
                                .map(|config| config.project.scripts_path.clone());

                            *load_promise = Some(luminol_core::spawn_future(async move {
                                let (mut file, filename) =
                                    luminol_filesystem::host::File::from_file_picker(
                                        "RPG Maker data",
                                        &[
                                            "rxdata", "rvdata", "rvdata2", "json", "yaml", "yml",
                                            "ron", "rgssad", "rgss2a", "rgss3a",
                                        ],
                                    )
                                    .await?;
                                let vec: Vec<_> = match filename
                                    .to_lowercase()
                                    .rsplit_once('.')
                                    .map(|(_, ext)| ext)
                                {
                                    Some("json") => {
                                        serde_json::from_reader(std::io::BufReader::new(file))?
                                    }

                                    Some("yaml" | "yml") => {
                                        serde_yml::from_reader(std::io::BufReader::new(file))?
                                    }

                                    Some("ron") => {
                                        ron::de::from_reader(std::io::BufReader::new(file))?
                                    }

                                    Some("rgssad" | "rgss2a" | "rgss3a") => {
                                        let archive =
                                            luminol_filesystem::archiver::FileSystem::new(file)?;
                                        let (_, scripts) = get_scripts_from_filesystem(
                                            scripts_path.as_deref(),
                                            &archive,
                                        )
                                        .ok_or(color_eyre::eyre::eyre!(
                                            "No Scripts file found in the archive"
                                        ))?;
                                        scripts
                                    }

                                    _ => {
                                        let mut buf =
                                            Vec::with_capacity(file.metadata()?.size as usize);
                                        file.read_to_end(&mut buf).await?;
                                        let mut de =
                                            luminol_core::alox_48::Deserializer::new(&buf)?;
                                        luminol_core::alox_48::path_to_error::deserialize(&mut de)
                                            .map_err(|(error, trace)| {
                                            luminol_core::format_traced_error(error, trace)
                                        })?
                                    }
                                };
                                Ok((vec, filename))
                            }));
                        } else if load_promise.is_some() {
                            ui.spinner();
                        }
                    });

                    columns[1].with_cross_justify_center(|ui| {
                        if save_promise.is_none()
                            && ui
                                .add_enabled(scripts.is_some(), egui::Button::new("Convert"))
                                .clicked()
                        {
                            if let Some((scripts, _)) = scripts {
                                let format = *format;
                                let scripts = scripts.clone();

                                *save_promise = Some(luminol_core::spawn_future(async move {
                                    let mut file = luminol_filesystem::host::File::new()?;

                                    match format {
                                        ScriptsFormat::Json => {
                                            serde_json::to_writer_pretty(
                                                std::io::BufWriter::new(&mut file),
                                                &*scripts.lock(),
                                            )?;
                                        }

                                        ScriptsFormat::Yaml => {
                                            serde_yml::to_writer(
                                                std::io::BufWriter::new(&mut file),
                                                &*scripts.lock(),
                                            )?;
                                        }

                                        ScriptsFormat::Ron => {
                                            ron::ser::to_writer_pretty(
                                                std::io::BufWriter::new(&mut file),
                                                &*scripts.lock(),
                                                ron::ser::PrettyConfig::new().indentor("  ".into()),
                                            )?;
                                        }

                                        ScriptsFormat::Rxdata
                                        | ScriptsFormat::Rvdata
                                        | ScriptsFormat::Rvdata2 => {
                                            let mut serializer =
                                                luminol_core::alox_48::Serializer::new();
                                            luminol_core::alox_48::path_to_error::serialize(
                                                &*scripts.lock(),
                                                &mut serializer,
                                            )
                                            .map_err(
                                                |(error, trace)| {
                                                    luminol_core::format_traced_error(error, trace)
                                                },
                                            )?;
                                            file.write_all(&serializer.output).await?;
                                        }
                                    }

                                    file.flush().await?;
                                    file.save(
                                        match format {
                                            ScriptsFormat::Rxdata => "Scripts.rxdata",
                                            ScriptsFormat::Rvdata => "Scripts.rvdata",
                                            ScriptsFormat::Rvdata2 => "Scripts.rvdata2",
                                            ScriptsFormat::Json => "Scripts.json",
                                            ScriptsFormat::Yaml => "Scripts.yaml",
                                            ScriptsFormat::Ron => "Scripts.ron",
                                        },
                                        "RPG Maker data",
                                    )
                                    .await
                                }));
                            }
                        } else if save_promise.is_some() {
                            ui.spinner();
                        }
                    });
                });

                if let Some(p) = save_promise.take() {
                    match p.try_take() {
                        Ok(Ok(())) => {
                            luminol_core::info!(
                                update_state.toasts,
                                "Created Scripts file successfully!"
                            );
                        }
                        Ok(Err(e)) => {
                            if !matches!(
                                e.root_cause().downcast_ref(),
                                Some(luminol_filesystem::Error::CancelledLoading)
                            ) {
                                luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error creating Scripts file")
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
