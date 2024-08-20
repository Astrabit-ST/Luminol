use crate::{
    git_revision,
    result::{Error, Result},
    RESTART_AFTER_PANIC,
};
use std::{rc::Rc, sync::Arc};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast};

type PanicSender =
    Arc<parking_lot::lock_api::Mutex<parking_lot::RawMutex, Option<oneshot::Sender<String>>>>;

const CANVAS_ID: &str = "luminol-canvas";

static WORKER_DATA: parking_lot::Mutex<Option<WorkerData>> = parking_lot::Mutex::new(None);

struct WorkerData {
    report: Option<String>,
    audio: luminol_audio::Audio,
    modified: luminol_core::ModifiedState,
    prefers_color_scheme_dark: Option<bool>,
    fs_worker_channels: luminol_filesystem::web::WorkerChannels,
    runner_worker_channels: luminol_eframe::web::WorkerChannels,
    runner_panic_tx: std::sync::Arc<parking_lot::Mutex<Option<oneshot::Sender<()>>>>,
}

#[wasm_bindgen(
    inline_js = "let report = null; export function get_panic_report() { return report; }; export function set_panic_report(r) { report = r; window.restartLuminol(); };"
)]
extern "C" {
    fn get_panic_report() -> Option<String>;
    fn set_panic_report(r: String);
}

pub fn handle_fatal_error(why: Error) {
    handle_fatal_error_str(why.to_string());
}
fn handle_fatal_error_str<Str: Into<String>>(text: Str) {
    let mut panicked = false;
    let mut text: String = text.into();
    let mut location: Option<String> = None;

    if text.starts_with("The application panicked") {
        panicked = true;
        let ntext = text.clone();
        let lines: Vec<&str> = ntext.lines().collect();

        text = lines[1].split(':').collect::<Vec<&str>>()[1]
            .trim()
            .to_string();
        location = Some(
            lines[2].split(':').collect::<Vec<&str>>()[1]
                .trim()
                .to_string(),
        )
    }

    let window = web_sys::window().expect("could not get `window` object");
    let document = window
        .document()
        .expect("could not get `window.document` object");

    let div = document
        .create_element("div")
        .expect("could not create a `div` element")
        .unchecked_into::<web_sys::HtmlElement>();
    let div_style = div.style();
    let _ = div_style.set_property("position", "absolute");
    let _ = div_style.set_property("top", "50%");
    let _ = div_style.set_property("left", "50%");
    let _ = div_style.set_property("transform", "translate(-50%, -50%)");
    let _ = div_style.set_property("color", "white");
    let _ = div_style.set_property("display", "flex");

    let img = document
        .create_element("img")
        .expect("could not create an `<img>` element")
        .unchecked_into::<web_sys::HtmlImageElement>();
    img.set_src("./icon-256.png");
    img.set_width(128);

    let msg_div = document
        .create_element("div")
        .expect("could not create a `div` element")
        .unchecked_into::<web_sys::HtmlElement>();
    let msg_div_style = msg_div.style();
    let _ = msg_div_style.set_property("padding-left", "1rem");
    let _ = msg_div_style.set_property("font-family", "monospace");

    let h1 = document
        .create_element("h1")
        .expect("could not create a `h2` element");
    h1.set_inner_html("Oops! Luminol crashed.");

    let p = document
        .create_element("p")
        .expect("could not create a `p`");
    p.set_inner_html(
        if panicked {
            format!("{text}<br><br>Location: {}", location.unwrap())
        } else {
            text
        }
        .as_str(),
    );

    msg_div
        .append_child(&h1)
        .expect("could not append a `<h1>` to `<div>`'s body");
    msg_div
        .append_child(&p)
        .expect("could not append a `<p>` to `<div>`'s body");
    div.append_child(&img)
        .expect("could not append an `<img>` to `<div>`'s body");
    div.append_child(&msg_div)
        .expect("could not append a `<div>` to the root `<div>`");
    document
        .body()
        .expect("could not get `document.body` object")
        .append_child(&div)
        .expect("could not append a `<div>` to the document's body");
}

fn setup_hooks(panic_tx: PanicSender) {
    let (panic_hook, eyre_hook) = color_eyre::config::HookBuilder::default()
        .panic_section(format!("Luminol version: {}", git_revision()))
        .theme(color_eyre::config::Theme::new()) // owo-colors doesn't work on web
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
}

fn check_cross_origin_isolation() {
    if !luminol_web::bindings::cross_origin_isolated() {
        tracing::error!(
            "Cross-Origin Isolation is not enabled. Reloading the page in an attempt to enable it."
        );
        web_sys::window()
            .expect("could not get `window` object")
            .location()
            .reload()
            .expect("could not reload the page");
    }
}

