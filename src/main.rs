#![warn(clippy::all, rust_2018_idioms)]
#![allow(clippy::uninlined_format_args)]
// Copyright (C) 2023 Lily Lyons
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
//cargo r
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use luminol::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    //let runtime = tokio::runtime::Builder::new_current_thread()
    //    .worker_threads(1)
    //    .enable_io()
    //    .build()
    //    .expect("failed to create tokio runtime");
    //let _guard = runtime.enter();

    #[cfg(feature = "steamworks")]
    if let Err(e) = luminol::steam::Steamworks::setup() {
        rfd::MessageDialog::new()
            .set_title("Error")
            .set_level(rfd::MessageLevel::Error)
            .set_description(format!(
                "Steam error: {e}\nPerhaps you want to compile yourself a free copy?"
            ))
            .show();
        return;
    }

    #[cfg(debug_assertions)]
    std::thread::spawn(|| loop {
        use std::fmt::Write;

        std::thread::sleep(std::time::Duration::from_secs(5));

        let deadlocks = parking_lot::deadlock::check_deadlock();
        if deadlocks.is_empty() {
            continue;
        }

        rfd::MessageDialog::new()
            .set_title("Fatal Error")
            .set_level(rfd::MessageLevel::Error)
            .set_description(&format!(
                "Luminol has deadlocked! Please file an issue.\n{} deadlocks detected",
                deadlocks.len()
            ))
            .show();
        for (i, threads) in deadlocks.iter().enumerate() {
            let mut description = String::new();
            for t in threads {
                writeln!(description, "Thread Id {:#?}", t.thread_id()).unwrap();
                writeln!(description, "{:#?}", t.backtrace()).unwrap();
            }
            rfd::MessageDialog::new()
                .set_title(&format!("Deadlock #{i}"))
                .set_level(rfd::MessageLevel::Error)
                .set_description(&description)
                .show();
        }

        std::process::abort();
    });

    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    color_eyre::install().expect("failed to setup eyre hooks");

    #[cfg(windows)]
    if let Err(e) = setup_file_assocs() {
        eprintln!("error setting up registry {e}")
    }

    let image = image::load_from_memory(luminol::ICON).expect("Failed to load Icon data.");

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(eframe::IconData {
            width: image.width(),
            height: image.height(),
            rgba: image.into_bytes(),
        }),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: eframe::wgpu::util::backend_bits_from_env()
                .unwrap_or(eframe::wgpu::Backends::PRIMARY),
            device_descriptor: luminol::Arc::new(|_| eframe::wgpu::DeviceDescriptor {
                label: Some("luminol device descriptor"),
                features: eframe::wgpu::Features::PUSH_CONSTANTS,
                limits: eframe::wgpu::Limits {
                    max_push_constant_size: 128,
                    ..eframe::wgpu::Limits::default()
                },
            }),
            ..Default::default()
        },
        app_id: Some("astrabit.luminol".to_string()),
        ..Default::default()
    };

    eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| Box::new(luminol::Luminol::new(cc, std::env::args_os().nth(1)))),
    )
    .expect("failed to start luminol");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
struct GlobalState {
    device_pixel_ratio: f32,
    prefers_color_scheme_dark: Option<bool>,
}

// The main thread and worker thread share the same global variables
#[cfg(target_arch = "wasm32")]
static GLOBAL_STATE: once_cell::sync::OnceCell<GlobalState> = once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
struct GlobalCallbackState {
    screen_resize_tx: mpsc::Sender<(u32, u32)>,
    event_tx: mpsc::Sender<egui::Event>,
}

