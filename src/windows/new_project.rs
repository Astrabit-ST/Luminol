// Copyright (C) 2022 Lily Lyons
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

use std::{cell::Cell, io::Read, rc::Rc};

use strum::IntoEnumIterator;

use crate::prelude::*;

/// The new project window
pub struct Window {
    name: String,
    rgss_ver: RGSSVer,
    project_promise: Option<poll_promise::Promise<Result<(), String>>>,
    download_executable: bool,
    progress: Progress,
    init_git: bool,
    git_branch_name: String,
}

#[derive(Clone, Default)]
struct Progress {
    total_progress: Rc<Cell<usize>>,
    current_progress: Rc<Cell<usize>>,
    zip_total: Rc<Cell<usize>>,
    zip_current: Rc<Cell<usize>>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            name: "My Project".to_string(),
            rgss_ver: RGSSVer::RGSS1,
            project_promise: None,
            download_executable: false,
            progress: Progress::default(),
            init_git: false,
            git_branch_name: "master".to_string(),
        }
    }
}

impl window::Window for Window {
    fn name(&self) -> String {
        "New Project".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("New Project")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
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
                            for ver in RGSSVer::iter() {
                                ui.selectable_value(&mut self.rgss_ver, ver, ver.to_string());
                            }
                        });

                    if matches!(
                        self.rgss_ver,
                        RGSSVer::ModShot | RGSSVer::MKXPFreebird | RGSSVer::MKXPZ
                    ) {
                        ui.checkbox(
                            &mut self.download_executable,
                            format!("Download latest version of {}", self.rgss_ver),
                        );
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    if let Some(ref promise) = self.project_promise {
                        if let Some(res) = promise.ready() {
                            match res {
                                Ok(_) => *open = false,
                                Err(e) => {
                                    info!()
                                        .toasts
                                        .error(format!("Failed to create project: {e}"));
                                    self.project_promise = None;
                                }
                            }
                        }

                        if self.progress.zip_total.get() != 0 {
                            ui.label(format!(
                                "Downloadind & Unzipping {}/{}",
                                self.progress.zip_current.get() + 1,
                                self.progress.zip_total.get()
                            ));
                        }

                        let total = self.progress.total_progress.get();
                        let current = self.progress.current_progress.get() + 1;
                        if total == 0 {
                            ui.spinner();
                        } else {
                            // FIXME: find a way to avoid cast precision loss
                            #[allow(clippy::cast_precision_loss)]
                            ui.add({
                                egui::ProgressBar::new(current as f32 / total as f32)
                                    .animate(true)
                                    .show_percentage()
                            });
                        }
                    } else {
                        if ui.button("Ok").clicked() {
                            let name = self.name.clone();
                            let rgss_ver = self.rgss_ver;
                            let download_executable = self.download_executable
                                && matches!(
                                    rgss_ver,
                                    RGSSVer::ModShot | RGSSVer::MKXPFreebird | RGSSVer::MKXPZ
                                );
                            let progress = self.progress.clone();

                            let init_git = self.init_git;

                            let branch_name = self.git_branch_name.clone();

                            self.project_promise =
                                Some(poll_promise::Promise::spawn_local(async move {
                                    let info = info!();
                                    let result =
                                        info.filesystem.try_create_project(name, rgss_ver).await;

                                    if init_git && result.is_ok() {
                                        use std::process::Command;
                                        match Command::new("git")
                                            .arg("init")
                                            .arg("-b")
                                            .arg(branch_name)
                                            .current_dir(info.filesystem.project_path().unwrap())
                                            .spawn()
                                        {
                                            Ok(mut c) => {
                                                if let Err(e) = c.wait() {
                                                    info.toasts.error(format!(
                                                        "Failed to initialize git repository {e}"
                                                    ));
                                                }
                                            }
                                            Err(e) => info.toasts.error(format!(
                                                "Failed to initialize git repository {e}"
                                            )),
                                        }
                                    }

                                    if download_executable && result.is_ok() {
                                        if let Err(e) =
                                            Self::download_executable(rgss_ver, progress).await
                                        {
                                            info.toasts.error(e);
                                        }
                                    }

                                    result
                                }));
                        }
                        if ui.button("Cancel").clicked() {
                            *open = false;
                        }
                    }
                })
            });

        *open &= win_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl Window {
    async fn download_executable(rgss_ver: RGSSVer, progress: Progress) -> Result<(), String> {
        let zip_url: &[_] = match rgss_ver {
            RGSSVer::ModShot => &[
                "https://github.com/thehatkid/ModShot/releases/download/latest/ModShot_Windows_bb6bcbc_Ruby-3.1-ucrt64_Steam-false.zip", 
                "https://github.com/thehatkid/ModShot/releases/download/latest/ModShot_Linux_bb6bcbc_Ruby-3.1_Steam-false.zip"
            ],
            RGSSVer::MKXPZ => &[
                "https://github.com/mkxp-z/mkxp-z/releases/download/v2.4.0-github/mkxp-z_2.4.0-linux.zip",
                "https://github.com/mkxp-z/mkxp-z/releases/download/v2.4.0-github/mkxp-z_2.4.0-windows.zip"
            ],
            RGSSVer::MKXPFreebird => &[
                // The cert has expired for mapleshrine.eu.
                // "https://mapleshrine.eu/releases/mkxp-freebird/win64/mkxp-win64-211207-5d38b1f.zip",
                // Use an unofficial host for now
                "https://nowaffles.com/wp-content/uploads/2022/11/mkxp-win64-211207-5d38b1f.zip",
                ],
                _ => unreachable!()
        };

        progress.zip_total.set(zip_url.len());

        let zips = futures::future::join_all(zip_url.iter().map(|url|
            // surf::get(format!("https://api.allorigins.win/raw?url={url}"))  FIXME: phishing scam, apparently
            surf::get(url)
            .middleware(surf::middleware::Redirect::new(10))))
        .await;

        for (index, zip_response) in zips.into_iter().enumerate() {
            progress.zip_current.set(index);

            progress.total_progress.set(0);
            let mut response =
                zip_response.map_err(|e| format!("Error downloading {rgss_ver}: {e}"))?;

            let bytes = response
                .body_bytes()
                .await
                .map_err(|e| format!("Error getting response body for {rgss_ver}: {e}"))?;

            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
                .map_err(|e| format!("Failed to read zip archive for {rgss_ver}: {e}"))?;
            progress.total_progress.set(archive.len());

            let info = info!();
            for index in 0..archive.len() {
                let mut file = archive.by_index(index).unwrap();
                progress.current_progress.set(index);

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

                if file_path.is_empty() || info.filesystem.path_exists(file_path) {
                    continue;
                }

                if file.is_dir() {
                    info.filesystem
                        .create_directory(file_path)
                        .map_err(|e| format!("Failed to create directory {file_path}: {e}"))?;
                } else {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())
                        .map_err(|e| format!("Failed to read file data {file_path}: {e}"))?;
                    info.filesystem
                        .save_data(file_path, bytes)
                        .map_err(|e| format!("Failed to save file data {file_path}: {e}"))?;
                }
            }
        }

        Ok(())
    }
}