fn get_canvases() -> (web_sys::HtmlCanvasElement, web_sys::OffscreenCanvas) {
    let window = web_sys::window().expect("could not get `window` object");
    let document = window
        .document()
        .expect("could not get `window.document` object");
    let canvas = document
        .create_element("canvas")
        .expect("could not create a `<canvas>` element")
        .unchecked_into::<web_sys::HtmlCanvasElement>();
    document
        .get_element_by_id(CANVAS_ID)
        .expect(format!("could not find an element with the id of `{CANVAS_ID}`").as_str())
        .replace_children_with_node_1(&canvas);
    let offscreen_canvas = canvas
        .transfer_control_to_offscreen()
        .expect("could not transfer canvas control to offscreen");

    (canvas, offscreen_canvas)
}
fn prefers_color_scheme_dark() -> Option<bool> {
    let window = web_sys::window().expect("could not get `window` object");
    window
        .match_media("(prefers-color-scheme: dark)")
        .unwrap()
        .map(|x| x.matches())
}

pub fn init_fs() -> luminol_filesystem::web::WorkerChannels {
    let (fs_worker_channels, fs_main_channels) = luminol_filesystem::web::channels();
    luminol_filesystem::web::setup_main_thread_hooks(fs_main_channels);
    fs_worker_channels
}

pub fn launch_worker(
    canvas: web_sys::HtmlCanvasElement,
    offscreen_canvas: web_sys::OffscreenCanvas,
    report: Option<String>,
    prefers_color_scheme_dark: Option<bool>,
    fs_worker_channels: luminol_filesystem::web::WorkerChannels,
    worker_cell: Rc<once_cell::unsync::OnceCell<web_sys::Worker>>,
    before_unload_cell: Rc<std::cell::RefCell<Option<Closure<dyn Fn(web_sys::BeforeUnloadEvent)>>>>,
) {
    let window = web_sys::window().expect("could not get `window` object");

    let (runner_worker_channels, runner_main_channels) = luminol_eframe::web::channels();
    let runner_panic_tx =
        luminol_eframe::WebRunner::setup_main_thread_hooks(luminol_eframe::web::MainState {
            inner: Default::default(),
            text_agent: Default::default(),
            canvas: canvas.clone(),
            channels: runner_main_channels,
        })
        .expect("unable to setup web runner main thread hooks");
    let modified_state = luminol_core::ModifiedState::default();

    *WORKER_DATA.lock() = Some(WorkerData {
        report,
        audio: luminol_audio::Audio::default(),
        modified: modified_state.clone(),
        prefers_color_scheme_dark,
        fs_worker_channels,
        runner_worker_channels,
        runner_panic_tx,
    });

    // Show confirmation dialogue if the user tries to close the browser tab while there are
    // unsaved changes in the current project
    {
        let closure: Closure<dyn Fn(_)> = Closure::new(move |e: web_sys::BeforeUnloadEvent| {
            if modified_state.get() {
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

pub fn run() -> Result<()> {
    /* Load the latest panic report */
    let report = get_panic_report();

    let worker_cell = Rc::new(once_cell::unsync::OnceCell::<web_sys::Worker>::new());
    let before_unload_cell = Rc::new(std::cell::RefCell::new(
        None::<Closure<dyn Fn(web_sys::BeforeUnloadEvent)>>,
    ));
    let (panic_tx, panic_rx) = oneshot::channel();
    let panic_tx = Arc::new(parking_lot::Mutex::new(Some(panic_tx)));

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
                    handle_fatal_error_str(report);
                }
            }
        });
    }

    /* Set up hooks for formatting errors and panics */
    setup_hooks(panic_tx);

    /* Redirect tracing to console.log and friends */
    tracing_wasm::set_as_global_default_with_config(
        tracing_wasm::WASMLayerConfigBuilder::new()
            .set_max_level(tracing::Level::INFO)
            .build(),
    );

    /* Redirect log (mainly egui) to tracing */
    tracing_log::LogTracer::init()?;

    /* Check if the Cross-Origin Isolation header is enabld */
    check_cross_origin_isolation();

    /* Create canvases */
    let (canvas, offscreen_canvas) = get_canvases();
    /* Check if the user prefers the dark colour scheme */
    let prefers_color_scheme_dark = prefers_color_scheme_dark();

    /* Initialise the file system driver */
    let fs_worker_channels = init_fs();

    launch_worker(
        canvas,
        offscreen_canvas,
        report,
        prefers_color_scheme_dark,
        fs_worker_channels,
        worker_cell,
        before_unload_cell,
    );

    Ok(())
}

#[wasm_bindgen]
pub async fn worker_start(canvas: web_sys::OffscreenCanvas) {
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
            Box::new(|cc| Ok(Box::new(crate::app::App::new(cc, report, modified, audio)))),
            luminol_eframe::web::WorkerOptions {
                prefers_color_scheme_dark,
                channels: runner_worker_channels,
            },
        )
        .await
        .expect("failed to start eframe");
}
