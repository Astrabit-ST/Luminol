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
#![cfg_attr(target_arch = "wasm32", no_main)] // there is no main function in web builds

#[cfg(not(target_arch = "wasm32"))]
use std::io::{Read, Write};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
/// Embedded icon 256x256 in size.
const ICON: &[u8] = include_bytes!("../assets/icon-256.png");

mod app;
mod lumi;

#[cfg(all(feature = "steamworks", target_arch = "wasm32"))]
compile_error!("Steamworks is not supported on webassembly");

#[cfg(feature = "steamworks")]
mod steam;

#[cfg(not(target_arch = "wasm32"))]
/// A writer that copies whatever is written to it to two other writers.
struct CopyWriter<A, B>(A, B);

#[cfg(not(target_arch = "wasm32"))]
impl<A, B> Write for CopyWriter<A, B>
where
    A: Write,
    B: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(self.0.write(buf)?.min(self.1.write(buf)?))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()?;
        self.1.flush()?;
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
static LOG_TERM_SENDER: once_cell::sync::OnceCell<luminol_term::TermSender> =
    once_cell::sync::OnceCell::new();

#[cfg(not(target_arch = "wasm32"))]
static LOG_BYTE_SENDER: once_cell::sync::OnceCell<luminol_term::ByteSender> =
    once_cell::sync::OnceCell::new();

#[cfg(not(target_arch = "wasm32"))]
static CONTEXT: once_cell::sync::OnceCell<egui::Context> = once_cell::sync::OnceCell::new();

#[cfg(not(target_arch = "wasm32"))]
/// A writer that writes to Luminol's log window.
struct LogWriter(luminol_term::termwiz::escape::parser::Parser);

