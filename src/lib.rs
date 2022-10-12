#![warn(clippy::all, rust_2018_idioms)]
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
#![feature(drain_filter)]

mod luminol;

mod audio;

mod windows {
    pub mod about;
    pub mod event_edit;
    pub mod map_picker;
    pub mod misc;
    pub mod sound_test;
    pub mod window;
}

mod components {
    pub mod toasts;
    pub mod toolbar;
    pub mod top_bar;

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

        pub trait TilemapDef {
            fn new(info: &'static UpdateInfo, id: i32) -> Self;

            fn ui(
                &mut self,
                ui: &mut egui::Ui,
                map: &rpg::Map,
                cursor_pos: &mut egui::Pos2,
                toggled_layers: &[bool],
                selected_layer: usize,
            ) -> egui::Response;

            fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16);

            fn textures_loaded(&self) -> bool;
        }
    }
}

mod tabs {
    pub mod map;
    pub mod started;
    pub mod tab;
}

mod data {
    pub mod data_cache;
    pub mod rgss_structs;
    pub mod rmxp_structs;
}

mod filesystem {
    cfg_if::cfg_if! {
        if #[cfg(not(target_arch = "wasm32"))] {
            mod filesystem_native;
            pub use filesystem_native::Filesystem;
        } else {
            mod filesystem_wasm32;
            pub use filesystem_wasm32::Filesystem;
        }
    }
}

#[cfg(feature = "discord-rpc")]
mod discord;

use std::sync::Arc;

use components::toasts::Toasts;
pub use eframe::egui_glow::glow;
use egui::TextureFilter;
use egui_extras::RetainedImage;
pub use luminol::Luminol;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

use crate::data::data_cache::DataCache;
use crate::filesystem::Filesystem;

/// Passed to windows and widgets when updating.
pub struct UpdateInfo {
    pub filesystem: Filesystem,
    pub data_cache: DataCache,
    pub windows: windows::window::Windows,
    pub tabs: tabs::tab::Tabs,
    pub audio: audio::Audio,
    pub toasts: Toasts,
    pub gl: Arc<glow::Context>,
}

impl UpdateInfo {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            filesystem: Default::default(),
            data_cache: Default::default(),
            windows: Default::default(),
            tabs: Default::default(),
            audio: Default::default(),
            toasts: Default::default(),
            gl,
        }
    }
}

pub async fn load_image_software(
    path: String,
    info: &'static UpdateInfo,
) -> Result<RetainedImage, String> {
    egui_extras::RetainedImage::from_image_bytes(
        path.clone(),
        &info.filesystem.read_bytes(&format!("{}.png", path)).await?,
    )
    .map(|i| i.with_texture_filter(TextureFilter::Nearest))
}

pub async fn load_image_hardware(
    path: String,
    info: &'static UpdateInfo,
) -> Result<glow::NativeTexture, String> {
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
