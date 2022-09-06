use crate::data::rmxp_structs::rpg::MapInfo;
use std::collections::HashMap;
use super::window::UpdateInfo;

/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
pub struct MapPicker {}

impl MapPicker {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render_submap(
        &self,
        id: &i32,
        children_data: &HashMap<i32, Vec<i32>>,
        mapinfos: &HashMap<i32, MapInfo>,
        ui: &mut egui::Ui,
    ) {
        // We get the map name. It's assumed that there is in fact a map with this ID in mapinfos.
        let map_name = &mapinfos.get(id).unwrap().name;
        // Does this map have children?
        if children_data.contains_key(id) {
            // Render a collapsing header.
            ui.collapsing(map_name, |ui| {
                for id in children_data.get(id).unwrap() {
                    // Render children.
                    self.render_submap(id, children_data, mapinfos, ui)
                }
            });
        } else {
            // Just display a label otherwise.
            ui.label(map_name);
        }
    }
}

impl super::window::Window for MapPicker {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &mut UpdateInfo) {
        egui::Window::new("Map Picker").open(open).show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mut filesystem = info.filesystem.borrow_mut();
                let mapinfos = &mut filesystem.data_cache().expect("Data Cache not loaded").mapinfos;
                let mut sorted_maps = Vec::from_iter(mapinfos.iter());

                // We sort maps by their order.
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
                        self.render_submap(id, &children_data, mapinfos, ui);
                    }
                });
            })
        });
    }

    fn name(&self) -> String {
        "Map Picker".to_string()
    }

    fn requires_cache(&self) -> bool {
        true
    }
}