#[cfg(not(target_arch = "wasm32"))]
impl Write for LogWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        LOG_BYTE_SENDER
            .get()
            .unwrap()
            .try_send(buf.into())
            .map_err(std::io::Error::other)?;

        let parsed = self.0.parse_as_vec(buf);

        // Convert from LF line endings to CRLF so that wezterm will display them properly
        let mut vec = Vec::with_capacity(2 * parsed.len());
        for action in parsed {
            if action
                == luminol_term::termwiz::escape::Action::Control(
                    luminol_term::termwiz::escape::ControlCode::LineFeed,
                )
            {
                vec.push(luminol_term::termwiz::escape::Action::Control(
                    luminol_term::termwiz::escape::ControlCode::CarriageReturn,
                ));
            }
            vec.push(action);
        }

        LOG_TERM_SENDER
            .get()
            .unwrap()
            .try_send(vec)
            .map_err(std::io::Error::other)?;

        if let Some(ctx) = CONTEXT.get() {
            ctx.request_repaint();
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Load the panic report from the previous run if it exists
    if let Some(path) = std::env::var_os("LUMINOL_PANIC_REPORT_FILE") {
        if let Ok(mut file) = std::fs::File::open(&path) {
            let mut buffer = String::new();
            let _success = file.read_to_string(&mut buffer).is_ok();
            // TODO: use this report to open a panic reporter
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

    // Enable full backtraces unless the user manually set the RUST_BACKTRACE environment variable
    if std::env::var("RUST_BACKTRACE").is_err() {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    // Set up hooks for formatting errors and panics
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", git_version::git_version!()))
        .add_frame_filter(Box::new(|frames| {
            let filters = &[
                "_",
                "core::",
                "alloc::",
                "tokio::",
                "winit::",
                "std::rt::",
                "std::sys_",
                "egui::ui::",
                "E as eyre::",
                "T as core::",
                "egui_dock::",
                "std::panic::",
                "egui::context::",
                "luminol_eframe::",
                "std::panicking::",
                "egui::containers::",
                "std::thread::local::",
            ];
            frames.retain(|frame| {
                !filters.iter().any(|f| {
                    frame
                        .name
                        .as_ref()
                        .is_some_and(|name| name.strip_prefix('<').unwrap_or(name).starts_with(f))
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
        std::env::set_var("LUMINOL_PANIC_REPORT_FILE", path);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let error = std::process::Command::new(exe_path).args(args).exec();
            eprintln!("Failed to restart Luminol: {error:?}");
        }

        #[cfg(not(unix))]
        {
            if let Err(error) = std::process::Command::new(exe_path).args(args).spawn() {
                eprintln!("Failed to restart Luminol: {error:?}");
            }
        }
    }));

    // Log to stderr as well as Luminol's log.
    let (log_term_tx, log_term_rx) = luminol_term::unbounded();
    let (log_byte_tx, log_byte_rx) = luminol_term::unbounded();
    LOG_TERM_SENDER.set(log_term_tx).unwrap();
    LOG_BYTE_SENDER.set(log_byte_tx).unwrap();
    tracing_subscriber::fmt()
        .with_writer(|| {
            CopyWriter(
                std::io::stderr(),
                LogWriter(luminol_term::termwiz::escape::parser::Parser::new()),
            )
        })
        .init();

    let image = image::load_from_memory(ICON).expect("Failed to load Icon data.");

    let native_options = luminol_eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_drag_and_drop(true)
            .with_transparent(true)
            .with_icon(egui::IconData {
                width: image.width(),
                height: image.height(),
                rgba: image.to_rgba8().into_vec(),
            })
            .with_app_id("astrabit.luminol"),
        wgpu_options: luminol_egui_wgpu::WgpuConfiguration {
            supported_backends: wgpu::util::backend_bits_from_env()
                .unwrap_or(wgpu::Backends::PRIMARY),
            device_descriptor: std::sync::Arc::new(|_| wgpu::DeviceDescriptor {
                label: Some("luminol device descriptor"),
                features: wgpu::Features::PUSH_CONSTANTS,
                limits: wgpu::Limits {
                    max_push_constant_size: 128,
                    ..wgpu::Limits::default()
                },
            }),
            ..Default::default()
        },
        persist_window: true,
        ..Default::default()
    };

    luminol_eframe::run_native(
        "Luminol",
        native_options,
        Box::new(|cc| {
            CONTEXT.set(cc.egui_ctx.clone()).unwrap();
            Box::new(app::App::new(
                cc,
                Default::default(),
                log_term_rx,
                log_byte_rx,
                std::env::args_os().nth(1),
                #[cfg(feature = "steamworks")]
                steamworks,
            ))
        }),
    )
    .expect("failed to start luminol");
}

#[cfg(target_arch = "wasm32")]
const CANVAS_ID: &str = "luminol-canvas";

#[cfg(target_arch = "wasm32")]
struct WorkerData {
    audio: luminol_audio::AudioWrapper,
    modified: luminol_core::ModifiedState,
    prefers_color_scheme_dark: Option<bool>,
    fs_worker_channels: luminol_filesystem::web::WorkerChannels,
    runner_worker_channels: luminol_eframe::web::WorkerChannels,
}

#[cfg(target_arch = "wasm32")]
static WORKER_DATA: parking_lot::Mutex<Option<WorkerData>> = parking_lot::Mutex::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn luminol_main_start(fallback: bool) {
    let (panic_tx, panic_rx) = oneshot::channel();
    let panic_tx = std::sync::Arc::new(parking_lot::Mutex::new(Some(panic_tx)));

    wasm_bindgen_futures::spawn_local(async move {
        if panic_rx.await.is_ok() {
            let _ = web_sys::window().map(|window| window.alert_with_message("Luminol has crashed! Please check your browser's developer console for more details."));
        }
    });

    // Set up hooks for formatting errors and panics
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", git_version::git_version!()))
        .into_hooks();
    eyre_hook
        .install()
        .expect("failed to install color-eyre hooks");
    std::panic::set_hook(Box::new(move |info| {
        web_sys::console::log_1(&js_sys::JsString::from(
            panic_hook.panic_report(info).to_string(),
        ));

        if let Some(mut panic_tx) = panic_tx.try_lock() {
            if let Some(panic_tx) = panic_tx.take() {
                let _ = panic_tx.send(());
            }
        }
    }));

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

    // Redirect tracing to console.log and friends:
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    // Redirect log (currently used by egui) to tracing
    tracing_log::LogTracer::init().expect("failed to initialize tracing-log");

    if !luminol_web::bindings::cross_origin_isolated() {
        tracing::error!("Luminol requires Cross-Origin Isolation to be enabled in order to run.");
        return;
    }

    let (fs_worker_channels, fs_main_channels) = luminol_filesystem::web::channels();
    let (runner_worker_channels, runner_main_channels) = luminol_eframe::web::channels();

    luminol_filesystem::host::setup_main_thread_hooks(fs_main_channels);
    luminol_eframe::WebRunner::setup_main_thread_hooks(luminol_eframe::web::MainState {
        inner: Default::default(),
        canvas: canvas.clone(),
        channels: runner_main_channels,
    })
    .expect("unable to setup web runner main thread hooks");

    let modified = luminol_core::ModifiedState::default();

    *WORKER_DATA.lock() = Some(WorkerData {
        audio: luminol_audio::Audio::default().into(),
        modified: modified.clone(),
        prefers_color_scheme_dark,
        fs_worker_channels,
        runner_worker_channels,
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
        closure.forget();
    }

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
        modified,
        prefers_color_scheme_dark,
        fs_worker_channels,
        runner_worker_channels,
    } = WORKER_DATA.lock().take().unwrap();

    luminol_filesystem::host::FileSystem::setup_worker_channels(fs_worker_channels);

    let web_options = luminol_eframe::WebOptions::default();

    luminol_eframe::WebRunner::new()
        .start(
            canvas,
            web_options,
            Box::new(|cc| Box::new(app::App::new(cc, modified, audio))),
            luminol_eframe::web::WorkerOptions {
                prefers_color_scheme_dark,
                channels: runner_worker_channels,
            },
        )
        .await
        .expect("failed to start eframe");
}
