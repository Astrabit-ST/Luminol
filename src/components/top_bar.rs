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

use std::sync::Arc;

use poll_promise::Promise;
use strum::IntoEnumIterator;

use crate::filesystem::Filesystem;
use crate::{Pencil, UpdateInfo};

/// The top bar for managing the project.
#[derive(Default)]
pub struct TopBar {
    open_project_promise: Option<Promise<Result<(), String>>>,
    save_project_promise: Option<Promise<Result<(), String>>>,
    egui_settings_open: bool,
    #[cfg(not(target_arch = "wasm32"))]
    fullscreen: bool,
}

impl TopBar {
    /// Display the top bar.
    #[allow(unused_variables)]
    pub fn ui(
        &mut self,
        info: &'static UpdateInfo,
        ui: &mut egui::Ui,
        style: &mut Arc<egui::Style>,
        frame: &mut eframe::Frame,
    ) {
        egui::widgets::global_dark_light_mode_switch(ui);

        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.checkbox(&mut self.fullscreen, "Fullscreen");

            frame.set_fullscreen(self.fullscreen);
        }

        let mut open_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O))
            && info.filesystem.project_loaded();
        let mut save_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S))
            && info.filesystem.project_loaded();
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            info.windows
                .add_window(crate::windows::new_project::NewProjectWindow::default());
        }

        ui.separator();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {
                info.windows
                    .add_window(crate::windows::new_project::NewProjectWindow::default());
            }

            if self.open_project_promise.is_none() {
                open_project |= ui.button("Open Project").clicked();
            } else {
                ui.spinner();
            }

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Project Config").clicked() {
                    info.windows
                        .add_window(crate::windows::config::ConfigWindow {});
                }

                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows();
                    info.tabs.clean_tabs();
                }

                if self.save_project_promise.is_none() {
                    save_project |= ui.button("Save Project").clicked();
                } else {
                    ui.spinner();
                }
            });

            ui.separator();

            // Or these together so if one OR the other is true the window shows.
            self.egui_settings_open =
                ui.button("Egui Settings").clicked() || self.egui_settings_open;

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.separator();

                if ui.button("Quit").clicked() {
                    frame.close();
                }
            }
        });

        ui.separator();

        ui.menu_button("Data", |ui| {
            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Maps").clicked() {
                    info.windows
                        .add_window(crate::windows::map_picker::MapPicker::default())
                }

                if ui.button("Items").clicked() {
                    info.windows
                        .add_window(crate::windows::items::ItemsWindow::new(
                            info.data_cache.items().clone().unwrap(),
                        ))
                }

                if ui.button("Common Events").clicked() {
                    info.windows
                        .add_window(crate::windows::common_event_edit::CommonEventEdit::default())
                }

                if ui.button("Scripts").clicked() {
                    info.windows
                        .add_window(crate::windows::script_edit::ScriptEdit::default())
                }

                if ui.button("Sound Test").clicked() {
                    info.windows
                        .add_window(crate::windows::sound_test::SoundTest::new(info))
                }
            });
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

            if ui.button("Profiler").clicked() {
                info.windows
                    .add_window(crate::windows::misc::Puffin::default())
            }

            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.toggle_value(&mut debug_on_hover, "Debug on hover");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        });

        ui.separator();

        ui.label("Brush:");

        let mut toolbar = info.toolbar.borrow_mut();
        for brush in Pencil::iter() {
            ui.selectable_value(&mut toolbar.pencil, brush, brush.to_string());
        }

        let ctx = ui.ctx();
        // Because style_ui makes a new style, AND we can't pass the style to a dedicated window, we handle the logic here.
        egui::Window::new("Egui Settings")
            .open(&mut self.egui_settings_open)
            .show(ui.ctx(), |ui| {
                ctx.style_ui(ui);
                *style = ctx.style();
            });

        if open_project {
            self.open_project_promise = Some(Promise::spawn_local(async move {
                info.filesystem.try_open_project(info).await
            }));
        }

        if save_project {
            info.toasts.info("Saving project...");
            self.save_project_promise = Some(Promise::spawn_local(async move {
                info.filesystem.save_cached(info).await
            }));
        }

        if self.open_project_promise.is_some() {
            if let Some(r) = self.open_project_promise.as_ref().unwrap().ready() {
                match r {
                    Ok(_) => info.toasts.info("Opened project successfully!"),
                    Err(e) => info.toasts.error(e),
                }
                self.open_project_promise = None;
            }
        }

        if self.save_project_promise.is_some() {
            if let Some(r) = self.save_project_promise.as_ref().unwrap().ready() {
                match r {
                    Ok(_) => info.toasts.info("Saved project sucessfully!"),
                    Err(e) => info.toasts.error(e),
                }
                self.save_project_promise = None;
            }
        }
    }
}
