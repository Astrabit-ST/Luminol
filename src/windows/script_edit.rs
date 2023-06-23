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

pub use crate::prelude::*;

/// The script editor.
pub struct Window {
    tabs: tab::Tabs<ScriptTab>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            tabs: tab::Tabs::new("script_editor", vec![]),
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

    fn id(&self) -> egui::Id {
        egui::Id::new("Script Edit")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .id(egui::Id::new("script_editor_window"))
            .show(ctx, |ui| {
                egui::SidePanel::left("script_edit_script_panel").show_inside(ui, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let mut scripts = state!().data_cache.scripts();

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
                                    self.tabs
                                        .add_tab(ScriptTab::new(index, script.script_text.clone()));
                                }
                            }

                            if let Some(index) = insert_index {
                                scripts.insert(
                                    index,
                                    rpg::Script {
                                        name: "New Script".to_string(),
                                        script_text: String::new(),
                                    },
                                );
                            }

                            if let Some(index) = del_index {
                                scripts.remove(index);
                            }
                        });
                });

                self.tabs.ui(ui);
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

/// FIXME: Change behavior of script tab to aboid panics and stay synchronized
struct ScriptTab {
    index: usize,
    script_text: String,
    force_close: bool,
}

impl ScriptTab {
    fn new(index: usize, script_text: String) -> Self {
        Self {
            index,
            script_text,
            force_close: false,
        }
    }
}

impl tab::Tab for ScriptTab {
    fn name(&self) -> String {
        self.index.to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_script_edit").with(self.index)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let theme = global_config!().theme;
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
                let mut scripts = state!().data_cache.scripts();

                scripts[self.index].script_text = self.script_text.clone();
            }
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = syntax_highlighting::highlight(ui.ctx(), theme, string, "rb");
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut self.script_text)
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
