pub struct Map {
    id: i32,
    name: String,
}

impl Map {
    pub fn new(id: i32, name: String) -> Self {
        Self { id, name }
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &crate::UpdateInfo<'_>) {
        ui.label(format!("Map number {}", self.id));

        info.data_cache.load_map(info.filesystem, self.id);
    }
}
