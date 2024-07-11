// Copyright (C) 2024 Melody Madeline Lyons
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
#[derive(Default)]
pub struct Windows {
    // A dynamic array of Windows. Iterated over and cleaned up in fn update().
    windows: Vec<Box<dyn Window>>,
}

#[derive(Default)]
pub struct EditWindows {
    clean_fn: Option<CleanFn>,
    added: Vec<Box<dyn Window>>,
    removed: std::collections::HashSet<egui::Id>,
}

type CleanFn = Box<dyn Fn(&Box<dyn Window>) -> bool>;

impl Windows {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_with_windows(windows: Vec<impl Window + 'static>) -> Self {
        Self {
            windows: windows.into_iter().map(|w| Box::new(w) as Box<_>).collect(),
        }
    }

    /// A function to add a window.
    pub fn add_window(&mut self, window: impl Window + 'static) {
        self.add_boxed_window(Box::new(window))
    }

    fn add_boxed_window(&mut self, window: Box<dyn Window>) {
        // FIXME use a hashmap, or something with less than O(n) search time
        if self.windows.iter().any(|w| w.id() == window.id()) {
            return;
        }
        self.windows.push(window)
    }

    /// Clean all windows that need the data cache.
    /// This is usually when a project is closed.
    pub fn clean_windows(&mut self, f: impl Fn(&Box<dyn Window>) -> bool) {
        self.windows.retain(f);
    }

    pub fn process_edit_windows(&mut self, mut edit_windows: EditWindows) {
        for window in edit_windows.added.drain(..) {
            self.add_boxed_window(window)
        }
        if let Some(f) = edit_windows.clean_fn.take() {
            self.clean_windows(f);
        }
    }

    pub fn display_without_edit(
        &mut self,
        ctx: &egui::Context,
        update_state: &mut crate::UpdateState<'_>,
    ) {
        // Iterate through all the windows and clean them up if necessary.
        self.windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open, update_state);
            open
        })
    }

    /// Update and draw all windows.
    pub fn display(&mut self, ctx: &egui::Context, update_state: &mut crate::UpdateState<'_>) {
        let mut edit_windows = EditWindows {
            clean_fn: None,
            added: Vec::new(),
            removed: std::collections::HashSet::new(),
        };
        let mut reborrowed_update_state = update_state.reborrow_with_edit_window(&mut edit_windows);

        // Iterate through all the windows and clean them up if necessary.
        self.windows.retain_mut(|window| {
            // Pass in a bool requesting to see if the window open.
            let mut open = true;
            window.show(ctx, &mut open, &mut reborrowed_update_state);
            open
        });

        for window in edit_windows.added {
            if self.windows.iter().any(|w| w.id() == window.id()) {
                return;
            }
            self.windows.push(window);
        }
        if let Some(f) = edit_windows.clean_fn {
            self.clean_windows(f)
        }
    }
}

impl EditWindows {
    pub fn clean(&mut self, f: impl Fn(&Box<dyn Window>) -> bool + 'static) {
        self.clean_fn = Some(Box::new(f));
    }

    pub fn add_window(&mut self, window: impl Window + 'static) {
        self.added.push(Box::new(window))
    }

    pub fn remove_window(&mut self, window: &impl Window) -> bool {
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
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut crate::UpdateState<'_>,
    );

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

impl From<Vec<Box<dyn Window>>> for Windows {
    fn from(windows: Vec<Box<dyn Window>>) -> Self {
        Self { windows }
    }
}
