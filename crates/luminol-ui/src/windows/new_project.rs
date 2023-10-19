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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use strum::IntoEnumIterator;

use std::io::Read;

/// The new project window
pub struct Window {
    name: String,
    rgss_ver: luminol_config::RGSSVer,
    editor_ver: luminol_config::RMVer,
    project_promise: Option<poll_promise::Promise<PromiseResult>>,
    download_executable: bool,
    progress: Arc<Progress>,
    init_git: bool,
    git_branch_name: String,
}

struct CreateProjectResult {
    data_cache: luminol_core::Data,
    config: luminol_config::project::Config,
    host_fs: luminol_filesystem::host::FileSystem,
}

type PromiseResult = Result<CreateProjectResult, String>;

#[derive(Default)]
struct Progress {
    total_progress: AtomicUsize,
    current_progress: AtomicUsize,
    zip_total: AtomicUsize,
    zip_current: AtomicUsize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            name: "My Project".to_string(),
            rgss_ver: luminol_config::RGSSVer::RGSS1,
            editor_ver: luminol_config::RMVer::XP,
            project_promise: None,
            download_executable: false,
            progress: Arc::default(),
            init_git: false,
            git_branch_name: "master".to_string(),
        }
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        "New Project".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("New Project")
    }

    fn show<W, T>(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_, W, T>,
    ) {
        let mut win_open = true;
        egui::Window::new(self.name())
            .open(&mut win_open)
            .show(ctx, |ui| {
                ui.add_enabled_ui(self.project_promise.is_none(), |ui| {
                    ui.label("Project Name");
                    ui.text_edit_singleline(&mut self.name);

                    ui.checkbox(&mut self.init_git, "Initialize with git repository");
                    ui.add_enabled_ui(self.init_git, |ui| {
                        ui.label("Git Branch");
                        ui.text_edit_singleline(&mut self.git_branch_name);
                    });

                    egui::ComboBox::from_label("RGSS runtime")
                        .selected_text(self.rgss_ver.to_string())
                        .show_ui(ui, |ui| {
                            for ver in luminol_config::RGSSVer::iter() {
                                ui.selectable_value(&mut self.rgss_ver, ver, ver.to_string());
                            }
                        });

                    if matches!(
                        self.rgss_ver,
                        luminol_config::RGSSVer::ModShot
                            | luminol_config::RGSSVer::MKXPFreebird
                            | luminol_config::RGSSVer::MKXPZ
                    ) {
                        ui.checkbox(
                            &mut self.download_executable,
                            format!("Download latest version of {}", self.rgss_ver),
                        );
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(self.project_promise.is_some(), |ui| {
                        if ui.button("Ok").clicked() {
                            let rgss_ver = self.rgss_ver;
                            let config = luminol_config::project::Config::from_project(
                                luminol_config::project::Project {
                                    project_name: self.name.clone(),
                                    rgss_ver,
                                    editor_ver: self.editor_ver,
                                    ..Default::default()
                                },
                            );
                            let download_executable = self.download_executable
                                && matches!(
                                    rgss_ver,
                                    luminol_config::RGSSVer::ModShot
                                        | luminol_config::RGSSVer::MKXPFreebird
                                        | luminol_config::RGSSVer::MKXPZ
                                );
                            let progress = self.progress.clone();

                            let init_git = self.init_git;

                            let branch_name = self.git_branch_name.clone();

                            self.project_promise =
                                Some(poll_promise::Promise::spawn_local(Self::setup_project(
                                    config,
                                    download_executable,
                                    init_git.then_some(branch_name),
                                    progress,
                                )));
                        }
                        if ui.button("Cancel").clicked() {
                            *open = false;
                        }
                    });
                })
            });

        if let Some(p) = self.project_promise.take() {
            match p.try_take() {
                Ok(Ok(CreateProjectResult {
                    data_cache,
                    config,
                    host_fs,
                })) => {
                    update_state.filesystem.load_partially_loaded_project(
                        host_fs,
                        &config,
                        &mut update_state.global_config,
                    );
                    *update_state.data = data_cache;
                    update_state.project_config.replace(config);
                }
                Ok(Err(error)) => update_state.toasts.error(error.to_string()),
                Err(p) => self.project_promise = Some(p),
            }
        }

        *open &= win_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl Window {
    async fn setup_project(
        config: luminol_config::project::Config,
        download_executable: bool,
        git_branch_name: Option<String>,
        progress: Arc<Progress>,
    ) -> PromiseResult {
        let host_fs = luminol_filesystem::host::FileSystem::from_pile_picker()
            .await
            .map_err(|e| e.to_string())?;

        // TODO
        let data_cache = luminol_core::Data::defaults_from_config(&config);
        data_cache.save()?;

        if download_executable {
            Self::download_executable(&config, &host_fs, progress).await?;
        }

        if let Some(branch_name) = git_branch_name {
            if let Err(e) = std::process::Command::new("git")
                .arg("init")
                .arg("-b")
                .arg(branch_name)
                .current_dir(host_fs.root_path())
                .spawn()
                .and_then(|mut c| c.wait())
            {
                return Err(format!("Failed to initialize git repository: {e}"));
            }
        }

        Ok(CreateProjectResult {
            data_cache,
            config,
            host_fs,
        })
    }

    async fn download_executable(
        config: &luminol_config::project::Config,
        filesystem: &impl luminol_filesystem::FileSystem,
        progress: Arc<Progress>,
    ) -> Result<(), String> {
        let zip_url: &[_] = match config.project.rgss_ver {
            luminol_config:: RGSSVer::ModShot => &[
                "https://github.com/thehatkid/ModShot/releases/download/latest/ModShot_Windows_bb6bcbc_Ruby-3.1-ucrt64_Steam-false.zip", 
                "https://github.com/thehatkid/ModShot/releases/download/latest/ModShot_Linux_bb6bcbc_Ruby-3.1_Steam-false.zip"
            ],
            luminol_config::RGSSVer::MKXPZ => &[
                "https://github.com/mkxp-z/mkxp-z/releases/download/v2.4.0-github/mkxp-z_2.4.0-linux.zip",
                "https://github.com/mkxp-z/mkxp-z/releases/download/v2.4.0-github/mkxp-z_2.4.0-windows.zip"
            ],
            luminol_config::RGSSVer::MKXPFreebird => &[
                "https://mapleshrine.eu/releases/mkxp-freebird/win64/mkxp-win64-231004-8bdbef1.zip",
            ],
            _ => unreachable!(),
        };

        progress.zip_total.store(zip_url.len(), Ordering::Relaxed);

        let zips = futures::future::join_all(zip_url.iter().map(|url| reqwest::get(*url))).await;

        for (index, zip_response) in zips.into_iter().enumerate() {
            progress.zip_current.store(index, Ordering::Relaxed);

            progress.total_progress.store(0, Ordering::Relaxed);
            let response = zip_response
                .map_err(|e| format!("Error downloading {}: {e}", config.project.rgss_ver))?;

            let bytes = response.bytes().await.map_err(|e| {
                format!(
                    "Error getting response body for {}: {e}",
                    config.project.rgss_ver
                )
            })?;

            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| {
                format!(
                    "Failed to read zip archive for {}: {e}",
                    config.project.rgss_ver
                )
            })?;
            progress
                .total_progress
                .store(archive.len(), Ordering::Relaxed);

            for index in 0..archive.len() {
                let mut file = archive.by_index(index).unwrap();
                progress.current_progress.store(index, Ordering::Relaxed);

                let file_path = match file.enclosed_name() {
                    Some(p) => p.to_owned(),
                    None => continue,
                };

                let file_path = file_path
                    .strip_prefix("mkxp-z_2.4.0/")
                    .unwrap_or(&file_path);
                let file_path = file_path
                    .to_str()
                    .ok_or(format!("Invalid file path {file_path:#?}"))?;

                if file_path.is_empty()
                    || filesystem.exists(file_path).map_err(|e| e.to_string())?
                {
                    continue;
                }

                if file.is_dir() {
                    filesystem
                        .create_dir(file_path)
                        .map_err(|e| format!("Failed to create directory {file_path}: {e}"))?;
                } else {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())
                        .map_err(|e| format!("Failed to read file data {file_path}: {e}"))?;
                    filesystem
                        .write(file_path, bytes)
                        .map_err(|e| format!("Failed to save file data {file_path}: {e}"))?;
                }
            }
        }

        Ok(())
    }
}
