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
//cargo r
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.
#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![cfg_attr(target_arch = "wasm32", no_main)] // there is no main function in web builds

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
/// Embedded icon 256x256 in size.
const ICON: &[u8] = luminol_macros::include_asset!("assets/icons/icon.png");

static RESTART_AFTER_PANIC: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod log;
mod lumi;

#[cfg(all(feature = "steamworks", target_arch = "wasm32"))]
compile_error!("Steamworks is not supported on webassembly");

#[cfg(feature = "steamworks")]
mod steam;

pub fn git_revision() -> &'static str {
    #[cfg(not(target_arch = "wasm32"))]
    {
        git_version::git_version!()
    }
    #[cfg(target_arch = "wasm32")]
    option_env!("LUMINOL_VERSION").unwrap_or(git_version::git_version!())
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Load the panic report from the previous run if it exists
    let mut report = None;
    if let Some(path) = std::env::var_os("LUMINOL_PANIC_REPORT_FILE") {
        if let Ok(mut file) = std::fs::File::open(&path) {
            let mut buffer = String::new();
            if file.read_to_string(&mut buffer).is_ok() {
                report = Some(buffer);
            }
        }
        let _ = std::fs::remove_file(path);
    }

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

    // Enable full backtraces
    std::env::set_var("RUST_BACKTRACE", "full");

    // Set up hooks for formatting errors and panics
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", git_revision()))
        .add_frame_filter(Box::new(|frames| {
            let filters = &[
                "_",
                "core::",
                "alloc::",
                "cocoa::",
                "tokio::",
                "winit::",
                "accesskit",
                "std::rt::",
                "std::sys_",
                "windows::",
                "egui::ui::",
                "E as eyre::",
                "T as core::",
                "egui_dock::",
                "std::panic::",
                "egui::context::",
                "luminol_eframe::",
                "std::panicking::",
                "egui::containers::",
                "glPushClientAttrib",
                "std::thread::local::",
            ];
            frames.retain(|frame| {
                !filters.iter().any(|f| {
                    frame.name.as_ref().is_some_and(|name| {
                        name.starts_with(|c: char| c.is_ascii_uppercase())
                            || name.strip_prefix('<').unwrap_or(name).starts_with(f)
                    })
                })
            })
        }))
        .into_hooks();
    eyre_hook
        .install()
        .expect("failed to install color-eyre hooks");
    std::panic::set_hook(Box::new(move |info| {
        let report = panic_hook.panic_report(info).to_string();
        eprintln!("{report}");

        if !RESTART_AFTER_PANIC.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }

        let mut args = std::env::args_os();
        let arg0 = args.next();
        let exe_path = std::env::current_exe().map_or_else(
            |_| arg0.expect("could not get path to current executable"),
            |exe_path| exe_path.into_os_string(),
        );

        let mut file = tempfile::NamedTempFile::new().expect("failed to create temporary file");
        file.write_all(report.as_bytes())
            .expect("failed to write to temporary file");
        file.flush().expect("failed to flush temporary file");
        let (_, path) = file.keep().expect("failed to persist temporary file");
        std::env::set_var("LUMINOL_PANIC_REPORT_FILE", &path);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let error = std::process::Command::new(exe_path).args(args).exec();
            eprintln!("Failed to restart Luminol: {error:?}");
            let _ = std::fs::remove_file(&path);
        }

        #[cfg(not(unix))]
        {
            if let Err(error) = std::process::Command::new(exe_path).args(args).spawn() {
                eprintln!("Failed to restart Luminol: {error:?}");
                let _ = std::fs::remove_file(&path);
            }
        }
    }));

    let (log_byte_tx, log_byte_rx) = std::sync::mpsc::channel();
    let ctx_cell = std::sync::Arc::new(once_cell::sync::OnceCell::new());
    log::initialize_log(log_byte_tx, ctx_cell.clone());

    let image = image::load_from_memory(ICON).expect("Failed to load Icon data.");

    let native_options = luminol_eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_icon(egui::IconData {
                width: image.width(),
                height: image.height(),
                rgba: image.to_rgba8().into_vec(),
            })
            .with_app_id("astrabit.luminol"),
        wgpu_options: luminol_egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::util::backend_bits_from_env()
                .unwrap_or(wgpu::Backends::PRIMARY | wgpu::Backends::SECONDARY),
            power_preference: wgpu::util::power_preference_from_env()
                .unwrap_or(wgpu::PowerPreference::LowPower),
            ..Default::default()
        },
        persist_window: true,

        ..Default::default()
    };

    luminol_eframe::run_native(
        "Luminol",
        native_options,
        Box::new(move |cc| {
            ctx_cell
                .set(cc.egui_ctx.clone())
                .expect("egui context cell already set (this shouldn't happen!)");

            Ok(Box::new(app::App::new(
                cc,
                report,
                Default::default(),
                log_byte_rx,
                std::env::args_os().nth(1),
                #[cfg(feature = "steamworks")]
                steamworks,
            )))
        }),
    )
    .expect("failed to start luminol");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(
    inline_js = "let report = null; export function get_panic_report() { return report; }; export function set_panic_report(r) { report = r; window.restartLuminol(); };"
)]
extern "C" {
    fn get_panic_report() -> Option<String>;
    fn set_panic_report(r: String);
}

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "luminol-canvas";

