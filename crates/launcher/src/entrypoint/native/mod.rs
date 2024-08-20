#![allow(clippy::arc_with_non_send_sync)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use once_cell::sync::OnceCell;

use crate::{
    app,
    result::{Error, Result},
};
use std::{
    env,
    fmt::Write,
    fs,
    io::Read,
    panic, process,
    sync::{self, atomic},
    thread, time,
};

mod log;

const FILTERS: &[&str] = &[
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
const ICON: &[u8] = luminol_macros::include_asset!("assets/icons/icon.png");

pub fn handle_fatal_error(why: Error) {
    rfd::MessageDialog::new()
        .set_title("Error")
        .set_level(rfd::MessageLevel::Error)
        .set_description(why.to_string())
        .show();
    process::exit(1);
}
fn unwrap_option<T: std::any::Any>(opt: Option<T>, err: Error) -> T {
    match opt {
        Some(val) => val,
        None => {
            handle_fatal_error(err);
            unreachable!()
        }
    }
}
fn unwrap_result<T: std::any::Any>(res: Result<T>) -> T {
    match res {
        Ok(val) => val,
        Err(why) => {
            handle_fatal_error(why);
            unreachable!();
        }
    }
}

fn load_panic_report() -> Option<String> {
    let mut report = None;
    if let Some(path) = env::var_os("LUMINOL_PANIC_REPORT_FILE") {
        if let Ok(mut file) = fs::File::open(path) {
            let mut buffer = String::new();
            if file.read_to_string(&mut buffer).is_ok() {
                report = Some(buffer);
            }
        }
    }
    report
}

#[cfg(debug_assertions)]
fn detect_deadlocks() {
    thread::spawn(|| loop {
        thread::sleep(time::Duration::from_secs(5));

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

        process::abort();
    });
}

fn setup_hooks() -> Result<()> {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", crate::git_revision()))
        .add_frame_filter(Box::new(|frames| {
            frames.retain(|frame| {
                !FILTERS.iter().any(|f| {
                    frame.name.as_ref().is_some_and(|name| {
                        name.starts_with(|c: char| c.is_ascii_uppercase())
                            || name.strip_prefix('<').unwrap_or(name).starts_with(f)
                    })
                })
            })
        }))
        .into_hooks();
    eyre_hook.install()?;
    panic::set_hook(Box::new(move |info| {
        let report = panic_hook.panic_report(info).to_string();
        eprintln!("{report}");

        if !crate::RESTART_AFTER_PANIC.load(atomic::Ordering::Relaxed) {
            return;
        }

        let mut args = env::args_os();
        let arg0 = args.next();
        let exe_path = env::current_exe().map_or_else(
            |_| unwrap_option(arg0, Error::ExePathQueryFailed),
            |exe_path| exe_path.into_os_string(),
        );

        let mut file = unwrap_result(tempfile::NamedTempFile::new().map_err(Error::Io));
        let path = unwrap_result(|report: String| -> Result<std::path::PathBuf> {
            use std::io::Write;

            file.write_all(report.as_bytes())?;
            file.flush()?;
            let (_, path) = file.keep()?;
            Ok(path)
        }(report));
        env::set_var("LUMINOL_PANIC_REPORT_FILE", &path);

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            let error = process::Command::new(exe_path).args(args).exec();
            eprintln!("Failed to restart Luminol: {error:?}");
            let _ = fs::remove_file(&path);
        }
        #[cfg(not(unix))]
        {
            if let Err(error) = std::process::Command::new(exe_path).args(args).spawn() {
                eprintln!("Failed to restart Luminol: {error:?}");
                let _ = std::fs::remove_file(&path);
            }
        }
    }));

    Ok(())
}

fn init_log() -> (sync::Arc<OnceCell<egui::Context>>, sync::mpsc::Receiver<u8>) {
    let (log_byte_tx, log_byte_rx) = sync::mpsc::channel();
    let ctx_cell = sync::Arc::new(OnceCell::new());
    log::initialize_log(log_byte_tx, ctx_cell.clone());
    (ctx_cell, log_byte_rx)
}

fn run_app(
    report: Option<String>,
    ctx_cell: sync::Arc<OnceCell<egui::Context>>,
    log_byte_rx: sync::mpsc::Receiver<u8>,
    #[cfg(feature = "steamworks")] steamworks: crate::steam::Steamworks,
) -> Result<()> {
    let icon_image = image::load_from_memory(ICON)?;

    luminol_eframe::run_native(
        "Luminol",
        luminol_eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_drag_and_drop(true)
                .with_icon(egui::IconData {
                    width: icon_image.width(),
                    height: icon_image.height(),
                    rgba: icon_image.to_rgba8().to_vec(),
                })
                .with_app_id("astrabit.luminol"),
            wgpu_options: luminol_egui_wgpu::WgpuConfiguration {
                supported_backends: wgpu::util::backend_bits_from_env()
                    .unwrap_or(wgpu::Backends::PRIMARY | wgpu::Backends::SECONDARY),
                // TODO: Load this value from a settings file
                power_preference: wgpu::util::power_preference_from_env()
                    .unwrap_or(wgpu::PowerPreference::LowPower),
                ..Default::default()
            },
            persist_window: true,

            ..Default::default()
        },
        Box::new(move |cc| {
            unwrap_result(
                ctx_cell
                    .set(cc.egui_ctx.clone())
                    .map_err(|_| Error::EguiContextCellAlreadySet),
            );

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

    Ok(())
}

pub fn run() -> Result<()> {
    /* Load the latest panic report */
    let report = load_panic_report();

    /* Initialise the Steamworks Application Programming Interface */
    #[cfg(feature = "steamworks")]
    let steamworks = crate::steam::Steamworks::new()?;

    /* Detect deadlocks */
    #[cfg(debug_assertions)]
    detect_deadlocks();

    /* Enable full backtraces */
    env::set_var("RUST_BACKTRACE", "full");

    /* Set up hooks for formatting errors and panics */
    setup_hooks()?;

    /* Initialise the log system */
    let (ctx_cell, log_byte_rx) = init_log();

    /* Show the graphical user interface */
    run_app(
        report,
        ctx_cell,
        log_byte_rx,
        #[cfg(feature = "steamworks")]
        steamworks,
    )?;

    Ok(())
}
