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

use crate::UpdateInfo;
#[derive(Default)]
pub struct TopBar {}

impl TopBar {
    #[allow(unused_variables)]
    pub fn ui(&mut self, info: &UpdateInfo<'_>, ui: &mut egui::Ui) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {
                todo!()
            }

            if ui.button("Open Project").clicked() {
                if let Err(e) = info.filesystem.try_open_project(info.data_cache) {
                    info.toasts.error(e);
                } else {
                    info.toasts.info("Opened project successfully!");
                }
            }

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows();
                    info.tabs.clean_tabs();
                }

                if ui.button("Save Project").clicked() {
                    if let Err(e) = info.filesystem.save_cached(info.data_cache) {
                        info.toasts.error(e);
                    }
                }
            });
        });

        ui.separator();

        ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
            if ui.button("Maps").clicked() {
                info.windows
                    .add_window(crate::windows::map_picker::MapPicker::new())
            }

            if ui.button("Sound Test").clicked() {
                info.windows
                    .add_window(crate::windows::sound_test::SoundTest::new())
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows.add_window(crate::windows::about::About::new());
            };
        });
    }
}
