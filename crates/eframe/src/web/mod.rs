//! [`egui`] bindings for web apps (compiling to WASM).

#![allow(clippy::missing_errors_doc)] // So many `-> Result<_, JsValue>`

mod app_runner;
mod backend;
mod events;
mod input;
mod panic_handler;
mod text_agent;
mod web_logger;
mod web_runner;

/// Access to the browser screen reader.
#[cfg(feature = "web_screen_reader")]
pub mod screen_reader;

/// Access to local browser storage.
pub mod storage;

pub(crate) use app_runner::AppRunner;
pub use panic_handler::{PanicHandler, PanicSummary};
pub use web_logger::WebLogger;
pub use web_runner::WebRunner;

#[cfg(not(any(feature = "glow", feature = "wgpu")))]
compile_error!("You must enable either the 'glow' or 'wgpu' feature");

mod web_painter;

#[cfg(feature = "glow")]
mod web_painter_glow;
#[cfg(feature = "glow")]
pub(crate) type ActiveWebPainter = web_painter_glow::WebPainterGlow;

#[cfg(feature = "wgpu")]
mod web_painter_wgpu;
#[cfg(all(feature = "wgpu", not(feature = "glow")))]
pub(crate) type ActiveWebPainter = web_painter_wgpu::WebPainterWgpu;

pub use backend::*;

use egui::Vec2;
use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;

use input::*;

use crate::Theme;

// ----------------------------------------------------------------------------

pub(crate) fn string_from_js_value(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:#?}"))
}

/// Current time in seconds (since undefined point in time).
///
/// Monotonically increasing.
pub fn now_sec() -> f64 {
    luminol_web::bindings::performance(
        &luminol_web::bindings::worker().expect("should have a DedicatedWorkerGlobalScope"),
    )
    .now()
        / 1000.0
}

/// The native GUI scale factor, taking into account the browser zoom.
///
/// Corresponds to [`window.devicePixelRatio`](https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio) in JavaScript.
pub fn native_pixels_per_point() -> f32 {
    let pixels_per_point = web_sys::window().unwrap().device_pixel_ratio() as f32;
    if pixels_per_point > 0.0 && pixels_per_point.is_finite() {
        pixels_per_point
    } else {
        1.0
    }
}

/// Ask the browser about the preferred system theme.
///
/// `None` means unknown.
pub fn system_theme() -> Option<Theme> {
    let dark_mode = prefers_color_scheme_dark(&web_sys::window()?)
        .ok()??
        .matches();
    Some(theme_from_dark_mode(dark_mode))
}

fn prefers_color_scheme_dark(window: &web_sys::Window) -> Result<Option<MediaQueryList>, JsValue> {
    window.match_media("(prefers-color-scheme: dark)")
}

fn theme_from_dark_mode(dark_mode: bool) -> Theme {
    if dark_mode {
        Theme::Dark
    } else {
        Theme::Light
    }
}

fn canvas_element(canvas_id: &str) -> Option<web_sys::HtmlCanvasElement> {
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id(canvas_id)?;
    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

fn canvas_element_or_die(canvas_id: &str) -> web_sys::HtmlCanvasElement {
    canvas_element(canvas_id)
        .unwrap_or_else(|| panic!("Failed to find canvas with id {canvas_id:?}"))
}

fn canvas_origin(canvas: &web_sys::HtmlCanvasElement) -> egui::Pos2 {
    let rect = canvas.get_bounding_client_rect();
    egui::pos2(rect.left() as f32, rect.top() as f32)
}

fn canvas_size_in_points(canvas: &web_sys::HtmlCanvasElement) -> egui::Vec2 {
    let pixels_per_point = native_pixels_per_point();
    egui::vec2(
        canvas.width() as f32 / pixels_per_point,
        canvas.height() as f32 / pixels_per_point,
    )
}

fn resize_canvas_to_screen_size(
    canvas: &web_sys::HtmlCanvasElement,
    max_size_points: egui::Vec2,
) -> Option<()> {
    let parent = canvas.parent_element()?;

    // Prefer the client width and height so that if the parent
    // element is resized that the egui canvas resizes appropriately.
    let width = parent.client_width();
    let height = parent.client_height();

    let canvas_real_size = Vec2 {
        x: width as f32,
        y: height as f32,
    };

    if width <= 0 || height <= 0 {
        log::error!("egui canvas parent size is {}x{}. Try adding `html, body {{ height: 100%; width: 100% }}` to your CSS!", width, height);
    }

    let pixels_per_point = native_pixels_per_point();

    let max_size_pixels = pixels_per_point * max_size_points;

    let canvas_size_pixels = pixels_per_point * canvas_real_size;
    let canvas_size_pixels = canvas_size_pixels.min(max_size_pixels);
    let canvas_size_points = canvas_size_pixels / pixels_per_point;

    // Make sure that the height and width are always even numbers.
    // otherwise, the page renders blurry on some platforms.
    // See https://github.com/emilk/egui/issues/103
    fn round_to_even(v: f32) -> f32 {
        (v / 2.0).round() * 2.0
    }

    canvas
        .style()
        .set_property(
            "width",
            &format!("{}px", round_to_even(canvas_size_points.x)),
        )
        .ok()?;
    canvas
        .style()
        .set_property(
            "height",
            &format!("{}px", round_to_even(canvas_size_points.y)),
        )
        .ok()?;
    canvas.set_width(round_to_even(canvas_size_pixels.x) as u32);
    canvas.set_height(round_to_even(canvas_size_pixels.y) as u32);

    Some(())
}

// ----------------------------------------------------------------------------

/// Set the cursor icon.
fn set_cursor_icon(cursor: egui::CursorIcon) -> Option<()> {
    let document = web_sys::window()?.document()?;
    document
        .body()?
        .style()
        .set_property("cursor", cursor_web_name(cursor))
        .ok()
}

/// Set the clipboard text.
#[cfg(web_sys_unstable_apis)]
fn set_clipboard_text(s: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(clipboard) = window.navigator().clipboard() {
            let promise = clipboard.write_text(s);
            let future = wasm_bindgen_futures::JsFuture::from(promise);
            let future = async move {
                if let Err(err) = future.await {
                    log::error!("Copy/cut action failed: {}", string_from_js_value(&err));
                }
            };
            wasm_bindgen_futures::spawn_local(future);
        }
    }
}

