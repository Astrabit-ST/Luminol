#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod data {
    pub mod rgss_structs;
    pub mod rmxp_structs;
}
pub use app::App;
