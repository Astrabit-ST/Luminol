use crate::components::tilemap::Tilemap;

pub struct Map {
    pub id: i32,
    pub name: String,
    pub selected_layer: usize,
    pub tilemap: Tilemap,
}

impl Map {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            selected_layer: 0,
            tilemap: Tilemap::new(),
        }
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    #[allow(unused_variables, unused_mut)]
    fn show(&mut self, ui: &mut egui::Ui, info: &crate::UpdateInfo<'_>) {
        // Load the map if it isn't loaded.
        info.data_cache.load_map(info.filesystem, self.id);
        let mut cache = info.data_cache.borrow_mut();
        let mut map = cache.maps.get_mut(&self.id).expect("No map loaded with ID");

        // Display the toolbar.
        self.toolbar(ui, map);

        // Display the tilepicker.
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id)).show_inside(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {});
        });

        egui::CentralPanel::default().show_inside(ui, |ui| self.tilemap.ui(ui, map));
    }
}
