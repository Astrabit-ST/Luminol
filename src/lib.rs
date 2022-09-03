#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod data {
    pub mod rgss_structs;
    pub mod rmxp_structs;
}
pub mod marshal {
    pub mod deserialize;
    pub mod serialize;
    pub mod error;
}
pub use app::App;
