use crate::app::App;

impl App {
    pub fn top_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = self.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {}

            if ui.button("Open Project").clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                self.filesystem.try_open_project()
            }

            ui.separator();

            ui.add_enabled_ui(self.filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    self.filesystem.unload_project();
                    self.clean_windows()
                }

                if ui.button("Save Project").clicked() {
                    self.filesystem.save_cached()
                }
            });

            #[cfg(not(target_arch = "wasm32"))]
            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Exit Luminol").clicked() {
                frame.close()
            }
        });

        ui.separator();

        ui.add_enabled_ui(self.filesystem.project_loaded(), |ui| {
            if ui.button("Maps").clicked() {
                self.add_window(super::map_picker::MapPicker::new())
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                self.add_window(super::about::About::new());
            };
        });
    }
}
