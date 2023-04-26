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

use crate::prelude::*;

use crate::Pencil;

/// The top bar for managing the project.
#[derive(Default)]
pub struct TopBar {
    open_project_promise: Option<Promise<Result<(), String>>>,
    egui_settings_open: bool,
    fullscreen: bool,
}

impl TopBar {
    /// Display the top bar.
    #[allow(unused_variables)]
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        style: &mut Arc<egui::Style>,
        frame: &mut eframe::Frame,
    ) {
        let info = info!();
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.checkbox(&mut self.fullscreen, "Fullscreen");

        frame.set_fullscreen(self.fullscreen);

        let mut open_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::O))
            && info.filesystem.project_loaded();
        let mut save_project = ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S))
            && info.filesystem.project_loaded();
        if ui.input(|i| i.modifiers.command && i.key_pressed(egui::Key::N)) {
            info.windows.add_window(new_project::Window::default());
        }

        ui.separator();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {
                info.windows.add_window(new_project::Window::default());
            }

            open_project |= ui.button("Open Project").clicked();

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Project Config").clicked() {
                    info.windows.add_window(config::Window {});
                }

                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows();
                    info.tabs.clean_tabs(|t| t.requires_filesystem());
                }

                save_project |= ui.button("Save Project").clicked();
            });

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Command Maker").clicked() {
                    info.windows
                        .add_window(crate::command_gen::CommandGeneratorWindow::default());
                }
            });

            ui.separator();

            if ui.button("Quit").clicked() {
                frame.close();
            }
        });

        ui.separator();

        ui.menu_button("Appearance", |ui| {
            // Or these together so if one OR the other is true the window shows.
            self.egui_settings_open =
                ui.button("Egui Settings").clicked() || self.egui_settings_open;

            ui.menu_button("Catppuccin theme", |ui| {
                if ui.button("Frappe").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::FRAPPE);
                }
                if ui.button("Latte").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::LATTE);
                }
                if ui.button("Macchiato").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::MACCHIATO);
                }
                if ui.button("Mocha").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::MOCHA);
                }

                *style = ui.ctx().style();
            });

            let theme = &mut info.saved_state.borrow_mut().theme;
            ui.menu_button("Code Theme", |ui| {
                theme.ui(ui);

                ui.label("Code sample");
                ui.label(syntax_highlighting::highlight(
                    ui.ctx(),
                    *theme,
                    r#"
                    class Foo < Array 
                    end
                    def bar(baz) 
                    end
                    print 1, 2.0
                    puts [0x3, :4, '5']
                    "#,
                    "rb",
                ));
            });
        });

        ui.separator();

        ui.menu_button("Data", |ui| {
            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Maps").clicked() {
                    info.windows.add_window(map_picker::Window::default());
                }

                if ui.button("Items").clicked() {
                    info.windows.add_window(items::Window::default());
                }

                if ui.button("Common Events").clicked() {
                    info.windows
                        .add_window(common_event_edit::Window::default());
                }

                if ui.button("Scripts").clicked() {
                    info.windows.add_window(script_edit::Window::default());
                }

                if ui.button("Sound Test").clicked() {
                    info.windows.add_window(sound_test::Window::default());
                }
            });
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows.add_window(about::Window::default());
            };

            ui.separator();

            if ui.button("Egui Inspection").clicked() {
                info.windows.add_window(misc::EguiInspection::default());
            }

            if ui.button("Egui Memory").clicked() {
                info.windows.add_window(misc::EguiMemory::default());
            }

            let mut debug_on_hover = ui.ctx().debug_on_hover();
            ui.toggle_value(&mut debug_on_hover, "Debug on hover");
            ui.ctx().set_debug_on_hover(debug_on_hover);
        });

        ui.separator();

        ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
            if ui.button("Playtest").clicked() {
                let mut cmd = luminol_term::CommandBuilder::new("steamshim");
                cmd.cwd(info.filesystem.project_path().expect("project not loaded"));

                let result = crate::windows::console::Console::new(cmd).or_else(|_| {
                    let mut cmd = luminol_term::CommandBuilder::new("game");
                    cmd.cwd(info.filesystem.project_path().expect("project not loaded"));

                    crate::windows::console::Console::new(cmd)
                });

                match result {
                    Ok(w) => info.windows.add_window(w),
                    Err(e) => info.toasts.error(format!(
                        "error starting game (tried steamshim.exe and then game.exe): {e}"
                    )),
                }
            }

            if ui.button("Terminal").clicked() {
                #[cfg(windows)]
                let shell = "powershell";
                #[cfg(unix)]
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());
                let mut cmd = luminol_term::CommandBuilder::new(shell);
                cmd.cwd(info.filesystem.project_path().expect("project not loaded"));

                match crate::windows::console::Console::new(cmd) {
                    Ok(w) => info.windows.add_window(w),
                    Err(e) => info.toasts.error(format!("error starting shell: {e}")),
                }
            }
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
            self.open_project_promise = Some(Promise::spawn_local(
                info.filesystem.spawn_project_file_picker(),
            ));
        }

        if save_project {
            info.toasts.info("Saving project...");
            match info.filesystem.save_cached() {
                Ok(_) => info.toasts.info("Saved project sucessfully!"),
                Err(e) => info.toasts.error(e),
            }
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
    }
}
