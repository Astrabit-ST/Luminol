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

    pub fn ui(&mut self, info: &UpdateInfo, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {}

            if self.project_open_promise.is_none() {
                if ui.button("Open Project").clicked() {
                    let filesystem = info.filesystem.clone();
                    let data_cache = info.data_cache.clone();

                    let promise = Promise::spawn_async(async {
                        pollster::block_on(async {
                            let filesystem = filesystem;
                            let data_cache = data_cache;
                            filesystem.try_open_project(data_cache.as_ref()).await
                        })
                    });

                    self.project_open_promise = Some(promise);
                }
            } else {
                ui.spinner();
            }

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows()
                }

                if ui.button("Save Project").clicked() {
                    info.filesystem.save_cached(info.data_cache.as_ref())
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

        ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
            if ui.button("Maps").clicked() {
                info.windows.add_window(super::map_picker::MapPicker::new())
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows.add_window(super::about::About::new());
            };
        });

        if let Some(p) = &self.project_open_promise {
            if p.ready().is_some() {
                self.project_open_promise = None
            }
        }
    }
}
