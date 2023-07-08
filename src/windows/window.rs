// Copyright (C) 2023 Lily Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use parking_lot::Mutex;

use crate::fl;

/// A window management system to handle heap allocated windows
///
/// Will deny any duplicated window titles and is not specialized like modals
#[derive(Default)]
pub struct Windows {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Mutex<Vec<Box<dyn Window + Send>>>,
}

impl Windows {
    /// A function to add a window.
    pub fn add_window<T>(&self, window: T)
    where
        T: Window + Send + 'static,
    {
        let mut windows = self.windows.lock();
        if windows.iter().any(|w| w.id() == window.id()) {
            return;
        }
        windows.push(Box::new(window));
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&self) {
        let mut windows = self.windows.lock();
        windows.retain(|window| !window.requires_filesystem());
    }

    /// Update and draw all windows.
    pub fn update(&self, ctx: &egui::Context) {
        // Iterate through all the windows and clean them up if necessary.
        let mut windows = self.windows.lock();
        windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open);
            open
        });
    }
}

/// A heap allocated window, unlike modals which are stored on the stack.
/// This makes them very unspecialized and they rely heavily on `UpdateInfo`.
///
/// However, they can store internal state and allow for multiple windows to be open at one time.
/// This makes up for most of their losses. Modals are still useful in certain cases, though.
pub trait Window {
    /// Show this window.
    fn show(&mut self, ctx: &egui::Context, open: &mut bool);

    /// Optionally used as the title of the window.
    fn name(&self) -> String {
        fl!("window_untitled_title")
    }

    /// Required to prevent duplication.
    fn id(&self) -> egui::Id;

    ///  A function to determine if this window needs the data cache.
    fn requires_filesystem(&self) -> bool {
        false
    }
}
