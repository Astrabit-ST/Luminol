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

                for ele in mapinfos.iter() {
                    ui.label(&ele.1.name);
                }
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
