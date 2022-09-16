use crate::UpdateInfo;
#[derive(Default)]
pub struct TopBar {}

impl TopBar {
    #[allow(unused_variables)]
    pub fn ui(&mut self, info: &UpdateInfo<'_>, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        egui::widgets::global_dark_light_mode_switch(ui);

        ui.separator();

        ui.menu_button("File", |ui| {
            ui.label(if let Some(path) = info.filesystem.project_path() {
                format!("Current project:\n{}", path.display())
            } else {
                "No project open".to_string()
            });

            if ui.button("New Project").clicked() {
                todo!()
            }

            if ui.button("Open Project").clicked() {
                info.filesystem.try_open_project(info.data_cache);
            }

            ui.separator();

            ui.add_enabled_ui(info.filesystem.project_loaded(), |ui| {
                if ui.button("Close Project").clicked() {
                    info.filesystem.unload_project();
                    info.windows.clean_windows();
                    info.tabs.clean_tabs();
                }

                if ui.button("Save Project").clicked() {
                    info.filesystem.save_cached(info.data_cache)
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
                info.windows
                    .add_window(crate::windows::map_picker::MapPicker::new())
            }

            if ui.button("Sound Test").clicked() {
                info.windows
                    .add_window(crate::windows::sound_test::SoundTest::new())
            }
        });

        ui.separator();

        ui.menu_button("Help", |ui| {
            if ui.button("About...").clicked() {
                info.windows.add_window(crate::windows::about::About::new());
            };
        });
    }
}
