use super::window::UpdateInfo;
use poll_promise::Promise;

pub struct TopBar {
    project_open_promise: Option<Promise<()>>,
}

impl TopBar {
    pub fn new() -> Self {
        Self {
            project_open_promise: None,
        }
    }

    pub fn ui(&mut self, info: &mut UpdateInfo, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let mut filesystem = info.filesystem.borrow_mut();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {}

            if self.project_open_promise.is_none() {
                if ui.button("Open Project").clicked() {}
            } else {
                ui.spinner();
            }

            ui.separator();

            ui.add_enabled_ui(filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    filesystem.unload_project();
                    info.windows.borrow_mut().clean_windows()
                }

                if ui.button("Save Project").clicked() {
                    filesystem.save_cached()
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

        ui.add_enabled_ui(filesystem.project_loaded(), |ui| {
            if ui.button("Maps").clicked() {
                info.windows
                    .borrow_mut()
                    .add_window(super::map_picker::MapPicker::new())
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows
                    .borrow_mut()
                    .add_window(super::about::About::new());
            };
        });
    }
}
