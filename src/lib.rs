//! Luminol is a supercharged FOSS version of the RPG Maker XP editor.
//!
//! Authors:
//!     Lily Madeline Lyons <lily@nowaffles.com>
//!     Egor Poleshko <somedevfox@gmail.com>
//!

#![warn(clippy::all, rust_2018_idioms)]
#![warn(missing_docs)]
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
#![feature(drain_filter, is_some_and, min_specialization)]

/// The main Luminol application.
pub mod luminol;

/// The state Luminol saves on shutdown.
pub mod saved_state;

/// Audio related structs and funtions.
pub mod audio;

/// Floating windows to be displayed anywhere.
pub mod windows {
    /// The about window.
    pub mod about;
    /// The common event editor.
    pub mod common_event_edit;
    /// Config window
    pub mod config;
    /// The event editor.
    pub mod event_edit;
    /// The item editor.
    pub mod items;
    /// The map picker.
    pub mod map_picker;
    /// Misc windows.
    pub mod misc;
    /// New project window
    pub mod new_project;
    /// The script editor
    pub mod script_edit;
    /// The sound test.
    pub mod sound_test;
    /// Traits and structs related to windows.
    pub mod window;
}

/// Stack defined windows that edit values.
pub mod modals {
    /// Traits related to modals.
    pub mod modal;
    /// The switch picker.
    pub mod switch;
    /// The variable picker.
    pub mod variable;
}

/// Various UI components used throughout Luminol.
pub mod components {
    /// Command editor for events
    pub mod command_view;
    /// Command view modals
    pub mod command_view_modals;
    /// Move route display
    pub mod move_display;
    /// Syntax highlighter
    pub mod syntax_highlighting;
    /// Toasts to be displayed for errors, information, etc.
    pub mod toasts;
    /// The toolbar for managing the project.
    pub mod top_bar;

    /// The tilemap.
    pub mod tilemap {
        use crate::{data::rmxp_structs::rpg, UpdateInfo};

        cfg_if::cfg_if! {
            if #[cfg(feature = "generic-tilemap")] {
                mod generic_tilemap;
                pub use generic_tilemap::Tilemap;
            } else {
                mod hardware_tilemap;
                pub use hardware_tilemap::Tilemap;
            }
        }

        /// A trait defining how a tilemap should function.
        pub trait TilemapDef {
            /// Create a new tilemap.
            fn new(info: &'static UpdateInfo, id: i32) -> Self;

            /// Display the tilemap.
            fn ui(
                &mut self,
                ui: &mut egui::Ui,
                map: &rpg::Map,
                cursor_pos: &mut egui::Pos2,
                toggled_layers: &[bool],
                selected_layer: usize,
                dragging_event: bool,
            ) -> egui::Response;

            /// Display the tile picker.
            fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16);

            /// Check if the textures are loaded yet.
            fn textures_loaded(&self) -> bool;

            /// Return the result of loading the tilemap.
            fn load_result(&self) -> Result<(), String>;
        }
    }
}

/// Tabs to be displayed in the center of Luminol.
pub mod tabs {
    /// The map editor.
    pub mod map;
    /// The getting started screen.
    pub mod started;
    /// Traits and structs related to tabs.
    pub mod tab;
}

/// Structs related to Luminol's internal data.
pub mod data {
    /// The tree data structure for commands
    pub mod command_tree;
    /// Event command related enums
    pub mod commands;
    /// Luminol configuration
    pub mod config;
    /// The data cache, used to store things before writing them to the disk.
    pub mod data_cache;
    /// RGSS structs.
    pub mod rgss_structs;
    /// RMXP structs.
    pub mod rmxp_structs;

    pub mod nil_padded;
}

/// Filesystem related structs.
/// Swaps between filesystem_native and filesystem_wasm depending on the target arch.
pub mod filesystem {
    /// Filesystem access API.
    pub mod filesystem_trait;
    pub use filesystem_trait::Filesystem;

    // FIXME: MAKE TRAIT
    cfg_if::cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            pub(crate) mod filesystem_native;
        } else {
            pub(crate) mod filesystem_wasm32;
        }
    }
}

#[cfg(feature = "discord-rpc")]
/// Discord RPC related structs.
pub mod discord;

use std::cell::RefCell;
use std::sync::Arc;

use components::toasts::Toasts;
pub use eframe::egui_glow::glow;
use egui::TextureOptions;
use egui_extras::RetainedImage;
pub use luminol::Luminol;
use saved_state::SavedState;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

use crate::data::data_cache::DataCache;
use crate::filesystem::Filesystem;

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
    pub data_cache: DataCache,
    /// Windows that are displayed.
    pub windows: windows::window::Windows,
    /// Tabs that are displayed.
    pub tabs: tabs::tab::Tabs,
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
            filesystem: Default::default(),
            data_cache: Default::default(),
            windows: Default::default(),
            tabs: Default::default(),
            audio: Default::default(),
            toasts: Default::default(),
            gl,
            saved_state: RefCell::new(state),
            toolbar: Default::default(),
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
        &info.filesystem.read_bytes(&format!("{}.png", path)).await?,
    )
    .map(|i| i.with_options(TextureOptions::NEAREST))
}

/// Load a gl texture from disk.
pub async fn load_image_hardware(
    path: String,
    info: &'static UpdateInfo,
) -> Result<glow::Texture, String> {
    use glow::HasContext;

    let image =
        image::load_from_memory(&info.filesystem.read_bytes(&format!("{}.png", path)).await?)
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
