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
use super::{loader::Manifest, result::Result, Manager};
use crate::{state, Window};
use log::debug;

#[derive(Debug, Default)]
pub struct PluginManagerWindow {}
impl Window for PluginManagerWindow {
    fn id(&self) -> egui::Id {
        egui::Id::new("pluginmgr")
    }

    fn name(&self) -> String {
        String::from("Plugin Manager")
    }

    fn requires_filesystem(&self) -> bool {
        false
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        let manager = Manager::get();
        let inner = egui::Window::new(self.name())
            .id(self.id())
            .open(open)
            .show(ctx, |ui| -> Result<()> {
                let plugins: Vec<Manifest> = manager.get_manifests().collect();

                /* TODO: For now, this field is static (will always print out "Active"), however this should be replaced with
                   something dynamic, once we implement a way for Luminol to NOT crash if interpreter failed to initialize. */
                ui.horizontal(|ui| {
                    ui.label("Interpreter Status: ");
                    ui.label(egui::RichText::new("Active").color(egui::Color32::GREEN));
                });
                egui::ScrollArea::both()
                    .show::<Result<()>>(ui, |ui| {
                        for manifest in plugins {
                            let is_plugin_active = manager.is_plugin_active(manifest.id.clone());
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                                ui.vertical(|ui| {
                                    ui.label(manifest.name);
                                    ui.label(format!("{}@{}", manifest.id, manifest.version));
                                });
                                ui.vertical::<Result<()>>(|ui| {
                                    if ui.button(if is_plugin_active { "Disable" } else { "Enable" }).clicked() {
                                        if is_plugin_active {
                                            manager.deactivate_plugin(manifest.id)?;
                                        } else {
                                            manager.activate_plugin(manifest.id)?;
                                        }
                                    }

                                    Ok(())
                                });
                            });
                        }

                        Ok(())
                    }).inner?;
                if ui.button("Reload").clicked() {
                    manager.reload_all();
                }
                Ok(())
            });
        if let Some(egui::InnerResponse {
            inner: Some(Err(why)),
            ..
        }) = inner
        {
            let why = why.to_string();
            /*let why = {
                let mut vec = why.chars().collect::<Vec<char>>();
                vec[0] = vec[0].to_uppercase().collect();
                vec.collect::<String>()
            };*/
            state!().toasts.error(why);
        }
    }
}
