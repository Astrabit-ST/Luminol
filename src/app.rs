use std::{cell::RefCell, rc::Rc};
use crate::{gui::window::{Windows, UpdateInfo}, filesystem::Filesystem};

pub struct App {
    filesystem: Rc<RefCell<Filesystem>>,
    windows: Rc<RefCell<Windows>>,
    top_bar: crate::gui::top_bar::TopBar
}

impl Default for App {
    fn default() -> Self {
        Self {
            filesystem: Rc::new(RefCell::new(Filesystem::new())),
            windows: Rc::new(RefCell::new(Windows::new())),
            top_bar: crate::gui::top_bar::TopBar::new()
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
        eframe::set_value::<Option<()>>(storage, eframe::APP_KEY,&None);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            let mut update_info = UpdateInfo {
                filesystem: self.filesystem.clone(),
                windows: self.windows.clone()
            };

            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(&mut update_info, ui, frame);
            });

            self.windows.borrow_mut().update(ctx, &mut update_info);
        });
    }
}
