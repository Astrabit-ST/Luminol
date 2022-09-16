use ndarray::Axis;

pub struct Map {
    id: i32,
    name: String,
    scale: u8,
    selected_layer: usize,
}

impl Map {
    pub fn new(id: i32, name: String) -> Self {
        Self {
            id,
            name,
            scale: 100,
            selected_layer: 0,
        }
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &crate::UpdateInfo<'_>) {
        // Load the map if it isn't loaded.
        info.data_cache.load_map(info.filesystem, self.id);
        let mut cache = info.data_cache.borrow_mut();
        let mut map = cache.maps.get(&self.id).expect("No map loaded with ID");

        // Display the toolbar.
        egui::TopBottomPanel::top(format!("map_{}_toolbar", self.id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Map {}: {}", self.name, self.id));

                ui.separator();

                ui.add(egui::Slider::new(&mut self.scale, 15..=150).text("Scale"));

                ui.separator();

                // Find the number of layers.
                let layers = map.data.len_of(Axis(0));
                egui::ComboBox::from_label("Layers")
                    // Format the text based on what layer is selected.
                    .selected_text(if self.selected_layer > layers {
                        "Events".to_string()
                    } else {
                        format!("Layer {}", self.selected_layer + 1)
                    })
                    .show_ui(ui, |ui| {
                        // TODO: Add layer enable button
                        // Display all layers.
                        for layer in 0..layers {
                            ui.selectable_value(
                                &mut self.selected_layer,
                                layer,
                                format!("Layer {}", layer + 1),
                            );
                        }
                        // Display event layer.
                        ui.selectable_value(&mut self.selected_layer, layers + 1, "Events");
                    })
            });
        });

        // Display the tilepicker.
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id)).show_inside(ui, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {});
        });

        egui::Frame::canvas(ui.style()).show(ui, |ui| {
            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show_viewport(ui, |ui, rect| {})
        });
    }
}
