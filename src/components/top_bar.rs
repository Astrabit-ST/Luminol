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

use crate::fl;
use crate::prelude::*;

use crate::Pencil;

/// The top bar for managing the project.
#[derive(Default)]
pub struct TopBar {
    open_project_promise: Option<Promise<Result<(), String>>>,

    fullscreen: bool,
}

impl TopBar {
    /// Display the top bar.
    #[allow(unused_variables)]
    pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let state = state!();
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.checkbox(&mut self.fullscreen, fl!("fullscreen"));

        frame.set_fullscreen(self.fullscreen);

        let mut open_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O))
            && state.filesystem.project_loaded();
        let mut save_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S))
            && state.filesystem.project_loaded();
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            state.windows.add_window(new_project::Window::default());
        }

        ui.separator();

        ui.menu_button(fl!("topbar_file_section"), |ui| {
            ui.label(if let Some(path) = state.filesystem.project_path() {
                fl!("topbar_file_current_proj_label", path = path.to_string())
            } else {
                fl!("topbar_file_no_proj_open_label")
            });

            if ui.button(fl!("new_project")).clicked() {
                state.windows.add_window(new_project::Window::default());
            }

            open_project |= ui.button(fl!("open_project")).clicked();

            ui.separator();

            ui.add_enabled_ui(state.filesystem.project_loaded(), |ui| {
                if ui.button(fl!("topbar_file_proj_config_btn")).clicked() {
                    state.windows.add_window(config_window::Window {});
                }

                if ui.button(fl!("topbar_file_close_proj_btn")).clicked() {
                    state.windows.clean_windows();
                    state.tabs.clean_tabs(|t| !t.requires_filesystem());
                    state.audio.clear_sinks(); // audio loads files borrows from the filesystem. unloading while they are playing is a crash
                    state.filesystem.unload_project();
                }

                save_project |= ui.button(fl!("topbar_file_save_proj_btn")).clicked();
            });

            ui.separator();

            ui.add_enabled_ui(state.filesystem.project_loaded(), |ui| {
                if ui.button(fl!("topbar_file_command_maker_btn")).clicked() {
                    state
                        .windows
                        .add_window(crate::command_gen::CommandGeneratorWindow::default());
                }
            });

            ui.separator();

            if ui.button(fl!("topbar_file_quit_btn")).clicked() {
                frame.close();
            }
        });

        ui.separator();

        ui.menu_button("Edit", |ui| {
            //
            if ui.button("Preferences").clicked() {
                state
                    .windows
                    .add_window(global_config_window::Window::default())
            }

            if ui.button("Appearance").clicked() {
                state.windows.add_window(appearance::Window::default())
            }
        });

        ui.separator();

        ui.menu_button(fl!("topbar_data_section"), |ui| {
            ui.add_enabled_ui(state.filesystem.project_loaded(), |ui| {
                if ui.button(fl!("maps")).clicked() {
                    state.windows.add_window(map_picker::Window::default());
                }

                if ui.button(fl!("items")).clicked() {
                    state.windows.add_window(items::Window::default());
                }

                if ui.button(fl!("common_events")).clicked() {
                    state
                        .windows
                        .add_window(common_event_edit::Window::default());
                }

                if ui.button(fl!("scripts")).clicked() {
                    state.windows.add_window(script_edit::Window::default());
                }

                if ui.button(fl!("scripts")).clicked() {
                    state.windows.add_window(sound_test::Window::default());
                }
            });
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            ui.button("Contents").clicked();

            if ui.button("About...").clicked() {
                state.windows.add_window(about::Window::default());
            };
        });

        ui.menu_button("Debug", |ui| {
            if ui.button("Egui Inspection").clicked() {
                state.windows.add_window(misc::EguiInspection::default());
            }

            if ui.button(fl!("topbar_egui_memory_btn")).clicked() {
                state.windows.add_window(misc::EguiMemory::default());
            }

            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.toggle_value(&mut debug_on_hover, fl!("topbar_debug_on_hover_tv"));
            ui.ctx().set_debug_on_hover(debug_on_hover);

            ui.separator();

            if ui.button("Filesystem Debug").clicked() {
                state.windows.add_window(misc::FilesystemDebug::default());
            }
        });

        ui.separator();

        ui.add_enabled_ui(state.filesystem.project_loaded(), |ui| {
            if ui.button(fl!("topbar_playtest_btn")).clicked() {
                let mut cmd = luminol_term::CommandBuilder::new("steamshim");
                cmd.cwd(state.filesystem.project_path().expect("project not loaded"));

                let result = crate::windows::console::Console::new(cmd).or_else(|_| {
                    let mut cmd = luminol_term::CommandBuilder::new("game");
                    cmd.cwd(state.filesystem.project_path().expect("project not loaded"));

                    crate::windows::console::Console::new(cmd)
                });

                match result {
                    Ok(w) => state.windows.add_window(w),
                    Err(e) => state
                        .toasts
                        .error(fl!("toast_error_starting_game", why = e.to_string())),
                }
            }

            if ui.button(fl!("topbar_terminal_btn")).clicked() {
                #[cfg(windows)]
                let shell = "powershell";
                #[cfg(unix)]
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
                let mut cmd = luminol_term::CommandBuilder::new(shell);
                cmd.cwd(state.filesystem.project_path().expect("project not loaded"));

                match crate::windows::console::Console::new(cmd) {
                    Ok(w) => state.windows.add_window(w),
                    Err(e) => state
                        .toasts
                        .error(fl!("toast_error_starting_shell", why = e.to_string())),
                }
            }
        });

        ui.separator();

        ui.label(format!("{}:", fl!("topbar_brush_label")));

        let mut toolbar = state.toolbar.borrow_mut();
        for brush in Pencil::iter() {
            ui.selectable_value(&mut toolbar.pencil, brush, brush.to_string());
        }

        if open_project {
            self.open_project_promise = Some(Promise::spawn_local(
                state.filesystem.spawn_project_file_picker(),
            ));
        }

        if save_project {
            state.toasts.info(fl!("toast_info_saving_proj"));
            match state.data_cache.save() {
                Ok(_) => state.toasts.info(fl!("toast_info_saved_proj")),
                Err(e) => state.toasts.error(e),
            }
        }

        if self.open_project_promise.is_some() {
            if let Some(r) = self.open_project_promise.as_ref().unwrap().ready() {
                match r {
                    Ok(_) => state.toasts.info(fl!("toast_info_opened_proj")),
                    Err(e) => state.toasts.error(e),
                }
                self.open_project_promise = None;
            }
        }
    }
}
