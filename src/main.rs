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
#![warn(rust_2018_idioms)]
#![warn(
    clippy::all,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::panicking_unwrap,
    clippy::unnecessary_wraps,
    // unsafe code is sometimes fine but in general we don't want to use it.
    unsafe_code,
)]
// These may be turned on in the future.
// #![warn(clippy::unwrap, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::doc_markdown,
    clippy::missing_panics_doc,
    clippy::too_many_lines
)]
// You must provide a safety doc.
#![forbid(clippy::missing_safety_doc, unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Embedded icon 256x256 in size.
const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

mod app;
mod lumi;

#[cfg(all(feature = "steamworks", target_arch = "wasm32"))]
compile_error!("Steamworks is not supported on webassembly");

#[cfg(feature = "steamworks")]
mod steam;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    #[cfg(feature = "steamworks")]
    let steamworks = match steam::Steamworks::new() {
        Ok(s) => s,
        Err(e) => {
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_level(rfd::MessageLevel::Error)
                .set_description(format!(
                    "Steam error: {e}\nPerhaps you want to compile yourself a free copy?"
                ))
                .show();
            return;
        }
    };

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
            .set_description(format!(
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

    color_backtrace::BacktracePrinter::new()
        .verbosity(color_backtrace::Verbosity::Full)
        .install(color_backtrace::default_output_stream());

    let image = image::load_from_memory(ICON).expect("Failed to load Icon data.");

    let native_options = eframe::NativeOptions {
        drag_and_drop_support: true,
        transparent: true,
        icon_data: Some(eframe::IconData {
            width: image.width(),
            height: image.height(),
            rgba: image.to_rgba8().into_vec(),
        }),
        wgpu_options: eframe::egui_wgpu::WgpuConfiguration {
            supported_backends: eframe::wgpu::util::backend_bits_from_env()
                .unwrap_or(eframe::wgpu::Backends::PRIMARY),
            device_descriptor: std::sync::Arc::new(|_| eframe::wgpu::DeviceDescriptor {
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
        persist_window: true,
        ..Default::default()
    };

    eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| {
            Box::new(app::App::new(
                cc,
                std::env::args_os().nth(1),
                #[cfg(feature = "steamworks")]
                steamworks,
            ))
        }),
    )
    .expect("failed to start luminol");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "luminol-canvas";

#[cfg(target_arch = "wasm32")]
struct WorkerData {
    audio: luminol_audio::AudioWrapper,
    prefers_color_scheme_dark: Option<bool>,
    filesystem_tx: flume::Sender<luminol_filesystem::host::FileSystemCommand>,
    web_runner_channels: eframe::web::WebRunnerChannels,
}

#[cfg(target_arch = "wasm32")]
static WORKER_DATA: parking_lot::RwLock<Option<WorkerData>> = parking_lot::RwLock::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn luminol_main_start(fallback: bool) {
    let (panic_tx, panic_rx) = flume::unbounded();

    wasm_bindgen_futures::spawn_local(async move {
        if panic_rx.recv_async().await.is_ok() {
            let _ = web_sys::window().map(|window| window.alert_with_message("Luminol has crashed! Please check your browser's developer console for more details."));
        }
    });

    std::panic::set_hook(Box::new(move |info| {
        let backtrace_printer =
            color_backtrace::BacktracePrinter::new().verbosity(color_backtrace::Verbosity::Full);
        let mut buffer = color_backtrace::termcolor::Ansi::new(vec![]);
        let _ = backtrace_printer.print_panic_info(info, &mut buffer);
        let report = String::from_utf8(buffer.into_inner()).expect("panic report not valid utf-8");

        web_sys::console::log_1(&js_sys::JsString::from(report));

        let _ = panic_tx.send(());
    }));

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    // Redirect log (currently used by egui) to tracing
    tracing_log::LogTracer::init().expect("failed to initialize tracing-log");

    let window = web_sys::window().expect("could not get `window` object (make sure you're running this in the main thread of a web browser)");
    let prefers_color_scheme_dark = window
        .match_media("(prefers-color-scheme: dark)")
        .unwrap()
        .map(|x| x.matches());

    let canvas = window
        .document()
        .expect("could not get `window.document` object (make sure you're running this in a web browser)")
        .get_element_by_id(CANVAS_ID)
        .expect(format!("could not find HTML element with ID '{CANVAS_ID}'").as_str())
        .unchecked_into::<web_sys::HtmlCanvasElement>();
    let offscreen_canvas = canvas
        .transfer_control_to_offscreen()
        .expect("could not transfer canvas control to offscreen");

    if !luminol_web::bindings::cross_origin_isolated() {
        tracing::error!("Luminol requires Cross-Origin Isolation to be enabled in order to run.");
        return;
    }

    let (filesystem_tx, filesystem_rx) = flume::unbounded();
    let (output_tx, output_rx) = flume::unbounded();
    let (event_tx, event_rx) = flume::unbounded();
    let (custom_event_tx, custom_event_rx) = flume::unbounded();

    luminol_filesystem::host::setup_main_thread_hooks(filesystem_rx);
    eframe::WebRunner::setup_main_thread_hooks(
        canvas.clone(),
        eframe::web::MainThreadChannels {
            event_tx: Some(event_tx),
            custom_event_tx: Some(custom_event_tx),
            output_rx: Some(output_rx),
        },
    )
    .expect("unable to setup web runner main thread hooks");

    *WORKER_DATA.write() = Some(WorkerData {
        audio: luminol_audio::Audio::default().into(),
        prefers_color_scheme_dark,
        filesystem_tx,
        web_runner_channels: eframe::web::WebRunnerChannels {
            event_rx: Some(event_rx),
            custom_event_rx: Some(custom_event_rx),
            output_tx: Some(output_tx),
        },
    });

    let mut worker_options = web_sys::WorkerOptions::new();
    worker_options.name("luminol-primary");
    worker_options.type_(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options("/worker.js", &worker_options)
        .expect("failed to spawn web worker");

    let message = js_sys::Array::new();
    message.push(&JsValue::from(fallback));
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
    let WorkerData {
        audio,
        prefers_color_scheme_dark,
        filesystem_tx,
        web_runner_channels,
    } = WORKER_DATA.write().take().unwrap();

    luminol_filesystem::host::FileSystem::setup_filesystem_sender(filesystem_tx);

    let web_options = eframe::WebOptions::default();

    eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Box::new(app::App::new(cc, audio))),
            eframe::web::WorkerOptions {
                app_id: String::from("astrabit.luminol"),
                prefers_color_scheme_dark,
                channels: web_runner_channels,
            },
        )
        .await
        .expect("failed to start eframe");
}
