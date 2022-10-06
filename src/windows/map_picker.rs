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

use crate::data::rmxp_structs::rpg::MapInfo;
use crate::tabs::map::Map;
use crate::UpdateInfo;
use std::collections::HashMap;

/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
pub struct MapPicker {}

impl MapPicker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_submap(
        id: &i32,
        children_data: &HashMap<i32, Vec<i32>>,
        mapinfos: &HashMap<i32, MapInfo>,
        info: &'static UpdateInfo,
        ui: &mut egui::Ui,
    ) {
        // We get the map name. It's assumed that there is in fact a map with this ID in mapinfos.
        let map_info = mapinfos.get(id).unwrap();
        let map_name = &map_info.name;
        // Does this map have children?
        if children_data.contains_key(id) {
            // Render a custom collapsing header.
            // It's custom so we can add a button to open a map.
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(format!("map_info_{}", id)),
                map_info.expanded,
            )
            .show_header(ui, |ui| {
                // Has the user
                if ui.button(map_name).double_clicked() {
                    Self::create_map_tab(*id, map_name.clone(), info);
                }
            })
            .body(|ui| {
                for id in children_data.get(id).unwrap() {
                    // Render children.
                    Self::render_submap(id, children_data, mapinfos, info, ui)
                }
            });
        } else {
            // Just display a label otherwise.
            if ui.button(map_name).double_clicked() {
                Self::create_map_tab(*id, map_name.clone(), info);
            }
        }
    }

    fn create_map_tab(id: i32, name: String, info: &'static UpdateInfo) {
        if let Some(m) = Map::new(id, name, info) {
            info.tabs.add_tab(m);
        }
    }
}

impl super::window::Window for MapPicker {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static UpdateInfo) {
        let mut window_open = true;
        egui::Window::new("Map Picker")
            .open(&mut window_open)
            .show(ctx, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    // Aquire the data cache.
                    let mapinfos = info.data_cache.map_infos();
                    let mapinfos = match mapinfos.as_ref() {
                        Some(m) => m,
                        None => {
                            *open = false;
                            info.toasts.error("MapInfos not loaded.");
                            return;
                        }
                    };

                    // We sort maps by their order.
                    let mut sorted_maps = Vec::from_iter(mapinfos.iter());
                    sorted_maps.sort_by(|a, b| a.1.order.cmp(&b.1.order));

                    // We preprocess maps to figure out what has nodes and what doesn't.
                    // This should result in an ordered hashmap of all the maps and their children.
                    let mut children_data: HashMap<i32, Vec<i32>> = HashMap::new();
                    for (id, map) in sorted_maps {
                        // Is there an entry for our parent?
                        // If not, then just add a blank vector to it.
                        let children = children_data.entry(map.parent_id).or_insert(vec![]);
                        children.push(*id);
                    }
                    children_data.entry(0).or_insert(vec![]); // If there is no `0` entry (i.e. there are no maps) then add one.

                    // Now we can actually render all maps.
                    ui.collapsing("root", |ui| {
                        // There will always be a map `0`.
                        // `0` is assumed to be the root map.
                        for id in children_data.get(&0).unwrap() {
                            Self::render_submap(id, &children_data, mapinfos, info, ui);
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
