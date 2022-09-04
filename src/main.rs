#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.drag_and_drop_support = true;

    let image = image::load_from_memory(luminol::ICON).expect("Failed to load Icon data.");

    native_options.icon_data = Some(eframe::IconData {
        width: image.width(),
        height: image.height(),
        rgba: image.into_bytes(),
    });

    eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| Box::new(luminol::App::new(cc))),
    );
}

// when compiling to web using trunk.
#[cfg(target_arch = "wasm32")]
fn main() {
    // Make sure panics are logged using `console.error`.
    console_error_panic_hook::set_once();

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let web_options = eframe::WebOptions::default();
    eframe::start_web(
        "the_canvas_id", // hardcode it
        web_options,
        Box::new(|cc| Box::new(luminol::App::new(cc))),
    )
    .expect("failed to start eframe");
}
