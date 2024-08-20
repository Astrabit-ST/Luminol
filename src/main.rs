// Copyright (C) 2024 Melody Madeline Lyons
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
#![cfg_attr(target_arch = "wasm32", no_main)] // there is no main function in web builds

shadow_rs::shadow!(build);

pub(crate) use luminol_result as result;

pub const BUILD_DIAGNOSTIC: luminol_core::BuildDiagnostics = luminol_core::BuildDiagnostics {
    build_time: build::BUILD_TIME,
    rustc_version: build::RUST_VERSION,
    cargo_version: build::CARGO_VERSION,
    build_os: build::BUILD_OS,
    git_revision: git_revision(),
    is_debug: cfg!(debug_assertions),
};

pub static RESTART_AFTER_PANIC: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

const fn git_revision() -> &'static str {
    #[cfg(not(target_arch = "wasm32"))]
    {
        git_version::git_version!()
    }
    #[cfg(target_arch = "wasm32")]
    match option_env!("LUMINOL_VERSION") {
        Some(v) => v,
        None => git_version::git_version!(),
    }
}

pub(crate) mod app;
mod entrypoint;
pub(crate) mod lumi;
#[cfg(feature = "steamworks")]
mod steam;

/// The native application entry point.
///
/// The application initialisation process on native platforms differs from web.\
/// Before showing any UI, Luminol has to:
///
/// 1. Load the latest panic report. (if there is one)
/// 2. Initialise the Steamworks Application Programming Interface. (if 'steamworks' feature is enabled)
/// 3. Set up all appropriate loggers, hooks and debugging utilities to handle a possible error/crash.
///
/// Only then can Luminol show the main window.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    if let Err(why) = entrypoint::run() {
        entrypoint::handle_fatal_error(why);
    }
}

/// The web entry point.
///
/// The application initialisation process on web engines differs from native platforms.\
/// Before showing any UI, Luminol has to:
///
/// 1. Load the latest panic report.
/// 2. Set up all appropriate loggers, hooks and debugging utilities to handle a possible error/crash.
/// 3. Check for appropriate headers and safety mechanisms.
/// 4. Create a canvas to render the application.
/// 5. Initialise the file system interface.
///
/// Only then can Luminol show the main window.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn luminol_main_start() {
    if let Err(why) = entrypoint::run() {
        entrypoint::handle_fatal_error(why);
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub async fn luminol_worker_start(canvas: web_sys::OffscreenCanvas) {
    entrypoint::worker_start(canvas).await;
}
