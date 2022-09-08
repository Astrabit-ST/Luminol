use std::{sync::{Mutex, Arc}};
pub use std::cell::RefCell;

use crate::filesystem::{Filesystem, data_cache::DataCache};
/// Passed to windows and widgets when updating.
pub struct UpdateInfo {
    pub filesystem: Arc<Filesystem>,
    pub data_cache: Arc<DataCache>,
    pub windows: Arc<Windows>,
}

pub struct Windows {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Mutex<RefCell<Vec<Box<dyn Window + Send>>>>,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            windows: Mutex::new(RefCell::new(Vec::new())),
        }
    }

    /// A function to add a window.
    pub fn add_window<T>(&self, window: T)
    where
        T: Window + Send + 'static,
    {
        let windows = self.windows.lock().unwrap();
        let mut windows = windows.borrow_mut();
        if windows.iter().any(|w| w.name() == window.name()) {
            return;
        }
        windows.push(Box::new(window));
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&self) {
        let mut windows = self.windows.lock().unwrap();
        let windows = windows.get_mut();
        windows.retain(|window| !window.requires_cache())
    }

    pub fn update(&self, ctx: &egui::Context, info: &UpdateInfo) {
        // Iterate through all the windows and clean them up if necessary.
        let mut windows = self.windows.lock().unwrap();
        let windows = windows.get_mut();
        windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open, info);
            open
        });
    }
}

/// A basic trait describing a window that can show itself.
/// A mutable bool is passed to it and is set to false if it is closed.
pub trait Window {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &UpdateInfo);

    /// Required to prevent duplication.
    fn name(&self) -> String;

    ///  A function to determine if this window needs the data cache.
    fn requires_cache(&self) -> bool {
        false
    }
}
