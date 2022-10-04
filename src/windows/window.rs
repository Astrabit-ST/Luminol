// Copyright (C) 2022 Lily Lyons
//
// This file is part of Luminol.
//
// Luminol is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Luminol is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Luminol.  If not, see <http://www.gnu.org/licenses/>.

pub use std::cell::RefCell;

use crate::UpdateInfo;

#[derive(Default)]
pub struct Windows {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: RefCell<Vec<Box<dyn Window>>>,
}

impl Windows {
    /// A function to add a window.
    pub fn add_window<T>(&self, window: T)
    where
        T: Window + 'static,
    {
        let mut windows = self.windows.borrow_mut();
        if windows.iter().any(|w| w.name() == window.name()) {
            return;
        }
        windows.push(Box::new(window));
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&self) {
        let mut windows = self.windows.borrow_mut();
        windows.retain(|window| !window.requires_filesystem())
    }

    pub fn update(&self, ctx: &egui::Context, info: &'static UpdateInfo) {
        // Iterate through all the windows and clean them up if necessary.
        let mut windows = self.windows.borrow_mut();
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
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static UpdateInfo);

    /// Required to prevent duplication.
    fn name(&self) -> String;

    ///  A function to determine if this window needs the data cache.
    fn requires_filesystem(&self) -> bool {
        false
    }
}
