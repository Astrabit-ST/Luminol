use crate::{
    filesystem::{data_cache::DataCache, Filesystem},
    gui::window::{UpdateInfo, Windows},
};

pub struct App {
    filesystem: Filesystem,
    data_cache: DataCache,
    windows: Windows,
    top_bar: crate::gui::top_bar::TopBar,
}

impl Default for App {
    fn default() -> Self {
        Self {
            filesystem: Filesystem::new(),
            data_cache: DataCache::new(),
            windows: Windows::new(),
            top_bar: crate::gui::top_bar::TopBar::new(),
        }
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
        eframe::set_value::<Option<()>>(storage, eframe::APP_KEY, &None);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let update_info = UpdateInfo {
            filesystem: &self.filesystem,
            data_cache: &self.data_cache,
            windows: &self.windows,
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

        egui::CentralPanel::default().show(ctx, |_ui| {});

        self.windows.update(ctx, &update_info);
    }
}
