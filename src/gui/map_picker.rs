use crate::data::rmxp_structs::rpg::MapInfo;

/// The map picker window.
/// Displays a list of maps in a tree.
/// Maps can be double clicked to open them in a map editor.
pub struct MapPicker {}

impl MapPicker {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::window::Window for MapPicker {
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data_cache: Option<&mut crate::filesystem::data_cache::DataCache>,
    ) {
        egui::Window::new("Map Picker").open(open).show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mapinfos = &mut data_cache.expect("Data Cache not loaded").mapinfos;
                let mut vec = Vec::from_iter(mapinfos.iter());

                vec.sort_by(|a, b| a.1.order.cmp(&b.1.order));

                ui.collapsing("root", |ui| {
                    let mut map_stack = vec![0];
                    let mut ui_stack = vec![ui];

                    for (id, map) in vec {
                        while map_stack.len() > 0
                            && id
                                != map_stack
                                    .last()
                                    .expect("There should be at least 1 element")
                        {
                            map_stack.pop();
                        }
                        map_stack.push(*id);
                        egui::CollapsingHeader::new(&map.name)
                            .id_source(format!("mapinfo_{}_{}", &map.name, id))
                            .show(ui_stack.last_mut().unwrap(), |ui| {});
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