#[cfg(target_arch = "wasm32")]
struct WorkerData {
    report: Option<String>,
    audio: luminol_audio::AudioWrapper,
    modified: luminol_core::ModifiedState,
    prefers_color_scheme_dark: Option<bool>,
    fs_worker_channels: luminol_filesystem::web::WorkerChannels,
    runner_worker_channels: luminol_eframe::web::WorkerChannels,
    runner_panic_tx: std::sync::Arc<parking_lot::Mutex<Option<oneshot::Sender<()>>>>,
}

#[cfg(target_arch = "wasm32")]
static WORKER_DATA: parking_lot::Mutex<Option<WorkerData>> = parking_lot::Mutex::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn luminol_main_start() {
    // Load the panic report from the previous run if it exists
    let report = get_panic_report();

    let worker_cell = std::rc::Rc::new(once_cell::unsync::OnceCell::<web_sys::Worker>::new());
    let before_unload_cell = std::rc::Rc::new(std::cell::RefCell::new(
        None::<Closure<dyn Fn(web_sys::BeforeUnloadEvent)>>,
    ));
    let (panic_tx, panic_rx) = oneshot::channel();
    let panic_tx = std::sync::Arc::new(parking_lot::Mutex::new(Some(panic_tx)));

    {
        let worker_cell = worker_cell.clone();
        let before_unload_cell = before_unload_cell.clone();

        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(report) = panic_rx.await {
                if let Some(worker) = worker_cell.get() {
                    worker.terminate();
                }

                if let (Some(window), Some(closure)) =
                    (web_sys::window(), before_unload_cell.take())
                {
                    let _ = window.remove_event_listener_with_callback(
                        "beforeunload",
                        closure.as_ref().unchecked_ref(),
                    );
                }

                if RESTART_AFTER_PANIC.load(std::sync::atomic::Ordering::Relaxed) {
                    set_panic_report(report);
                } else {
                    let _ = web_sys::window().map(|window| window.alert_with_message("Luminol has crashed! Please check your browser's developer console for more details."));
                }
            }
        });
    }

    // Set up hooks for formatting errors and panics
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", git_revision()))
        .into_hooks();
    eyre_hook
        .install()
        .expect("failed to install color-eyre hooks");
    std::panic::set_hook(Box::new(move |info| {
        let report = panic_hook.panic_report(info).to_string();
        web_sys::console::log_1(&report.as_str().into());

        // Send the panic report to the main thread to be persisted
        // We need to send the panic report to the main thread because JavaScript global variables
        // are thread-local and this panic handler runs on the thread that panicked
        if let Some(mut panic_tx) = panic_tx.try_lock() {
            if let Some(panic_tx) = panic_tx.take() {
                let _ = panic_tx.send(report);
            }
        }
    }));

    let window = web_sys::window().expect("could not get `window` object");
    let prefers_color_scheme_dark = window
        .match_media("(prefers-color-scheme: dark)")
        .unwrap()
        .map(|x| x.matches());

    let document = window
        .document()
        .expect("could not get `window.document` object");
    let canvas = document
        .create_element("canvas")
        .expect("could not create canvas element")
        .unchecked_into::<web_sys::HtmlCanvasElement>();
    document
        .get_element_by_id(CANVAS_ID)
        .expect(format!("could not find HTML element with ID '{CANVAS_ID}'").as_str())
        .replace_children_with_node_1(&canvas);
    let offscreen_canvas = canvas
        .transfer_control_to_offscreen()
        .expect("could not transfer canvas control to offscreen");

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    // Redirect log (currently used by egui) to tracing
    tracing_log::LogTracer::init().expect("failed to initialize tracing-log");

    if !luminol_web::bindings::cross_origin_isolated() {
        tracing::error!(
            "Cross-Origin Isolation is not enabled. Reloading page to attempt to enable it."
        );
        window.location().reload().expect("failed to reload page");
        return;
    }

    let (fs_worker_channels, fs_main_channels) = luminol_filesystem::web::channels();
    let (runner_worker_channels, runner_main_channels) = luminol_eframe::web::channels();

    luminol_filesystem::host::setup_main_thread_hooks(fs_main_channels);
    let runner_panic_tx =
        luminol_eframe::WebRunner::setup_main_thread_hooks(luminol_eframe::web::MainState {
            inner: Default::default(),
            text_agent: Default::default(),
            canvas: canvas.clone(),
            channels: runner_main_channels,
        })
        .expect("unable to setup web runner main thread hooks");

    let modified = luminol_core::ModifiedState::default();

    *WORKER_DATA.lock() = Some(WorkerData {
        report,
        audio: luminol_audio::AudioWrapper::default(),
        modified: modified.clone(),
        prefers_color_scheme_dark,
        fs_worker_channels,
        runner_worker_channels,
        runner_panic_tx,
    });

    // Show confirmation dialogue if the user tries to close the browser tab while there are
    // unsaved changes in the current project
    {
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::BeforeUnloadEvent| {
            if modified.get() {
                // Recommended method of activating the confirmation dialogue
                e.prevent_default();
                // Fallback for Chromium < 119
                e.set_return_value("arbitrary non-empty string");
            }
        });
        window
            .add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())
            .expect("failed to add beforeunload listener");
        *before_unload_cell.borrow_mut() = Some(closure);
    }

    canvas.focus().expect("could not focus the canvas");

    let mut worker_options = web_sys::WorkerOptions::new();
    worker_options.name("luminol-primary");
    worker_options.type_(web_sys::WorkerType::Module);
    let worker = web_sys::Worker::new_with_options("./worker.js", &worker_options)
        .expect("failed to spawn web worker");
    worker_cell.set(worker.clone()).unwrap();

    let message = js_sys::Array::new();
    message.push(&wasm_bindgen::module());
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
        report,
        audio,
        modified,
        prefers_color_scheme_dark,
        fs_worker_channels,
        runner_worker_channels,
        runner_panic_tx,
    } = WORKER_DATA.lock().take().unwrap();

    luminol_filesystem::host::FileSystem::setup_worker_channels(fs_worker_channels);

    let web_options = luminol_eframe::WebOptions::default();

    luminol_eframe::WebRunner::new(runner_panic_tx)
        .start(
            canvas,
            web_options,
            Box::new(|cc| Ok(Box::new(app::App::new(cc, report, modified, audio)))),
            luminol_eframe::web::WorkerOptions {
                prefers_color_scheme_dark,
                channels: runner_worker_channels,
            },
        )
        .await
        .expect("failed to start eframe");
}
