#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{handle_fatal_error, run};
#[cfg(target_arch = "wasm32")]
pub use web::{handle_fatal_error, run, worker_start};
