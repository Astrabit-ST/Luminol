#![warn(clippy::all, rust_2018_idioms)]

mod app;

mod windows {
    pub mod about;
    pub mod map_picker;
    pub mod sound_test;
    pub mod window;
}

mod components {
    pub mod top_bar;
    pub mod toolbar;
    pub mod tilemap;
}

mod tabs {
    pub mod map;
    pub mod started;
    pub mod tab;
}

mod data {
    pub mod rgss_structs;
    pub mod rmxp_structs;
}
mod marshal {
    //pub mod deserialize;
    pub mod error;
    pub mod serialize;
}

mod filesystem {
    #[cfg(not(target_arch = "wasm32"))]
    mod filesystem_native;
    #[cfg(target_arch = "wasm32")]
    mod filesystem_wasm32;
    #[cfg(not(target_arch = "wasm32"))]
    pub use filesystem_native::Filesystem;
    #[cfg(target_arch = "wasm32")]
    pub use filesystem_wasm32::Filesystem;
    pub mod data_cache;
}

pub use app::App;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

use crate::filesystem::{data_cache::DataCache, Filesystem};
/// Passed to windows and widgets when updating.
pub struct UpdateInfo<'a> {
    pub filesystem: &'a Filesystem,
    pub data_cache: &'a DataCache,
    pub windows: &'a windows::window::Windows,
    pub tabs: &'a tabs::tab::Tabs,
}
