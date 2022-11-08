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

use std::io::Read;

use strum::IntoEnumIterator;

use crate::{data::config::RGSSVer, UpdateInfo};

use super::window::Window;

/// The new project window
pub struct NewProjectWindow {
    name: String,
    rgss_ver: RGSSVer,
    project_promise: Option<poll_promise::Promise<Result<(), String>>>,
    download_executable: bool,
    #[cfg(not(target_arch = "wasm32"))]
    init_git: bool,
    #[cfg(not(target_arch = "wasm32"))]
    git_branch_name: String,
}

impl Default for NewProjectWindow {
    fn default() -> Self {
        Self {
            name: "My Project".to_string(),
            rgss_ver: RGSSVer::RGSS1,
            project_promise: None,
            download_executable: false,
            #[cfg(not(target_arch = "wasm32"))]
            init_git: false,
            #[cfg(not(target_arch = "wasm32"))]
            git_branch_name: "master".to_string(),
        }
    }
}

impl Window for NewProjectWindow {
    fn name(&self) -> String {
        "New Project".to_string()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        let mut win_open = true;
        egui::Window::new(self.name())
            .open(&mut win_open)
            .show(ctx, |ui| {
                ui.label("Project Name");
                ui.text_edit_singleline(&mut self.name);

                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.checkbox(&mut self.init_git, "Initialize with git repository");
                    ui.add_enabled_ui(self.init_git, |ui| {
                        ui.label("Git Branch");
                        ui.text_edit_singleline(&mut self.git_branch_name);
                    });
                }

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

                ui.separator();

                ui.horizontal(|ui| {
                    if let Some(ref promise) = self.project_promise {
                        if let Some(res) = promise.ready() {
                            match res {
                                Ok(_) => *open = false,
                                Err(e) => {
                                    info.toasts.error(format!("Failed to create project: {e}"));
                                    self.project_promise = None;
                                }
                            }
                        }

                        ui.spinner();
                    } else if ui.button("Ok").clicked() {
                        let name = self.name.clone();
                        let rgss_ver = self.rgss_ver;
                        let download_executable = self.download_executable
                            && matches!(
                                rgss_ver,
                                RGSSVer::ModShot | RGSSVer::MKXPFreebird | RGSSVer::MKXPZ
                            );
                        #[cfg(not(target_arch = "wasm32"))]
                        let init_git = self.init_git;
                        #[cfg(not(target_arch = "wasm32"))]
                        let branch_name = self.git_branch_name.clone();

                        self.project_promise =
                            Some(poll_promise::Promise::spawn_local(async move {
                                let result = info
                                    .filesystem
                                    .try_create_project(name, info, rgss_ver)
                                    .await;

                                if result.is_ok() {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    if init_git {
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
                                                    ))
                                                }
                                            }
                                            Err(e) => info.toasts.error(format!(
                                                "Failed to initialize git repository {e}"
                                            )),
                                        }
                                    }

                                    if download_executable {
                                        if let Err(e) =
                                            Self::download_executable(rgss_ver, info).await
                                        {
                                            info.toasts.error(format!(
                                                "Failed to download {rgss_ver}: {e}",
                                            ));
                                        }
                                    }
                                }

                                result
                            }))
                    }
                    if ui.button("Cancel").clicked() {
                        *open = false;
                    }
                })
            });

        *open &= win_open;
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl NewProjectWindow {
    async fn download_executable(
        rgss_ver: RGSSVer,
        info: &'static UpdateInfo,
    ) -> Result<(), String> {
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
                "https://mapleshrine.eu/releases/mkxp-freebird/win64/mkxp-win64-211207-5d38b1f.zip",
            ],
            _ => unreachable!()
        };

        let zips = futures::future::join_all(zip_url.iter().map(|url| async move {
            surf::get(url)
                .middleware(surf::middleware::Redirect::new(10))
                .await
        }))
        .await;

        for zip_response in zips.into_iter() {
            let mut response = zip_response.map_err(|e| e.to_string())?;

            let bytes = response.body_bytes().await.map_err(|e| e.to_string())?;

            let mut archive =
                zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| e.to_string())?;

            for index in 0..archive.len() {
                let mut file = archive.by_index(index).unwrap();

                let file_path = match file.enclosed_name() {
                    Some(p) => p.to_owned(),
                    None => continue,
                };

                let file_path = file_path
                    .strip_prefix("mkxp-z_2.4.0/")
                    .unwrap_or(&file_path);
                let file_path = file_path.to_str().unwrap();

                if file_path.is_empty() || info.filesystem.file_exists(file_path).await {
                    continue;
                }

                if file.is_dir() {
                    info.filesystem.create_directory(file_path).await?;
                } else {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
                    info.filesystem.save_bytes_at(file_path, bytes).await?;
                }
            }
        }

        Ok(())
    }
}
