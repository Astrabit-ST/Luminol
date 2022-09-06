pub use std::{cell::RefCell, rc::Rc};

use crate::filesystem::Filesystem;
/// Passed to windows and widgets when updating.
pub struct UpdateInfo {
    pub filesystem: Rc<RefCell<Filesystem>>,
    pub windows: Rc<RefCell<Windows>>,
}

pub struct Windows {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Vec<Box<dyn Window>>,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
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

    pub fn update(&mut self, ctx: &egui::Context, info: &mut UpdateInfo) {
        // Iterate through all the windows and clean them up if necessary.
        self.windows.retain_mut(|window| {
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
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &mut UpdateInfo);

    /// Required to prevent duplication.
    fn name(&self) -> String;

    ///  A function to determine if this window needs the data cache.
    fn requires_cache(&self) -> bool {
        false
    }
}
