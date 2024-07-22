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

use wasm_bindgen::prelude::*;
use web_sys::MediaQueryList;

use input::*;

use crate::Theme;

// ----------------------------------------------------------------------------

pub(crate) fn string_from_js_value(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:#?}"))
}

/// Returns the `Element` with active focus.
///
/// Elements can only be focused if they are:
/// - `<a>`/`<area>` with an `href` attribute
/// - `<input>`/`<select>`/`<textarea>`/`<button>` which aren't `disabled`
/// - any other element with a `tabindex` attribute
pub(crate) fn focused_element() -> Option<web_sys::Element> {
    web_sys::window()?
        .document()?
        .active_element()?
        .dyn_into()
        .ok()
}

pub(crate) fn has_focus<T: JsCast>(element: &T) -> bool {
    fn try_has_focus<T: JsCast>(element: &T) -> Option<bool> {
        let element = element.dyn_ref::<web_sys::Element>()?;
        let focused_element = focused_element()?;
        Some(element == &focused_element)
    }
    try_has_focus(element).unwrap_or(false)
}

/// Current time in seconds (since undefined point in time).
///
/// Monotonically increasing.
pub fn now_sec() -> f64 {
    luminol_web::bindings::worker()
        .expect("should have a DedicatedWorkerGlobalScope")
        .performance()
        .unwrap()
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

fn get_canvas_element_by_id(canvas_id: &str) -> Option<web_sys::HtmlCanvasElement> {
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id(canvas_id)?;
    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

fn get_canvas_element_by_id_or_die(canvas_id: &str) -> web_sys::HtmlCanvasElement {
    get_canvas_element_by_id(canvas_id)
        .unwrap_or_else(|| panic!("Failed to find canvas with id {canvas_id:?}"))
}

/// Returns the canvas in client coordinates.
fn canvas_content_rect(canvas: &web_sys::HtmlCanvasElement) -> egui::Rect {
    let bounding_rect = canvas.get_bounding_client_rect();

    let mut rect = egui::Rect::from_min_max(
        egui::pos2(bounding_rect.left() as f32, bounding_rect.top() as f32),
        egui::pos2(bounding_rect.right() as f32, bounding_rect.bottom() as f32),
    );

    // We need to subtract padding and border:
    if let Some(window) = web_sys::window() {
        if let Ok(Some(style)) = window.get_computed_style(canvas) {
            let get_property = |name: &str| -> Option<f32> {
                let property = style.get_property_value(name).ok()?;
                property.trim_end_matches("px").parse::<f32>().ok()
            };

            rect.min.x += get_property("padding-left").unwrap_or_default();
            rect.min.y += get_property("padding-top").unwrap_or_default();
            rect.max.x -= get_property("padding-right").unwrap_or_default();
            rect.max.y -= get_property("padding-bottom").unwrap_or_default();
        }
    }

    rect
}

fn canvas_size_in_points(canvas: &web_sys::HtmlCanvasElement, zoom_factor: f32) -> egui::Vec2 {
    let pixels_per_point = zoom_factor * native_pixels_per_point();
    egui::vec2(
        canvas.width() as f32 / pixels_per_point,
        canvas.height() as f32 / pixels_per_point,
    )
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
        } else {
            let is_secure_context = window.is_secure_context();
            if is_secure_context {
                log::warn!("window.navigator.clipboard is null; can't copy text");
            } else {
                log::warn!("window.navigator.clipboard is null; can't copy text, probably because we're not in a secure context. See https://developer.mozilla.org/en-US/docs/Web/Security/Secure_Contexts");
            }
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

// ensure that AtomicF32 and AtomicF64 is using atomic ops (otherwise it would use global locks, and that would be bad)
const _: [(); 0 - !{
    const ASSERT: bool = portable_atomic::AtomicF32::is_always_lock_free()
        && portable_atomic::AtomicF64::is_always_lock_free();
    ASSERT
} as usize] = [];

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
    event_rx: flume::Receiver<WebRunnerEvent>,
    /// The sender used to send outputs to the main thread.
    output_tx: flume::Sender<WebRunnerOutput>,
    /// This should be set to the app's current zoom factor every frame.
    zoom_tx: std::sync::Arc<portable_atomic::AtomicF32>,
    /// This should be set to whether or not any mouse button is down or any touchscreen touches
    /// are occurring on every frame.
    pointer_down_tx: std::sync::Arc<portable_atomic::AtomicBool>,
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
    /// The text agent that eframe uses to handle IME. This will always be set after
    /// the web runner's initialization.
    pub text_agent: std::rc::Rc<once_cell::unsync::OnceCell<text_agent::TextAgent>>,
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
    /// This should be set to `true` (HTML canvas is focused) or `false` (HTML canvas is blurred)
    /// every frame.
    has_focus: bool,
}

/// The halves of the web runner channels that are used in the main thread.
#[derive(Clone)]
pub struct MainChannels {
    /// The sender used to send egui events to the worker thread.
    event_tx: flume::Sender<WebRunnerEvent>,
    /// The receiver used to receive outputs from the worker thread.
    output_rx: flume::Receiver<WebRunnerOutput>,
    /// This is set to the app's current zoom factor every frame.
    zoom_rx: std::sync::Arc<portable_atomic::AtomicF32>,
    /// This is set to whether or not any mouse button is down or any touchscreen touches are
    /// occurring on every frame.
    pointer_down_rx: std::sync::Arc<portable_atomic::AtomicBool>,
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

        // Add a hook to unregister this event listener after panicking
        EVENTS_TO_UNSUBSCRIBE.with_borrow_mut(|events| {
            events.push(web_runner::EventToUnsubscribe::TargetEvent(
                web_runner::TargetEvent {
                    target: target.clone(),
                    event_name: event_name.to_string(),
                    closure,
                },
            ));
        });

        Ok(())
    }

    /// You need to call this every frame to poll for changes to the app's HTML canvas focus/blur
    /// state.
    fn update_focus(&self) {
        let has_focus = has_focus(&self.canvas)
            || self
                .text_agent
                .get()
                .expect("text agent should be initialized at this point")
                .has_focus();
        let mut inner = self.inner.borrow_mut();
        if inner.has_focus != has_focus {
            log::trace!("Focus changed to {has_focus}");
            inner.has_focus = has_focus;

            if !has_focus {
                // We lost focus - good idea to save
                self.channels.send_custom(WebRunnerEvent::Save);
            }
        }
    }
}

impl MainChannels {
    /// Send an egui event to the worker thread.
    fn send(&self, event: egui::Event) {
        let _ = self.event_tx.send(WebRunnerEvent::EguiEvent(event));
    }

    /// Send a custom event to the worker thread.
    fn send_custom(&self, event: WebRunnerEvent) {
        let _ = self.event_tx.send(event);
    }

    /// Get the egui app's current zoom factor from the worker thread.
    fn zoom_factor(&self) -> f32 {
        self.zoom_rx.load(portable_atomic::Ordering::Relaxed)
    }

    /// Ask the worker thread if any mouse button is down or any touchscreen touches are occurring.
    fn is_pointer_down(&self) -> bool {
        self.pointer_down_rx
            .load(portable_atomic::Ordering::Relaxed)
    }
}

/// Create a new connected `(WorkerChannels, MainChannels)` pair for initializing a web runner.
pub fn channels() -> (WorkerChannels, MainChannels) {
    let (event_tx, event_rx) = flume::unbounded();
    let (output_tx, output_rx) = flume::unbounded();
    let zoom_arc = std::sync::Arc::new(portable_atomic::AtomicF32::new(1.));
    let pointer_down_arc = std::sync::Arc::new(portable_atomic::AtomicBool::new(false));
    (
        WorkerChannels {
            event_rx,
            output_tx,
            zoom_tx: zoom_arc.clone(),
            pointer_down_tx: pointer_down_arc.clone(),
        },
        MainChannels {
            event_tx,
            output_rx,
            zoom_rx: zoom_arc,
            pointer_down_rx: pointer_down_arc,
        },
    )
}

/// A custom event that can be sent from the main thread to the worker thread.
enum WebRunnerEvent {
    /// Misc egui events
    EguiEvent(egui::Event),
    /// This should be sent whenever a repaint is desired without anything else (all the other
    /// `WebRunnerEvent` types will also request a repaint)
    Repaint,
    /// (window.innerWidth, window.innerHeight, window.devicePixelRatio)
    ScreenResize(u32, u32, f32),
    /// This should be sent whenever the modifiers change
    Modifiers(egui::Modifiers),
    /// This should be sent whenever the app needs to save immediately
    Save,
    /// The browser detected a touchstart or touchmove event with this ID
    Touch(Option<egui::TouchId>),
    /// This should be sent whenever the web page gains or loses focus (true when focus is gained,
    /// false when focus is lost)
    Focus(bool),
    /// This should be sent whenever the app detects scrolling; please use this instead of
    /// `egui::Event::MouseWheel`
    Wheel(egui::MouseWheelUnit, egui::Vec2, egui::Modifiers),
    /// This should be sent whenever the control key or meta key is released after being pressed
    CommandKeyReleased,
    /// The browser detected that the hash in the URL changed to this value
    Hash(String),
    /// The browser detected that the color scheme changed
    Theme(Theme),
}

/// A custom output that can be sent from the worker thread to the main thread.
enum WebRunnerOutput {
    /// Miscellaneous egui output events
    PlatformOutput(egui::PlatformOutput, bool),
    /// The runner wants to read a key from storage
    StorageGet(String, oneshot::Sender<Option<String>>),
    /// The runner wants to write a key to storage
    StorageSet(String, String, oneshot::Sender<bool>),
}

static PANIC_LOCK: once_cell::sync::OnceCell<()> = once_cell::sync::OnceCell::new();

thread_local! {
    static EVENTS_TO_UNSUBSCRIBE: std::cell::RefCell<Vec<web_runner::EventToUnsubscribe>> = std::cell::RefCell::new(Vec::new());
}
