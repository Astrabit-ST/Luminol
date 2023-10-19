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

/// A window management system to handle heap allocated windows
///
/// Will deny any duplicated window titles and is not specialized like modals

pub struct Windows<W> {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Vec<W>,
}

pub struct EditWindows<W> {
    clean_fn: Option<CleanFn<W>>,
    added: Vec<W>,
    removed: std::collections::HashSet<egui::Id>,
}

type CleanFn<T> = Box<dyn Fn(&T) -> bool>;

impl<W> Default for Windows<W> {
    fn default() -> Self {
        Self {
            windows: Vec::new(),
        }
    }
}

impl<W> Default for EditWindows<W> {
    fn default() -> Self {
        Self {
            clean_fn: None,
            added: Vec::new(),
            removed: std::collections::HashSet::new(),
        }
    }
}

impl<W> Windows<W>
where
    W: Window,
{
    /// A function to add a window.
    pub fn add_window(&mut self, window: W) {
        // FIXME use a hashmap, or something with less than O(n) search time
        if self.windows.iter().any(|w| w.id() == window.id()) {
            return;
        }
        self.windows.push(window);
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&mut self, f: impl Fn(&W) -> bool) {
        self.windows.retain(f);
    }

    /// Update and draw all windows.
    pub fn display<O, T>(
        &mut self,
        ctx: &egui::Context,
        update_state: &mut crate::UpdateState<'_, O, T>,
    ) {
        let mut edit_windows = EditWindows::<W> {
            clean_fn: None,
            added: Vec::new(),
            removed: std::collections::HashSet::new(),
        };
        let mut update_state = update_state.reborrow_with_edit_window(&mut edit_windows);

        // Iterate through all the windows and clean them up if necessary.
        self.windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open, &mut update_state);
            open
        });

        for window in edit_windows.added {
            self.add_window(window);
        }
        if let Some(f) = edit_windows.clean_fn {
            self.clean_windows(f)
        }
    }
}

impl<T> EditWindows<T>
where
    T: Window,
{
    pub fn clean(&mut self, f: impl Fn(&T) -> bool + 'static) {
        self.clean_fn = Some(Box::new(f));
    }

    pub fn add_window(&mut self, window: T) {
        self.added.push(window)
    }

    pub fn remove_window(&mut self, window: &T) -> bool {
        self.remove_window_by_id(window.id())
    }

    pub fn remove_window_by_id(&mut self, id: egui::Id) -> bool {
        self.removed.insert(id)
    }
}

/// A heap allocated window, unlike modals which are stored on the stack.
/// This makes them very unspecialized and they rely heavily on `UpdateInfo`.
///
/// However, they can store internal state and allow for multiple windows to be open at one time.
/// This makes up for most of their losses. Modals are still useful in certain cases, though.
pub trait Window {
    /// Show this window.
    fn show<W, T>(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut crate::UpdateState<'_, W, T>,
    );

    /// Optionally used as the title of the window.
    fn name(&self) -> String {
        "Untitled Window".to_string()
    }

    /// Required to prevent duplication.
    fn id(&self) -> egui::Id;

    ///  A function to determine if this window needs the data cache.
    fn requires_filesystem(&self) -> bool {
        false
    }
}

/*
impl Window for Box<dyn Window + Send + Sync> {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        self.show(ctx, open)
    }

    fn name(&self) -> String {
        self.name()
    }

    fn id(&self) -> egui::Id {
        self.id()
    }

    fn requires_filesystem(&self) -> bool {
        self.requires_filesystem()
    }
}
*/
