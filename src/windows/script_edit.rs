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

use std::io::{Read, Write};

use super::window::Window;
use crate::components::syntax_highlighting;
use crate::{
    data::rmxp_structs::intermediate::Script,
    tabs::tab::{Tab, Tabs},
};

/// The script editor.
pub struct ScriptEdit {
    tabs: Tabs,
}

impl Default for ScriptEdit {
    fn default() -> Self {
        Self {
            tabs: Tabs::new("script_editor"),
        }
    }
}

impl Window for ScriptEdit {
    fn name(&self) -> String {
        self.tabs
            .focused_name()
            .map_or("Scripts".to_string(), |name| {
                format!("Editing Script {name}")
            })
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        egui::Window::new(self.name())
            .open(open)
            .id(egui::Id::new("script_editor_window"))
            .show(ctx, |ui| {
                egui::SidePanel::left("script_edit_script_panel").show_inside(ui, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let mut scripts = info.data_cache.scripts();
                            let scripts = scripts.as_mut().unwrap();

                            let mut insert_index = None;
                            let mut del_index = None;

                            let scripts_len = scripts.len();
                            for (index, script) in scripts.iter_mut().enumerate() {
                                let response = ui
                                    .text_edit_singleline(&mut script.name)
                                    .context_menu(|ui| {
                                        if ui.button("Insert").clicked() {
                                            insert_index = Some(index);
                                        }

                                        ui.add_enabled_ui(scripts_len > 1, |ui| {
                                            if ui.button("Delete").clicked() {
                                                del_index = Some(index);
                                            }
                                        });
                                    });

                                if response.double_clicked() {
                                    match ScriptTab::new(index, script.clone()) {
                                        Ok(tab) => self.tabs.add_tab(tab),
                                        Err(e) => {
                                            info.toasts.error(format!("Error Opening Script: {e}"));
                                        }
                                    }
                                }
                            }

                            if let Some(index) = insert_index {
                                scripts.insert(
                                    index,
                                    Script {
                                        id: index,
                                        name: "New Script".to_string(),
                                        data: vec![0x78, 0x9C, 0x03, 0x00, 0x00, 0x00, 0x00, 0x01],
                                    },
                                );
                            }

                            if let Some(index) = del_index {
                                scripts.remove(index);
                            }
                        });
                });

                self.tabs.ui(ui, info);
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

/// FIXME: Change behavior of script tab to aboid panics and stay synchronized
struct ScriptTab {
    name: String,
    id: usize,
    script: String,
    force_close: bool,
}

impl ScriptTab {
    fn new(id: usize, script: Script) -> Result<Self, String> {
        let mut decoder = flate2::bufread::ZlibDecoder::new(&script.data[..]);
        let mut script_data = String::new();
        decoder
            .read_to_string(&mut script_data)
            .map_err(|e| e.to_string())?;

        Ok(Self {
            name: script.name,
            id,
            script: script_data,
            force_close: false,
        })
    }
}

impl Tab for ScriptTab {
    fn name(&self) -> String {
        format!("{}: {}", self.name, self.id)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
        let mut theme = syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.horizontal(|ui| {
            ui.collapsing("Theme", |ui| {
                ui.group(|ui| {
                    theme.ui(ui);
                    theme.clone().store_in_memory(ui.ctx());
                });
            });

            let mut save_script = false;

            if ui.button("Ok").clicked() {
                save_script = true;
                self.force_close = true;
            }

            if ui.button("Cancel").clicked() {
                self.force_close = true;
            }

            if ui.button("Apply").clicked() {
                save_script = true;
            }

            // FIXME: perform this on deserialization/serialization, not here
            if save_script {
                let mut scripts = info.data_cache.scripts();
                let scripts = scripts.as_mut().unwrap();

                let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), Default::default());
                let result = encoder
                    .write_all(self.script.as_bytes())
                    .and_then(|_| encoder.finish());
                match result {
                    Err(e) => info.toasts.error(format!("Failed to encode script {e}")),
                    Ok(data) => {
                        let script = Script {
                            id: 0,
                            name: self.name.clone(),
                            data,
                        };

                        scripts[self.id] = script;
                    }
                }
            }

            ui.label("Name");
            ui.text_edit_singleline(&mut self.name);
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = syntax_highlighting::highlight(ui.ctx(), &theme, string, "rb");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.script)
                    .code_editor()
                    .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }
}
