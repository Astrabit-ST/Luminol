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
use std::collections::HashMap;

/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
#[derive(Default)]
pub struct Window {}

impl Window {
    fn render_submap(
        id: i32,
        children_data: &HashMap<i32, Vec<i32>>,
        mapinfos: &mut HashMap<i32, rpg::MapInfo>,
        info: &'static UpdateInfo,
        ui: &mut egui::Ui,
    ) {
        // We get the map name. It's assumed that there is in fact a map with this ID in mapinfos.
        let map_info = mapinfos.get_mut(&id).unwrap();
        let map_name = &map_info.name;
        // Does this map have children?
        if children_data.contains_key(&id) {
            // Render a custom collapsing header.
            // It's custom so we can add a button to open a map.
            let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(format!("map_info_{id}")),
                map_info.expanded,
            );

            map_info.expanded = header.openness(ui.ctx()) >= 1.;

            header
                .show_header(ui, |ui| {
                    // Has the user
                    if ui.button(map_name).double_clicked() {
                        Self::create_map_tab(id, map_name.clone(), info);
                    }
                })
                .body(|ui| {
                    for id in children_data.get(&id).unwrap() {
                        // Render children.
                        Self::render_submap(*id, children_data, mapinfos, info, ui);
                    }
                });
        } else {
            // Just display a label otherwise.
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent);
                if ui.button(map_name).double_clicked() {
                    Self::create_map_tab(id, map_name.clone(), info);
                }
            });
        }
    }

    fn create_map_tab(id: i32, name: String, info: &'static UpdateInfo) {
        if let Some(m) = map::Tab::new(id, name, info) {
            info.tabs.add_tab(m);
        }
    }
}

impl window::Window for Window {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static UpdateInfo) {
        let mut window_open = true;
        egui::Window::new("Map Picker")
            .open(&mut window_open)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // Aquire the data cache.
                        let mut mapinfos = info.data_cache.mapinfos();

                        // We sort maps by their order.
                        let mut sorted_maps = mapinfos.iter().collect::<Vec<_>>();
                        sorted_maps.sort_by(|a, b| a.1.order.cmp(&b.1.order));

                        // We preprocess maps to figure out what has nodes and what doesn't.
                        // This should result in an ordered hashmap of all the maps and their children.
                        let mut children_data: HashMap<i32, Vec<i32>> = HashMap::new();
                        for (id, map) in sorted_maps {
                            // Is there an entry for our parent?
                            // If not, then just add a blank vector to it.
                            let children = children_data.entry(map.parent_id).or_default();
                            children.push(*id);
                        }
                        children_data.entry(0).or_default(); // If there is no `0` entry (i.e. there are no maps) then add one.

                        // Now we can actually render all maps.
                        egui::CollapsingHeader::new("root")
                            .default_open(true)
                            .show(ui, |ui| {
                                // There will always be a map `0`.
                                // `0` is assumed to be the root map.
                                for id in children_data.get(&0).unwrap() {
                                    Self::render_submap(
                                        *id,
                                        &children_data,
                                        &mut mapinfos,
                                        info,
                                        ui,
                                    );
                                }
                            });
                    })
            });
        *open = window_open;
    }

    fn name(&self) -> String {
        "Map Picker".to_string()
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
