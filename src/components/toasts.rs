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

use egui_notify::{Toasts as ToastsInner, Toast};
use std::cell::RefCell;

#[derive(Default)]
pub struct Toasts {
    inner: RefCell<ToastsInner>
}

// We wrap the toasts structs in a RefCell to maintain interior mutability.
#[allow(dead_code)]
impl Toasts {
    pub fn add(&self, toast: Toast) {
        self.inner.borrow_mut().add(toast);
    }

    pub fn info(&self, caption: impl Into<String>) {
        self.inner.borrow_mut().info(caption);
    }

    pub fn warning(&self, caption: impl Into<String>) {
        self.inner.borrow_mut().warning(caption);
    }

    pub fn error(&self, caption: impl Into<String>) {
        self.inner.borrow_mut().error(caption);
    }

    pub fn basic(&self, caption: impl Into<String>) {
        self.inner.borrow_mut().basic(caption);
    }

    pub fn show(&self, ctx: &egui::Context) {
        self.inner.borrow_mut().show(ctx);
    }
}