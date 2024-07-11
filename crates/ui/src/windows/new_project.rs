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

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use color_eyre::eyre::WrapErr;
use luminol_filesystem::FileSystem;
use strum::IntoEnumIterator;

use std::io::Read;

/// The new project window
pub struct Window {
    name: String,
    rgss_ver: luminol_config::RGSSVer,
    editor_ver: luminol_config::RMVer,
    download_executable: bool,
    progress: Arc<Progress>,
    init_git: bool,
    git_branch_name: String,
}

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
            download_executable: false,
            progress: Arc::default(),
            init_git: false,
            git_branch_name: "master".to_string(),
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("New Project")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut win_open = true;
        egui::Window::new("New Project")
            .open(&mut win_open)
            .show(ctx, |ui| {
                ui.add_enabled_ui(
                    update_state
                        .project_manager
                        .create_project_promise
                        .is_none(),
                    |ui| {
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
                    },
                );

                ui.separator();

                if update_state
                    .project_manager
                    .create_project_promise
                    .is_some()
                {
                    let zip_current = self.progress.zip_current.load(Ordering::Relaxed) + 1;
                    let zip_total = self.progress.zip_total.load(Ordering::Relaxed);

                    if zip_total != 0 {
                        ui.label(format!(
                            "Downloading & Unzipping {zip_current} / {zip_total}"
                        ));
                    }

                    let total = self.progress.total_progress.load(Ordering::Relaxed);
                    let current = self.progress.current_progress.load(Ordering::Relaxed) + 1;

                    match total {
                        0 => ui.spinner(),
                        _ => ui.add(
                            egui::ProgressBar::new(current as f32 / total as f32)
                                .show_percentage()
                                .animate(true),
                        ),
                    };

                    ui.separator();
                }

                ui.horizontal(|ui| {
                    ui.add_enabled_ui(
                        update_state
                            .project_manager
                            .create_project_promise
                            .is_none(),
                        |ui| {
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

                                update_state
                                    .project_manager
                                    .run_custom(move |update_state| {
                                        update_state.project_manager.create_project_promise =
                                            Some(luminol_core::spawn_future(Self::setup_project(
                                                config,
                                                download_executable,
                                                init_git.then_some(branch_name),
                                                progress,
                                            )));
                                    });
                            }
                            if ui.button("Cancel").clicked() {
                                *open = false;
                            }
                        },
                    );
                })
            });

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
    ) -> luminol_core::project_manager::CreateProjectPromiseResult {
        // because we re-export host based on the platform specific filesystem, we don't actually need to change any of this code!
        let host_fs = luminol_filesystem::host::FileSystem::from_folder_picker().await?;

        host_fs.create_dir("Audio")?;
        host_fs.create_dir("Data")?;
        host_fs.create_dir("Graphics")?;

        host_fs.create_file(format!("{}.lumproj", config.project.project_name))?;

        let mut data_cache = luminol_core::Data::from_defaults();
        data_cache.save(&host_fs, &config)?;

        if download_executable {
            Self::download_executable(&config, &host_fs, progress)
                .await
                .wrap_err_with(|| format!("While downloading {}", config.project.rgss_ver))?;
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
                color_eyre::eyre::bail!("Failed to initialize git repository: {e}");
            }
        }

        Ok(luminol_core::project_manager::CreateProjectResult {
            data_cache,
            config,
            host_fs,
        })
    }

    async fn download_executable(
        config: &luminol_config::project::Config,
        filesystem: &impl luminol_filesystem::FileSystem,
        progress: Arc<Progress>,
    ) -> color_eyre::Result<()> {
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

        let zips =
            futures_util::future::join_all(zip_url.iter().map(|url| reqwest::get(*url))).await;

        for (index, zip_response) in zips.into_iter().enumerate() {
            progress.zip_current.store(index, Ordering::Relaxed);

            progress.total_progress.store(0, Ordering::Relaxed);
            let response = zip_response
                .map_err(color_eyre::Report::from)
                .wrap_err("While downloading the zip")?;

            let bytes = response.bytes().await?;

            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                .wrap_err("While reading the zip archive")?;
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
                    .ok_or(color_eyre::eyre::eyre!("Invalid file path {file_path:#?}"))?;

                if file_path.is_empty() || filesystem.exists(file_path)? {
                    continue;
                }

                if file.is_dir() {
                    filesystem
                        .create_dir(file_path)
                        .wrap_err_with(|| format!("While creating the directory {file_path}"))?;
                } else {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)
                        .wrap_err_with(|| format!("While reading the file {file_path}"))?;
                    filesystem
                        .write(file_path, bytes)
                        .wrap_err_with(|| format!("While writing the file {file_path}"))?;
                }
            }
        }

        Ok(())
    }
}
