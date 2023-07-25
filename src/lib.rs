//! Luminol is a supercharged FOSS version of the RPG Maker XP editor.
//!
//! Authors:
//!     Lily Madeline Lyons <lily@nowaffles.com>
//!
// #![warn(missing_docs)]

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

#![warn(rust_2018_idioms)]
#![warn(
    clippy::all,
    // clippy::pedantic,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    // clippy::unwrap_used,
    clippy::unnecessary_wraps
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
#![deny(unsafe_code)]
#![feature(drain_filter, min_specialization)]

pub use prelude::*;

/// The main Luminol application.
pub mod luminol;

pub mod prelude;
/// The state Luminol saves on shutdown.
pub mod saved_state;

/// Audio related structs and funtions.
pub mod audio;

pub mod cache;

pub mod components;

pub mod command_gen;

/// Floating windows to be displayed anywhere.
pub mod windows;

/// Stack defined windows that edit values.
pub mod modals;

/// Structs related to Luminol's internal data.
pub mod project;
/// Tabs to be displayed in the center of Luminol.
pub mod tabs;

/// Filesystem related structs.
/// Swaps between filesystem_native and filesystem_wasm depending on the target arch.
pub mod filesystem;

/// The code for handling lumi, the friendly world machine!
pub mod lumi;

#[cfg(feature = "steamworks")]
pub mod steam;

pub use luminol::Luminol;
use saved_state::SavedState;
use tabs::tab::Tab;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

#[allow(missing_docs)]
#[derive(Default)]
pub struct ToolbarState {
    /// The currently selected pencil.
    pub pencil: Pencil,
}

#[derive(Default, strum::EnumIter, strum::Display, PartialEq, Eq, Clone, Copy)]
#[allow(missing_docs)]
pub enum Pencil {
    #[default]
    Pen,
    Circle,
    Rectangle,
    Fill,
}

/// Passed to windows and widgets when updating.
pub struct State {
    /// Filesystem to be passed around.
    pub filesystem: filesystem::Filesystem,
    /// The data cache.
    pub data_cache: data::Cache,
    pub image_cache: image_cache::Cache,
    /// Windows that are displayed.
    pub windows: window::Windows,
    /// Tabs that are displayed.
    pub tabs: tabs::tab::Tabs<Box<dyn Tab + Send>>,
    /// Audio that's played.
    pub audio: audio::Audio,
    /// Toasts to be displayed.
    pub toasts: Toasts,
    /// The gl context.
    pub gl: Arc<glow::Context>,
    /// State to be saved.
    pub saved_state: AtomicRefCell<SavedState>,
    /// Toolbar state
    pub toolbar: AtomicRefCell<ToolbarState>,
}

static_assertions::assert_impl_all!(State: Send, Sync);

impl State {
    /// Create a new UpdateInfo.
    pub fn new(gl: Arc<glow::Context>, state: SavedState) -> Self {
        Self {
            filesystem: filesystem::Filesystem::default(),
            data_cache: data::Cache::default(),
            image_cache: image_cache::Cache::default(),
            windows: windows::window::Windows::default(),
            tabs: tab::Tabs::new("global_tabs", vec![Box::new(started::Tab::new())]),
            audio: audio::Audio::default(),
            toasts: Toasts::default(),
            gl,
            saved_state: AtomicRefCell::new(state),
            toolbar: AtomicRefCell::default(),
        }
    }
}

static STATE: once_cell::sync::OnceCell<State> = once_cell::sync::OnceCell::new();

#[allow(clippy::panic)]
fn set_state(info: State) {
    if STATE.set(info).is_err() {
        panic!("failed to set updateinfo")
    }
}

#[macro_export]
macro_rules! state {
    () => {
        $crate::STATE.get().expect("failed to get updateinfo")
    };
}
