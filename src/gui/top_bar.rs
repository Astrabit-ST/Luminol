use eframe::epaint::ahash::HashMap;

use crate::app::App;

impl App {
    pub fn top_bar(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = &self.filesystem.project_path {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {}

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Open Project").clicked() {
                if let Some(mut path) = rfd::FileDialog::default()
                    .add_filter("project file", &["rxproj", "lum"])
                    .pick_file()
                {
                    path.pop(); // Pop off filename
                    self.filesystem.project_path = Some(path);
                }
            }

            if ui.button("Close Project").clicked() {
                self.filesystem.project_path = None
            }

            if ui.button("Save Project").clicked() {}

            #[cfg(not(target_arch = "wasm32"))]
            ui.separator();

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Exit Luminol").clicked() {
                frame.close()
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                self.add_window(Box::new(super::about::About::new()));
            };
        });
    }
}
