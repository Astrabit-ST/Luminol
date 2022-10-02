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

mod luminol;

mod audio;

mod windows {
    pub mod about;
    pub mod map_picker;
    pub mod sound_test;
    pub mod window;
}

mod components {
    pub mod map_toolbar;
    pub mod toasts;
    pub mod toolbar;
    pub mod top_bar;

    pub mod tilemap {
        use std::collections::HashMap;
        use egui_extras::RetainedImage;

        pub struct Textures {
            pub tileset_tex: RetainedImage,
            pub autotile_texs: Vec<Option<RetainedImage>>,
            pub event_texs: HashMap<(String, i32), Option<RetainedImage>>,
            pub fog_tex: Option<RetainedImage>,
            pub fog_zoom: i32,
            pub pano_tex: Option<RetainedImage>,
        }

        cfg_if::cfg_if! {
            if #[cfg(feature = "generic-tilemap")] {
                pub mod generic_tilemap;
                pub use generic_tilemap::Tilemap;
            } else {
                pub mod hardware_tilemap;
                pub use hardware_tilemap::Tilemap;
            }
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

use components::toasts::Toasts;
use egui::TextureFilter;
use egui_extras::RetainedImage;
pub use luminol::Luminol;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

use crate::data::data_cache::DataCache;
use crate::filesystem::Filesystem;

/// Passed to windows and widgets when updating.
pub struct UpdateInfo<'a> {
    pub filesystem: &'a Filesystem,
    pub data_cache: &'a DataCache,
    pub windows: &'a windows::window::Windows,
    pub tabs: &'a tabs::tab::Tabs,
    pub audio: &'a audio::Audio,
    pub toasts: &'a Toasts,
}

pub fn load_image_software(
    path: String,
    _hue: i32,
    filesystem: &Filesystem,
) -> Result<RetainedImage, String> {
    egui_extras::RetainedImage::from_image_bytes(
        path.clone(),
        &filesystem.read_bytes(&format!("{}.png", path))?,
    )
    .map(|i| i.with_texture_filter(TextureFilter::Nearest))
}
