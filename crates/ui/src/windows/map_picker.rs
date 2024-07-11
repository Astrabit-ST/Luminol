// Copyright (C) 2024 Melody Madeline Lyons
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

use std::collections::{BTreeMap, BTreeSet};

/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
#[derive(Default)]
pub struct Window {}

impl Window {
    fn render_submap(
        id: usize,
        children_data: &BTreeMap<usize, BTreeSet<usize>>,
        mapinfos: &mut luminol_data::rpg::MapInfos,
        open_map_id: &mut Option<usize>,
        ui: &mut egui::Ui,
    ) {
        // We get the map name. It's assumed that there is in fact a map with this ID in mapinfos.
        let map_info = mapinfos.data.get_mut(&id).unwrap();

        // Does this map have children?
        if children_data.contains_key(&id) {
            // Render a custom collapsing header.
            // It's custom so we can add a button to open a map.
            let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(egui::Id::new("luminol_map_info").with(id)),
                map_info.expanded,
            );

            map_info.expanded = header.openness(ui.ctx()) >= 1.;

            header
                .show_header(ui, |ui| {
                    // Has the user
                    if ui.text_edit_singleline(&mut map_info.name).double_clicked() {
                        *open_map_id = Some(id)
                    }
                })
                .body(|ui| {
                    for id in children_data.get(&id).unwrap() {
                        // Render children.
                        Self::render_submap(*id, children_data, mapinfos, open_map_id, ui);
                    }
                });
        } else {
            // Just display a label otherwise.
            ui.horizontal(|ui| {
                ui.add_space(ui.spacing().indent);
                if ui.text_edit_singleline(&mut map_info.name).double_clicked() {
                    *open_map_id = Some(id)
                }
            });
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("Map Picker")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut window_open = true;
        egui::Window::new("Map Picker")
            .open(&mut window_open)
            .show(ctx, |ui| {
                egui::ScrollArea::both()
                    .id_source(
                        update_state
                            .project_config
                            .as_ref()
                            .expect("project not loaded")
                            .project
                            .persistence_id,
                    )
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // Aquire the data cache.
                        let mut mapinfos = update_state.data.map_infos();

                        // We preprocess maps to figure out what has nodes and what doesn't.
                        // This should result in an ordered hashmap of all the maps and their children.
                        let mut children_data: BTreeMap<_, BTreeSet<_>> = BTreeMap::new();
                        for (&id, map) in mapinfos.data.iter() {
                            // Is there an entry for our parent?
                            // If not, then just add a blank vector to it.
                            let children = children_data.entry(map.parent_id).or_default(); // FIXME: this doesn't handle sorting properly
                            children.insert(id);
                        }
                        children_data.entry(0).or_default(); // If there is no `0` entry (i.e. there are no maps) then add one.

                        let mut open_map_id = None;

                        // Now we can actually render all maps.
                        egui::CollapsingHeader::new("root")
                            .default_open(true)
                            .show(ui, |ui| {
                                // There will always be a map `0`.
                                // `0` is assumed to be the root map.
                                for &id in children_data.get(&0).unwrap() {
                                    Self::render_submap(
                                        id,
                                        &children_data,
                                        &mut mapinfos,
                                        &mut open_map_id,
                                        ui,
                                    );
                                }
                            });

                        drop(mapinfos);

                        if let Some(id) = open_map_id {
                            match crate::tabs::map::Tab::new(id, update_state) {
                                Ok(tab) => update_state.edit_tabs.add_tab(tab),
                                Err(e) => luminol_core::error!(
                                    update_state.toasts,
                                    e.wrap_err("Error enumerating maps")
                                ),
                            }
                        }
                    })
            });
        *open = window_open;
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
