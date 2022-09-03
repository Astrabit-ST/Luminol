/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {}

impl Default for App {
    fn default() -> Self {
        Self {}
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("New Project...").clicked() {
                    
                }
                if ui.button("Open Project").clicked() {

                }
                if ui.button("Close Project").clicked() {

                }
                if ui.button("Save Project").clicked() {

                }
                ui.separator();
                if ui.button("Compress Game Data...").clicked() {

                }
                ui.separator();
                if ui.button("Exit Luminol").clicked() {
                    frame.close();
                }
            });
        });

        egui::Window::new("Test").show(ctx, |ui| {
            ui.button("test buton!")
        });
    }
}
