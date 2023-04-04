#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::uninlined_format_args)]
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
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(has_std)]
fn test() {}

// TODO: Use eyre!
use color_eyre::Result;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<()> {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    color_eyre::install()?;

    let image = image::load_from_memory(luminol::ICON).expect("Failed to load Icon data.");

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        icon_data: Some(eframe::IconData {
            width: image.width(),
            height: image.height(),
            rgba: image.into_bytes(),
        }),
        ..Default::default()
    };

    eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| Box::new(luminol::Luminol::new(cc))),
    )
    .expect("failed to start luminol");

    Ok(())
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() -> Result<()> {
    let (panic, _) = color_eyre::config::HookBuilder::new().into_hooks();
    std::panic::set_hook(Box::new(move |info| {
        let report = panic.panic_report(info);

        web_sys::console::log_1(&js_sys::JsString::from(report.to_string()));
    }));

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::start_web(
            "the_canvas_id", // hardcode it
            web_options,
            Box::new(|cc| Box::new(luminol::Luminol::new(cc))),
        )
        .await
        .expect("failed to start eframe");
    });

    Ok(())
}
