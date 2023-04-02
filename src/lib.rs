//! Luminol is a supercharged FOSS version of the RPG Maker XP editor.
//!
//! Authors:
//!     Lily Madeline Lyons <lily@nowaffles.com>
//!
// #![warn(missing_docs)]

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

#![warn(rust_2018_idioms)]
#![warn(
    clippy::all,
    // clippy::pedantic,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap
)]
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
#![deny(unsafe_code)]
#![feature(drain_filter, is_some_and, min_specialization)]

pub use prelude::*;

/// The main Luminol application.
pub mod luminol;

pub mod prelude;
/// The state Luminol saves on shutdown.
pub mod saved_state;

/// Audio related structs and funtions.
pub mod audio;

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

cfg_if::cfg_if! {
    if #[cfg(not(target_arch = "wasm32"))] {
        type FSAlias =  filesystem::filesystem_native::Filesystem;
    } else {
        type FSAlias = filesystem::filesystem_wasm32::Filesystem;
    }
}

/// Passed to windows and widgets when updating.
pub struct UpdateInfo {
    /// Filesystem to be passed around.
    pub filesystem: FSAlias,
    /// The data cache.
    pub data_cache: Cache,
    /// Windows that are displayed.
    pub windows: window::Windows,
    /// Tabs that are displayed.
    pub tabs: tabs::tab::Tabs<Box<dyn Tab>>,
    /// Audio that's played.
    pub audio: audio::Audio,
    /// Toasts to be displayed.
    pub toasts: Toasts,
    /// The gl context.
    pub gl: Arc<glow::Context>,
    /// State to be saved.
    pub saved_state: RefCell<SavedState>,
    /// Toolbar state
    pub toolbar: RefCell<ToolbarState>,
}

impl UpdateInfo {
    /// Create a new UpdateInfo.
    pub fn new(gl: Arc<glow::Context>, state: SavedState) -> Self {
        Self {
            filesystem: FSAlias::default(),
            data_cache: Cache::default(),
            windows: windows::window::Windows::default(),
            tabs: tab::Tabs::new("global_tabs", vec![Box::new(started::Tab::new())]),
            audio: audio::Audio::default(),
            toasts: Toasts::default(),
            gl,
            saved_state: RefCell::new(state),
            toolbar: RefCell::default(),
        }
    }
}

/// Load a RetainedImage from disk.
pub async fn load_image_software(
    path: String,
    info: &'static UpdateInfo,
) -> Result<RetainedImage, String> {
    egui_extras::RetainedImage::from_image_bytes(
        path.clone(),
        &info.filesystem.read_bytes(&format!("{path}.png",)).await?,
    )
    .map(|i| i.with_options(TextureOptions::NEAREST))
}

/// Load a gl texture from disk.
#[allow(clippy::cast_possible_wrap, unsafe_code)]
pub async fn load_image_hardware(
    path: String,
    info: &'static UpdateInfo,
) -> Result<glow::Texture, String> {
    use glow::HasContext;

    let image =
        image::load_from_memory(&info.filesystem.read_bytes(&format!("{path}.png",)).await?)
            .map_err(|e| e.to_string())?;

    unsafe {
        let texture = info.gl.create_texture()?;
        info.gl.bind_texture(glow::TEXTURE_2D, Some(texture));

        info.gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA as _,
            image.width() as _,
            image.height() as _,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            Some(image.as_bytes()),
        );
        info.gl.generate_mipmap(glow::TEXTURE_2D);

        Ok(texture)
    }
}
