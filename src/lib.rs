#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod gui {
    pub mod about;
    pub mod top_bar;
    pub mod window;
}
mod data {
    pub mod rgss_structs;
    pub mod rmxp_structs;
}
mod marshal {
    pub mod deserialize;
    pub mod error;
    pub mod serialize;
}
pub use app::App;

/// Embedded icon 256x256 in size.
pub const ICON: &[u8] = include_bytes!("../assets/icon-256.png");