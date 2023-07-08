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

use crate::{fl, prelude::*};
use config::{RGSSVer, RMVer};

use std::io::Read;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// The new project window
pub struct Window {
    name: String,
    rgss_ver: RGSSVer,
    editor_ver: RMVer,
    project_promise: Option<poll_promise::Promise<Result<(), String>>>,
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
            name: fl!("window_new_proj_my_proj_str"),
            rgss_ver: RGSSVer::RGSS1,
            editor_ver: RMVer::XP,
            project_promise: None,
            download_executable: false,
            progress: Arc::default(),
            init_git: false,
            git_branch_name: "master".to_string(),
        }
    }
}

impl window::Window for Window {
    fn name(&self) -> String {
        fl!("new_project")
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
                    ui.label(fl!("window_new_proj_name_label"));
                    ui.text_edit_singleline(&mut self.name);

                    ui.checkbox(&mut self.init_git, fl!("window_new_proj_with_git_cb"));
                    ui.add_enabled_ui(self.init_git, |ui| {
                        ui.label(fl!("window_new_proj_git_branch_label"));
                        ui.text_edit_singleline(&mut self.git_branch_name);
                    });

                    egui::ComboBox::from_label(fl!("window_new_proj_rgss_runtime_label"))
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
                            fl!(
                                "window_new_proj_with_exe_download_cb",
                                variant = self.rgss_ver.to_string()
                            ),
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
                                    state!().toasts.error(fl!(
                                        "toast_error_creating_proj",
                                        why = e.to_string()
                                    ));
                                    self.project_promise = None;
                                }
                            }
                        }

                        if self.progress.zip_total.load(Ordering::Relaxed) != 0 {
                            ui.label(fl!(
                                "window_new_proj_dl_and_unzipping_label",
                                current = (self.progress.zip_current.load(Ordering::Relaxed) + 1),
                                total = self.progress.zip_total.load(Ordering::Relaxed)
                            ));
                        }

                        let total = self.progress.total_progress.load(Ordering::Relaxed);
                        let current = self.progress.current_progress.load(Ordering::Relaxed) + 1;
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
                        if ui.button(fl!("ok")).clicked() {
                            let rgss_ver = self.rgss_ver;
                            let config = config::project::Config {
                                project_name: self.name.clone(),
                                rgss_ver,
                                editor_ver: self.editor_ver,
                                ..Default::default()
                            };
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
                                    let state = state!();
                                    let result = state.data_cache.create_project(config).await;

                                    if init_git && result.is_ok() {
                                        use std::process::Command;
                                        match Command::new("git")
                                            .arg("init")
                                            .arg("-b")
                                            .arg(branch_name)
                                            .current_dir(state.filesystem.project_path().unwrap())
                                            .spawn()
                                        {
                                            Ok(mut c) => {
                                                if let Err(e) = c.wait() {
                                                    state.toasts.error(fl!(
                                                        "toast_error_init_git",
                                                        why = e.to_string()
                                                    ));
                                                }
                                            }
                                            Err(e) => state.toasts.error(fl!(
                                                "toast_error_init_git",
                                                why = e.to_string()
                                            )),
                                        }
                                    }

                                    if download_executable && result.is_ok() {
                                        if let Err(e) =
                                            Self::download_executable(rgss_ver, progress).await
                                        {
                                            state.toasts.error(e);
                                        }
                                    }

                                    result
                                }));
                        }
                        if ui.button(fl!("cancel")).clicked() {
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
    async fn download_executable(rgss_ver: RGSSVer, progress: Arc<Progress>) -> Result<(), String> {
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

        progress.zip_total.store(zip_url.len(), Ordering::Relaxed);

        let zips = futures::future::join_all(zip_url.iter().map(|url|
            // surf::get(format!("https://api.allorigins.win/raw?url={url}"))  FIXME: phishing scam, apparently
            surf::get(url)
            .middleware(surf::middleware::Redirect::new(10))))
        .await;

        for (index, zip_response) in zips.into_iter().enumerate() {
            progress.zip_current.store(index, Ordering::Relaxed);

            progress.total_progress.store(0, Ordering::Relaxed);
            let mut response = zip_response.map_err(|e| {
                fl!(
                    "toast_error_downloading_rgss",
                    variant = rgss_ver.to_string(),
                    why = e.to_string()
                )
            })?;

            let bytes = response.body_bytes().await.map_err(|e| {
                fl!(
                    "toast_error_getting_body_resp",
                    variant = rgss_ver.to_string(),
                    why = e.to_string()
                )
            })?;

            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| {
                fl!(
                    "toast_error_read_zip",
                    variant = rgss_ver.to_string(),
                    why = e.to_string()
                )
            })?;
            progress
                .total_progress
                .store(archive.len(), Ordering::Relaxed);

            let state = state!();
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
                let file_path = file_path.to_str().ok_or(fl!(
                    "toast_error_invalid_file_path",
                    file_path = file_path.to_string_lossy()
                ))?;

                if file_path.is_empty()
                    || state
                        .filesystem
                        .exists(file_path)
                        .map_err(|e| e.to_string())?
                {
                    continue;
                }

                if file.is_dir() {
                    state.filesystem.create_dir(file_path).map_err(|e| {
                        fl!(
                            "toast_error_create_dir",
                            file_path = file_path,
                            why = e.to_string()
                        )
                    })?;
                } else {
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes)
                        .map_err(|e| e.to_string())
                        .map_err(|e| {
                            fl!(
                                "toast_error_reading_file_data",
                                file_path = file_path,
                                why = e
                            )
                        })?;
                    state.filesystem.write(file_path, bytes).map_err(|e| {
                        fl!(
                            "toast_error_saving_file_data",
                            file_path = file_path,
                            why = e.to_string()
                        )
                    })?;
                }
            }
        }

        Ok(())
    }
}
