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

use egui_notify::{Toast, Toasts as ToastsInner};
use parking_lot::RwLock;

/// A toasts management struct.
#[derive(Default)]
pub struct Toasts {
    inner: RwLock<ToastsInner>,
}

// We wrap the toasts structs in a RefCell to maintain interior mutability.
#[allow(dead_code)]
impl Toasts {
    /// Add a custom toast.
    pub fn add(&self, toast: Toast) {
        self.inner.write().add(toast);
    }

    /// Display an info toast.
    pub fn info(&self, caption: impl Into<String>) {
        self.inner.write().info(caption);
    }

    /// Display a warning toast.
    pub fn warning(&self, caption: impl Into<String>) {
        self.inner.write().warning(caption);
    }

    /// Display an error toast.
    pub fn error(&self, caption: impl Into<String>) {
        self.inner.write().error(caption);
    }

    /// Display a generic toast.
    pub fn basic(&self, caption: impl Into<String>) {
        self.inner.write().basic(caption);
    }

    /// Display all toasts.
    pub fn show(&self, ctx: &egui::Context) {
        self.inner.write().show(ctx);
    }
}
