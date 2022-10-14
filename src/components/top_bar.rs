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

use poll_promise::Promise;

use crate::UpdateInfo;

/// The top bar for managing the project.
#[derive(Default)]
pub struct TopBar {
    open_project_promise: Option<Promise<Result<(), String>>>,
    save_project_promise: Option<Promise<Result<(), String>>>,
}

impl TopBar {
    /// Display the top bar.
    #[allow(unused_variables)]
    pub fn ui(&mut self, info: &'static UpdateInfo, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {
                info.toasts.error("Not implemented");
            }

            if self.open_project_promise.is_none() {
                if ui.button("Open Project").clicked() {
                    self.open_project_promise = Some(Promise::spawn_local(async move {
                        info.filesystem.try_open_project(info).await
                    }));
                }
            } else {
                if let Some(r) = self.open_project_promise.as_ref().unwrap().ready() {
                    match r {
                        Ok(_) => info.toasts.info("Opened project successfully!"),
                        Err(e) => info.toasts.error(e),
                    }
                    self.open_project_promise = None;
                }
                ui.spinner();
            }

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows();
                    info.tabs.clean_tabs();
                }

                if self.save_project_promise.is_none() {
                    if ui.button("Save Project").clicked() {
                        self.save_project_promise = Some(Promise::spawn_local(async move {
                            info.filesystem.save_cached(&info.data_cache).await
                        }));
                    }
                } else {
                    if let Some(r) = self.open_project_promise.as_ref().unwrap().ready() {
                        match r {
                            Ok(_) => {}
                            Err(e) => info.toasts.error(e),
                        }
                        self.save_project_promise = None;
                    }
                    ui.spinner();
                }
            });

            ui.separator();

            if ui.button("Egui Settings").clicked() {
                info.windows
                    .add_window(crate::windows::misc::EguiSettings::default())
            }
        });

        ui.separator();

        ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
            if ui.button("Maps").clicked() {
                info.windows
                    .add_window(crate::windows::map_picker::MapPicker::default())
            }

            if ui.button("Sound Test").clicked() {
                info.windows
                    .add_window(crate::windows::sound_test::SoundTest::new(info))
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows
                    .add_window(crate::windows::about::About::default());
            };

            ui.separator();

            if ui.button("Egui Inspection").clicked() {
                info.windows
                    .add_window(crate::windows::misc::EguiInspection::default())
            }

            if ui.button("Egui Memory").clicked() {
                info.windows
                    .add_window(crate::windows::misc::EguiMemory::default())
            }

            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.toggle_value(&mut debug_on_hover, "Debug on hover");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        });
    }
}
