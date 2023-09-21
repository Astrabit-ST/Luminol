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

use crate::prelude::*;
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
            name: "My Project".to_string(),
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
                                    state!()
                                        .toasts
                                        .error(format!("Failed to create project: {e}"));
                                    self.project_promise = None;
                                }
                            }
                        }

                        if self.progress.zip_total.load(Ordering::Relaxed) != 0 {
                            ui.label(format!(
                                "Downloadind & Unzipping {}/{}",
                                self.progress.zip_current.load(Ordering::Relaxed) + 1,
                                self.progress.zip_total.load(Ordering::Relaxed)
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
                        if ui.button("Ok").clicked() {
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
                                                    state.toasts.error(format!(
                                                        "Failed to initialize git repository {e}"
                                                    ));
                                                }
                                            }
                                            Err(e) => state.toasts.error(format!(
                                                "Failed to initialize git repository {e}"
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

        todo!("this feature is temporarily unavailable while we investigate WebAssembly builds");
    }
}
