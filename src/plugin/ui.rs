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
use super::{result::Result, Manifest, LOADER};
use crate::{state, Window};
use dashmap::DashMap;
use log::debug;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct PluginManagerWindow {
    plugins: DashMap<String, Manifest>,
}
impl Default for PluginManagerWindow {
    fn default() -> Self {
        Self {
            plugins: DashMap::new(),
        }
    }
}
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
        let inner = egui::Window::new(self.name())
            .id(self.id())
            .open(open)
            .show(ctx, |ui| -> Result<()> {
                let plugins: Vec<Manifest> = LOADER.get_manifests().collect();

                /* TODO: For now, this field is static (will always print out "Active"), however this should be replaced with
                   something dynamic, once we implement a way for Luminol to NOT crash if interpreter failed to initialize. */
                ui.horizontal(|ui| {
                    ui.label("Interpreter Status: ");
                    ui.label(egui::RichText::new("Active").color(egui::Color32::GREEN));
                });
                egui::ScrollArea::both()
                    .show::<Result<()>>(ui, |ui| {
                        for manifest in plugins {
                            let is_plugin_active = LOADER.is_plugin_active(manifest.id.clone());
                            debug!("is_plugin_active = {}", is_plugin_active);
                            ui.with_layout(egui::Layout::left_to_right(egui::Align::LEFT), |ui| {
                                ui.vertical(|ui| {
                                    ui.label(manifest.name);
                                    ui.label(format!("{}@{}", manifest.id, manifest.version));
                                });
                                ui.vertical::<Result<()>>(|ui| {
                                    if ui.button(if is_plugin_active { "Enable" } else { "Disable" }).clicked() {
                                        if is_plugin_active {
                                            LOADER.activate_plugin(manifest.id)?;
                                        } else {
                                            LOADER.deactivate_plugin(manifest.id)?;
                                        }
                                    }

                                    Ok(())
                                });
                            });
                        }

                        Ok(())
                    }).inner?;
                if ui.button("Reload").clicked() {
                    LOADER.load("net.somedevfox.test")?;
                }
                if ui.button("Activate").clicked() {
                    LOADER.activate_plugin("net.somedevfox.test")?;
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
