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

pub use crate::prelude::*;

/// The script editor.
pub struct Window {
    tabs: tab::Tabs,
    selected_script: usize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            tabs: tab::Tabs::new("script_editor"),
            selected_script: 0,
        }
    }
}

impl window::Window for Window {
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
                            let scripts = info.data_cache.scripts();

                            for (id, script) in scripts.iter().enumerate() {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_script,
                                        id,
                                        script.name.clone(),
                                    )
                                    .double_clicked()
                                {
                                    match ScriptTab::new(id, script.clone()) {
                                        Ok(tab) => self.tabs.add_tab(tab),
                                        Err(e) => {
                                            info.toasts.error(format!("Error Opening Script: {e}"));
                                        }
                                    }
                                }
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

struct ScriptTab {
    name: String,
    id: usize,
    script: String,
    force_close: bool,
}

impl ScriptTab {
    fn new(id: usize, script: rpg::Script) -> Result<Self, String> {
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

impl tab::Tab for ScriptTab {
    fn name(&self) -> String {
        format!("{}: {}", self.name, self.id)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
        let theme = syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.horizontal(|ui| {
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

            if save_script {
                let mut scripts = info.data_cache.scripts();

                let mut encoder =
                    flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
                let result = encoder
                    .write_all(self.script.as_bytes())
                    .and_then(|_| encoder.finish());
                match result {
                    Err(e) => info.toasts.error(format!("Failed to encode script {e}")),
                    Ok(data) => {
                        let script = rpg::Script {
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