#[cfg(target_arch = "wasm32")]
static GLOBAL_CALLBACK_STATE: once_cell::sync::OnceCell<GlobalCallbackState> =
    once_cell::sync::OnceCell::new();

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "luminol-canvas";

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn luminol_main_start() {
    let (panic, _) = color_eyre::config::HookBuilder::new().into_hooks();
    std::panic::set_hook(Box::new(move |info| {
        let report = panic.panic_report(info);

        web_sys::console::log_1(&js_sys::JsString::from(report.to_string()));
    }));

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default();

    let window = web_sys::window().expect("could not get `window` object (make sure you're running this in the main thread of a web browser)");
    let device_pixel_ratio = window.device_pixel_ratio() as f32;
    let prefers_color_scheme_dark = window
        .match_media("(prefers-color-scheme: dark)")
        .unwrap()
        .map(|x| x.matches());

    if GLOBAL_STATE
        .set(GlobalState {
            device_pixel_ratio,
            prefers_color_scheme_dark,
        })
        .is_err()
    {
        panic!("failed to initialize global variables");
    }

    let canvas = window
        .document()
        .expect("could not get `window.document` object (make sure you're running this in a web browser)")
        .get_element_by_id(CANVAS_ID)
        .expect(format!("could not find HTML element with ID '{CANVAS_ID}'").as_str())
        .unchecked_into::<web_sys::HtmlCanvasElement>();
    let offscreen_canvas = canvas
        .transfer_control_to_offscreen()
        .expect("could not transfer canvas control to offscreen");

    let mut worker_options = web_sys::WorkerOptions::new();
    worker_options.name("luminol-primary");
    worker_options.type_(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options("/worker.js", &worker_options)
        .expect("failed to spawn web worker");

    let callback = Closure::once(luminol_main_callback);
    worker.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();

    let message = js_sys::Array::new();
    message.push(&JsValue::from("init"));
    message.push(&wasm_bindgen::memory());
    message.push(&offscreen_canvas);
    let transfer = js_sys::Array::new();
    transfer.push(&offscreen_canvas);
    worker
        .post_message_with_transfer(&message, &transfer)
        .expect("failed to post message to web worker");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn luminol_worker_start(canvas: web_sys::OffscreenCanvas) {
    let (screen_resize_tx, screen_resize_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::channel();
    if GLOBAL_CALLBACK_STATE
        .set(GlobalCallbackState {
            screen_resize_tx,
            event_tx,
        })
        .is_err()
    {
        panic!("failed to initialize global callback variables");
    }

    luminol::web::get_worker()
        .post_message(&JsValue::null())
        .expect("failed to post callback message from web worker to main thread");

    let web_options = eframe::WebOptions::default();

    let state = GLOBAL_STATE.get().unwrap();
    let runner = luminol::web::WebWorkerRunner::new(
        Box::new(|cc| Box::new(luminol::Luminol::new(cc, std::env::args_os().nth(1)))),
        canvas,
        web_options,
        state.device_pixel_ratio,
        state.prefers_color_scheme_dark,
        Some(screen_resize_rx),
        Some(event_rx),
    )
    .await;
    runner.setup_render_hook();
}

#[cfg(target_arch = "wasm32")]
fn luminol_main_callback() {
    let state = GLOBAL_CALLBACK_STATE.get().unwrap();
    let window = web_sys::window().unwrap();

    let canvas = window
        .document()
        .unwrap()
        .get_element_by_id(CANVAS_ID)
        .unwrap()
        .unchecked_into::<web_sys::HtmlCanvasElement>();

    luminol::web::web_worker_runner::setup_main_thread_hooks(
        canvas,
        &state.screen_resize_tx,
        &state.event_tx,
    );
}

#[cfg(windows)]
fn setup_file_assocs() -> std::io::Result<()> {
    /*
       use winreg::enums::*;
       use winreg::RegKey;

       let path = std::env::current_exe().expect("failed to get current executable path");
       let path = path.to_string_lossy();
       let command = format!("\"{path}\" \"%1\"");

       let hkcu = RegKey::predef(HKEY_CURRENT_USER);

       // RXPROJ
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.rxproj")?;
       key.set_value("", &"Luminol.rxproj")?;
       let (rxproj_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.rxproj")?;
       rxproj_key.set_value("", &"RPG Maker XP Project")?;
       let (open_key, _) = rxproj_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
       let (icon_key, _) = rxproj_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",2"))?;

       // RXDATA
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.rxdata")?;
       key.set_value("", &"Luminol.rxdata")?;
       let (rxdata_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.rxdata")?;
       rxdata_key.set_value("", &"RPG Maker XP Data")?;
       let (icon_key, _) = rxdata_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",3"))?;

       // LUMPROJ
       let (key, _) = hkcu.create_subkey("Software\\Classes\\.lumproj")?;
       key.set_value("", &"Luminol.lumproj")?;
       let (lumproj_key, _) = hkcu.create_subkey("Software\\Classes\\Luminol.lumproj")?;
       lumproj_key.set_value("", &"Luminol project")?;
       let (open_key, _) = lumproj_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
       let (icon_key, _) = lumproj_key.create_subkey("DefaultIcon")?;
       icon_key.set_value("", &format!("\"{path}\",4"))?;

       let (app_key, _) = hkcu.create_subkey("Software\\Classes\\Applications\\luminol.exe")?;
       app_key.set_value("FriendlyAppName", &"Luminol")?;
       let (supported_key, _) = app_key.create_subkey("SupportedTypes")?;
       supported_key.set_value(".rxproj", &"")?;
       supported_key.set_value(".lumproj", &"")?;
       let (open_key, _) = app_key.create_subkey("shell\\open\\command")?;
       open_key.set_value("", &command)?;
    */
    Ok(())
}