fn cursor_web_name(cursor: egui::CursorIcon) -> &'static str {
    match cursor {
        egui::CursorIcon::Alias => "alias",
        egui::CursorIcon::AllScroll => "all-scroll",
        egui::CursorIcon::Cell => "cell",
        egui::CursorIcon::ContextMenu => "context-menu",
        egui::CursorIcon::Copy => "copy",
        egui::CursorIcon::Crosshair => "crosshair",
        egui::CursorIcon::Default => "default",
        egui::CursorIcon::Grab => "grab",
        egui::CursorIcon::Grabbing => "grabbing",
        egui::CursorIcon::Help => "help",
        egui::CursorIcon::Move => "move",
        egui::CursorIcon::NoDrop => "no-drop",
        egui::CursorIcon::None => "none",
        egui::CursorIcon::NotAllowed => "not-allowed",
        egui::CursorIcon::PointingHand => "pointer",
        egui::CursorIcon::Progress => "progress",
        egui::CursorIcon::ResizeHorizontal => "ew-resize",
        egui::CursorIcon::ResizeNeSw => "nesw-resize",
        egui::CursorIcon::ResizeNwSe => "nwse-resize",
        egui::CursorIcon::ResizeVertical => "ns-resize",

        egui::CursorIcon::ResizeEast => "e-resize",
        egui::CursorIcon::ResizeSouthEast => "se-resize",
        egui::CursorIcon::ResizeSouth => "s-resize",
        egui::CursorIcon::ResizeSouthWest => "sw-resize",
        egui::CursorIcon::ResizeWest => "w-resize",
        egui::CursorIcon::ResizeNorthWest => "nw-resize",
        egui::CursorIcon::ResizeNorth => "n-resize",
        egui::CursorIcon::ResizeNorthEast => "ne-resize",
        egui::CursorIcon::ResizeColumn => "col-resize",
        egui::CursorIcon::ResizeRow => "row-resize",

        egui::CursorIcon::Text => "text",
        egui::CursorIcon::VerticalText => "vertical-text",
        egui::CursorIcon::Wait => "wait",
        egui::CursorIcon::ZoomIn => "zoom-in",
        egui::CursorIcon::ZoomOut => "zoom-out",
    }
}

/// Open the given url in the browser.
pub fn open_url(url: &str, new_tab: bool) -> Option<()> {
    let name = if new_tab { "_blank" } else { "_self" };

    web_sys::window()?
        .open_with_url_and_target(url, name)
        .ok()?;
    Some(())
}

/// e.g. "#fragment" part of "www.example.com/index.html#fragment",
///
/// Percent decoded
pub fn location_hash() -> String {
    percent_decode(
        &web_sys::window()
            .unwrap()
            .location()
            .hash()
            .unwrap_or_default(),
    )
}

/// Percent-decodes a string.
pub fn percent_decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .to_string()
}

// ----------------------------------------------------------------------------

/// Options and state that will be sent to the web worker part of the web runner.
#[derive(Clone)]
pub struct WorkerOptions {
    /// Whether or not the user's browser prefers dark mode.
    /// `Some(true)` means dark mode is preferred.
    /// `Some(false)` means light mode is preferred.
    /// `None` means no preference was detected.
    pub prefers_color_scheme_dark: Option<bool>,
    /// The halves of the web runner channels that are used in the web worker.
    pub channels: WorkerChannels,
}

