use crate::{
    filesystem::{data_cache::DataCache, Filesystem},
    tabs::tab::Tabs,
    top_bar::TopBar,
    windows::window::Windows,
    UpdateInfo,
};

#[derive(Default)]
pub struct App {
    filesystem: Filesystem,
    data_cache: DataCache,
    windows: Windows,
    top_bar: TopBar,
    tabs: Tabs,
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
        eframe::set_value::<Option<()>>(storage, eframe::APP_KEY, &None);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // This struct is passed to windows and widgets so they can modify internal state.
        // Bit jank but it works.
        let update_info = UpdateInfo {
            filesystem: &self.filesystem,
            data_cache: &self.data_cache,
            windows: &self.windows,
            tabs: &self.tabs,
        };

        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(&update_info, ui, frame);
            });
        });

        // Central panel with tabs.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.tabs.ui(ui, &update_info);
        });

        // Update all windows.
        self.windows.update(ctx, &update_info);
    }
}
