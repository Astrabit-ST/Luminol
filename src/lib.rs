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
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unnecessary_wraps,
    // unsafe code is sometimes fine but in general we don't want to use it.
    unsafe_code,
)]
// These may be turned on in the future.
// #![warn(clippy::unwrap, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
// You must provide a safety doc. DO NOT TURN OFF THESE LINTS.
#![forbid(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
// Okay, lemme run through *why* some of these are enabled
// 1) min_specialization
// min_specialization is used in alox-48 to deserialize extra data types.
// 2) int_roundings
// int_roundings is close to stabilization.
#![feature(min_specialization, int_roundings)]

#[cfg(not(target_arch = "wasm32"))]
/// Whether or not to use push constants when rendering the map editor. Disabling this will switch
/// to fallback rendering using uniforms, which is slightly slower but is required for Luminol to
/// work in web browsers until push constants are standardized in WebGPU.
pub const USE_PUSH_CONSTANTS: bool = true;
#[cfg(target_arch = "wasm32")]
/// Whether or not to use push constants when rendering the map editor. Disabling this will switch
/// to fallback rendering using uniforms, which is slightly slower but is required for Luminol to
/// work in web browsers until push constants are standardized in WebGPU.
pub const USE_PUSH_CONSTANTS: bool = false;

pub use prelude::*;

/// The main Luminol application.
pub mod luminol;

pub mod prelude;

/// Audio related structs and funtions.
#[cfg(not(target_arch = "wasm32"))]
pub mod audio;

pub mod config;

pub mod cache;

pub mod components;

pub mod command_gen;

/// Floating windows to be displayed anywhere.
pub mod windows;

/// Stack defined windows that edit values.
pub mod modals;

/// Tabs to be displayed in the center of Luminol.
pub mod tabs;

/// Filesystem related structs.
/// Swaps between filesystem_native and filesystem_wasm depending on the target arch.
pub mod filesystem;

/// The code for handling lumi, the friendly world machine!
pub mod lumi;

/// Utilities specific to WebAssembly builds of Luminol.
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(feature = "steamworks")]
pub mod steam;

pub mod graphics;

pub use luminol::Luminol;
use tabs::tab::Tab;

#[cfg(target_arch = "wasm32")]
pub struct GlobalState {
    pub device_pixel_ratio: f32,
    pub prefers_color_scheme_dark: Option<bool>,
    pub filesystem_tx: mpsc::UnboundedSender<filesystem::web::FileSystemCommand>,
}

#[cfg(target_arch = "wasm32")]
pub static GLOBAL_STATE: once_cell::sync::OnceCell<GlobalState> = once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
pub struct GlobalCallbackState {
    pub screen_resize_tx: mpsc::UnboundedSender<(u32, u32)>,
    pub event_tx: mpsc::UnboundedSender<egui::Event>,
}

#[cfg(target_arch = "wasm32")]
pub static GLOBAL_CALLBACK_STATE: once_cell::sync::OnceCell<GlobalCallbackState> =
    once_cell::sync::OnceCell::new();

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
    pub filesystem: filesystem::project::FileSystem,
    /// The data cache.
    pub data_cache: data::Cache,
    pub image_cache: image_cache::Cache,
    pub atlas_cache: atlas::Cache,
    /// Windows that are displayed.
    pub windows: window::Windows,
    /// Tabs that are displayed.
    pub tabs: tabs::tab::Tabs<Box<dyn Tab + Send>>,
    /// Audio that's played.
    #[cfg(not(target_arch = "wasm32"))]
    pub audio: audio::Audio,
    /// Toasts to be displayed.
    pub toasts: Toasts,
    pub render_state: egui_wgpu::RenderState,
    /// Toolbar state
    pub toolbar: AtomicRefCell<ToolbarState>,
}

static_assertions::assert_impl_all!(State: Send, Sync);

impl State {
    /// Create a new UpdateInfo.
    pub fn new(render_state: egui_wgpu::RenderState) -> Self {
        Self {
            filesystem: filesystem::project::FileSystem::default(),
            data_cache: data::Cache::default(),
            image_cache: image_cache::Cache::default(),
            atlas_cache: atlas::Cache::default(),
            windows: windows::window::Windows::default(),
            tabs: tab::Tabs::new("global_tabs", vec![Box::new(started::Tab::new())]),
            #[cfg(not(target_arch = "wasm32"))]
            audio: audio::Audio::default(),
            toasts: Toasts::default(),
            render_state,
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