/// The halves of the web runner channels that are used in the web worker.
#[derive(Clone)]
pub struct WorkerChannels {
    /// The receiver used to receive egui events from the main thread.
    event_rx: flume::Receiver<egui::Event>,
    /// The receiver used to receive custom events from the main thread.
    custom_event_rx: flume::Receiver<WebRunnerCustomEvent>,
    /// The sender used to send outputs to the main thread.
    output_tx: flume::Sender<WebRunnerOutput>,
}

impl WorkerChannels {
    /// Send an output to the main thread.
    fn send(&self, output: WebRunnerOutput) {
        let _ = self.output_tx.send(output);
    }
}

/// The state of the web runner that is accessible to the main thread.
#[derive(Clone)]
pub struct MainState {
    /// The state of the web runner that is accessible to the main thread.
    pub inner: std::rc::Rc<std::cell::RefCell<MainStateInner>>,
    /// The HTML canvas element that this runner renders onto.
    pub canvas: web_sys::HtmlCanvasElement,
    /// The halves of the web runner channels that are used in the main thread.
    pub channels: MainChannels,
}

/// The state of the web runner that is accessible to the main thread.
#[derive(Default)]
pub struct MainStateInner {
    /// If the user is currently interacting with the touchscreen, this is the ID of the touch,
    /// measured with `Touch.identifier` in JavaScript.
    touch_id: Option<egui::TouchId>,
    /// The position relative to the canvas of the last received touch event. If no touch event has
    /// been received yet, this will be (0, 0).
    touch_pos: egui::Pos2,
    /// If the user is typing something, the position of the text cursor (for IME) in screen
    /// coordinates.
    text_cursor_pos: Option<egui::Pos2>,
    /// Whether or not the user is editing a mutable egui text box.
    mutable_text_under_cursor: bool,
    /// Whether or not egui is trying to receive text input.
    wants_keyboard_input: bool,
}

/// The halves of the web runner channels that are used in the main thread.
#[derive(Clone)]
pub struct MainChannels {
    /// The sender used to send egui events to the worker thread.
    event_tx: flume::Sender<egui::Event>,
    /// The sender used to send custom events to the worker thread.
    custom_event_tx: flume::Sender<WebRunnerCustomEvent>,
    /// The receiver used to receive outputs from the worker thread.
    output_rx: flume::Receiver<WebRunnerOutput>,
}

impl MainState {
    /// Add an event listener to the given JavaScript `EventTarget`.
    fn add_event_listener<E: wasm_bindgen::JsCast>(
        &self,
        target: &web_sys::EventTarget,
        event_name: &'static str,
        mut closure: impl FnMut(E, &MainState) + 'static,
    ) -> Result<(), wasm_bindgen::JsValue> {
        let state = self.clone();

        // Create a JS closure based on the FnMut provided
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            // Only call the wrapped closure if the egui code has not panicked
            if PANIC_LOCK.get().is_none() {
                // Cast the event to the expected event type
                let event = event.unchecked_into::<E>();
                closure(event, &state);
            }
        }) as Box<dyn FnMut(web_sys::Event)>);

        // Add the event listener to the target
        target.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;

        closure.forget();
        Ok(())
    }
}

impl MainChannels {
    /// Send an egui event to the worker thread.
    fn send(&self, event: egui::Event) {
        let _ = self.event_tx.send(event);
    }

    /// Send a custom event to the worker thread.
    fn send_custom(&self, event: WebRunnerCustomEvent) {
        let _ = self.custom_event_tx.send(event);
    }
}

/// Create a new connected `(WorkerChannels, MainChannels)` pair for initializing a web runner.
pub fn channels() -> (WorkerChannels, MainChannels) {
    let (event_tx, event_rx) = flume::unbounded();
    let (custom_event_tx, custom_event_rx) = flume::unbounded();
    let (output_tx, output_rx) = flume::unbounded();
    (
        WorkerChannels {
            event_rx,
            custom_event_rx,
            output_tx,
        },
        MainChannels {
            event_tx,
            custom_event_tx,
            output_rx,
        },
    )
}

/// A custom event that can be sent from the main thread to the worker thread.
enum WebRunnerCustomEvent {
    /// (window.innerWidth, window.innerHeight, window.devicePixelRatio)
    ScreenResize(u32, u32, f32),
    /// This should be sent whenever the modifiers change
    Modifiers(egui::Modifiers),
    /// This should be sent whenever the app needs to save immediately
    Save,
    /// The browser detected a touchstart or touchmove event with this ID and position in canvas coordinates
    Touch(Option<egui::TouchId>, egui::Pos2),
}

/// A custom output that can be sent from the worker thread to the main thread.
enum WebRunnerOutput {
    /// Miscellaneous egui output events
    PlatformOutput(egui::PlatformOutput, bool, bool),
    /// The runner wants to read a key from storage
    StorageGet(String, oneshot::Sender<Option<String>>),
    /// The runner wants to write a key to storage
    StorageSet(String, String, oneshot::Sender<bool>),
}

static PANIC_LOCK: once_cell::sync::OnceCell<()> = once_cell::sync::OnceCell::new();
