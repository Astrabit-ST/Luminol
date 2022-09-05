use crate::gui::window::Window;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    pub filesystem: crate::filesystem::Filesystem,
    #[serde(skip)]
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Vec<Box<dyn Window>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            filesystem: crate::filesystem::Filesystem::new(),
            windows: Vec::new(),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    /// A function to add a window.
    pub fn add_window<T>(&mut self, window: T)
    where
        T: Window + 'static,
    {
        if self.windows.iter().any(|w| w.name() == window.name()) {
            return;
        }
        self.windows.push(Box::new(window));
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&mut self) {
        self.windows.retain(|window| !window.requires_cache())
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar(ui, frame)
            })
        });

        // Iterate through all the windows and clean them up if necessary.
        self.windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open, self.filesystem.data_cache());
            open
        })
    }
}
