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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use luminol_web::IdbQuerySource;
use wasm_bindgen::JsCast;

struct UnsubscribeHook {
    target: web_sys::EventTarget,
    event_name: &'static str,
    closure: wasm_bindgen::closure::Closure<dyn FnMut(web_sys::Event)>,
}

thread_local! {
    static UNSUBSCRIBE_HOOKS: std::cell::RefCell<Vec<UnsubscribeHook>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn add_event_listener<E>(
    state_cell: std::rc::Rc<std::cell::RefCell<super::MainState>>,
    target: &web_sys::EventTarget,
    event_name: &'static str,
    mut listener: impl FnMut(&mut super::MainState, E) + 'static,
) -> Result<(), wasm_bindgen::JsValue>
where
    E: AsRef<web_sys::Event> + wasm_bindgen::JsCast,
{
    let closure = wasm_bindgen::closure::Closure::new(move |event: web_sys::Event| {
        let mut state = state_cell.borrow_mut();
        if !super::has_panicked() {
            listener(&mut state, event.unchecked_into());
        }
    });

    target.add_event_listener_with_callback(event_name, closure.as_ref().unchecked_ref())?;

    UNSUBSCRIBE_HOOKS.with_borrow_mut(|hooks| {
        hooks.push(UnsubscribeHook {
            target: target.clone(),
            event_name,
            closure,
        });
    });

    Ok(())
}

trait JsEventWithModifiers {
    fn alt_key(&self) -> bool;
    fn ctrl_key(&self) -> bool;
    fn meta_key(&self) -> bool;
    fn shift_key(&self) -> bool;

    fn egui_modifiers(&self) -> egui::Modifiers {
        let ctrl = self.ctrl_key();
        let meta = self.meta_key();
        egui::Modifiers {
            alt: self.alt_key(),
            ctrl,
            shift: self.shift_key(),
            mac_cmd: meta,
            command: ctrl || meta,
        }
    }
}

macro_rules! js_event_modifiers_impl {
    () => {
        fn alt_key(&self) -> bool {
            self.alt_key()
        }
        fn ctrl_key(&self) -> bool {
            self.ctrl_key()
        }
        fn meta_key(&self) -> bool {
            self.meta_key()
        }
        fn shift_key(&self) -> bool {
            self.shift_key()
        }
    };
}
impl JsEventWithModifiers for web_sys::KeyboardEvent {
    js_event_modifiers_impl!();
}
impl JsEventWithModifiers for web_sys::MouseEvent {
    js_event_modifiers_impl!();
}
impl JsEventWithModifiers for web_sys::TouchEvent {
    js_event_modifiers_impl!();
}

fn get_primary_touch(
    state: &mut super::MainState,
    event: &web_sys::TouchEvent,
) -> Option<web_sys::Touch> {
    for touch in (0..event.touches().length())
        .filter_map(|i| event.touches().get(i))
        .chain((0..event.changed_touches().length()).filter_map(|i| event.changed_touches().get(i)))
    {
        if !state
            .touch_id
            .is_some_and(|id| id != egui::TouchId::from(touch.identifier()))
        {
            state.touch_id = Some(egui::TouchId::from(touch.identifier()));
            state
                .event_tx
                .send(super::Event::Touch(state.touch_id))
                .unwrap();
            return Some(touch);
        }
    }
    state.touch_id = None;
    state
        .event_tx
        .send(super::Event::Touch(state.touch_id))
        .unwrap();
    None
}

pub(super) fn register_events(
    state: super::MainState,
    output_rx: flume::Receiver<super::Output>,
    panic_rx: oneshot::Receiver<()>,
    window: web_sys::Window,
) -> Result<(), wasm_bindgen::JsValue> {
    // Set up panic handler to unsubscribe all event handlers on panic
    wasm_bindgen_futures::spawn_local(async move {
        let _ = panic_rx.await;
        UNSUBSCRIBE_HOOKS.with_borrow_mut(|hooks| {
            for hook in hooks.drain(..) {
                let _ = hook.target.remove_event_listener_with_callback(
                    hook.event_name,
                    hook.closure.as_ref().unchecked_ref(),
                );
            }
        });
    });

    let document = window.document().unwrap();

    let state_cell = std::rc::Rc::new(std::cell::RefCell::new(state));
    let canvas = state_cell.borrow().canvas.clone();

    // Register event listener for screen resizing
    let listener = {
        let window = window.clone();
        move |state: &mut super::MainState, _event: web_sys::Event| {
            let device_pixel_ratio = window.device_pixel_ratio();
            let device_pixel_ratio = if device_pixel_ratio > 0. && device_pixel_ratio.is_finite() {
                device_pixel_ratio as f32
            } else {
                1.
            };
            let inner_width = window.inner_width().unwrap().as_f64().unwrap() as u32;
            let inner_height = window.inner_height().unwrap().as_f64().unwrap() as u32;
            let _ = state
                .canvas
                .set_attribute("width", inner_width.to_string().as_str());
            let _ = state
                .canvas
                .set_attribute("height", inner_height.to_string().as_str());
            state
                .event_tx
                .send(super::Event::ScreenResize {
                    inner_width,
                    inner_height,
                    device_pixel_ratio,
                })
                .unwrap();
        }
    };
    listener(&mut state_cell.borrow_mut(), web_sys::Event::new("")?);
    add_event_listener(state_cell.clone(), &window, "resize", listener)?;

    // The canvas automatically resizes itself whenever a frame is drawn.
    // The resizing does not take window.devicePixelRatio into account,
    // so this mutation observer is to detect canvas resizes and correct them.
    {
        let window = window.clone();
        let callback: wasm_bindgen::closure::Closure<dyn FnMut(_)> =
            wasm_bindgen::closure::Closure::new(move |mutations: js_sys::Array| {
                if super::has_panicked() {
                    return;
                }
                let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
                let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
                mutations.for_each(&mut |mutation, _index, _mutations| {
                    let mutation = mutation.unchecked_into::<web_sys::MutationRecord>();
                    if mutation.type_().as_str() != "attributes" {
                        return;
                    }
                    let Some(canvas) = mutation.target() else {
                        return;
                    };
                    let canvas = canvas.unchecked_into::<web_sys::HtmlCanvasElement>();
                    if canvas.width() == width && canvas.height() == height {
                        return;
                    }
                    let _ = canvas.set_attribute("width", width.to_string().as_str());
                    let _ = canvas.set_attribute("height", height.to_string().as_str());
                });
            });
        let observer = web_sys::MutationObserver::new(callback.as_ref().unchecked_ref())?;
        let mut options = web_sys::MutationObserverInit::new();
        options.attributes(true);
        observer.observe_with_options(&canvas, &options)?;
        // We don't need to unregister this mutation observer on panic because it auto-deregisters
        // when the target (the canvas) is removed from the DOM and garbage-collected
        callback.forget();
    }

    // Register event listener for mouse button presses and releases
    let listener_builder = |pressed| {
        move |state: &mut super::MainState, event: web_sys::MouseEvent| {
            let modifiers = event.egui_modifiers();
            state
                .event_tx
                .send(super::Event::Modifiers(modifiers))
                .unwrap();
            if let Some(button) = match event.button() {
                0 => Some(egui::PointerButton::Primary),
                1 => Some(egui::PointerButton::Middle),
                2 => Some(egui::PointerButton::Secondary),
                3 => Some(egui::PointerButton::Extra1),
                4 => Some(egui::PointerButton::Extra2),
                _ => None,
            } {
                state
                    .event_tx
                    .send(super::Event::Egui(egui::Event::PointerButton {
                        pos: egui::pos2(
                            event.client_x() as f32 / state.zoom_factor,
                            event.client_y() as f32 / state.zoom_factor,
                        ),
                        button,
                        pressed,
                        modifiers,
                    }))
                    .unwrap();
            }
            let _ = state.canvas.focus();
            event.stop_propagation();
            if !pressed {
                event.prevent_default();
            }
        }
    };
    add_event_listener(
        state_cell.clone(),
        &canvas,
        "mousedown",
        listener_builder(true),
    )?;
    add_event_listener(
        state_cell.clone(),
        &document,
        "mouseup",
        listener_builder(false),
    )?;

    // Register event listener for mouse movement
    let listener = move |state: &mut super::MainState, event: web_sys::MouseEvent| {
        let modifiers = event.egui_modifiers();
        state
            .event_tx
            .send(super::Event::Modifiers(modifiers))
            .unwrap();
        state
            .event_tx
            .send(super::Event::Egui(egui::Event::PointerMoved(egui::pos2(
                event.client_x() as f32 / state.zoom_factor,
                event.client_y() as f32 / state.zoom_factor,
            ))))
            .unwrap();
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "mousemove", listener)?;

    // Register event listener for mouse going outside of the canvas
    let listener = move |state: &mut super::MainState, event: web_sys::MouseEvent| {
        state.event_tx.send(super::Event::Save).unwrap();
        state
            .event_tx
            .send(super::Event::Egui(egui::Event::PointerGone))
            .unwrap();
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "mouseleave", listener)?;

    // Register event listener for touch presses and releases on a touchscreen device
    let listener_builder = |pressed| {
        move |state: &mut super::MainState, event: web_sys::TouchEvent| {
            if let Some(touch) = get_primary_touch(state, &event) {
                state
                    .event_tx
                    .send(super::Event::Egui(egui::Event::PointerButton {
                        pos: egui::pos2(
                            touch.client_x() as f32 / state.zoom_factor,
                            touch.client_y() as f32 / state.zoom_factor,
                        ),
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: event.egui_modifiers(),
                    }))
                    .unwrap();
                if !pressed {
                    state
                        .event_tx
                        .send(super::Event::Egui(egui::Event::PointerGone))
                        .unwrap();
                }
            }
            for touch in
                (0..event.changed_touches().length()).filter_map(|i| event.changed_touches().get(i))
            {
                state
                    .event_tx
                    .send(super::Event::Egui(egui::Event::Touch {
                        device_id: egui::TouchDeviceId(0),
                        id: egui::TouchId::from(touch.identifier()),
                        phase: if pressed {
                            egui::TouchPhase::Start
                        } else {
                            egui::TouchPhase::End
                        },
                        pos: egui::pos2(
                            touch.client_x() as f32 / state.zoom_factor,
                            touch.client_y() as f32 / state.zoom_factor,
                        ),
                        force: Some(touch.force()),
                    }))
                    .unwrap();
            }
            event.stop_propagation();
            event.prevent_default();
        }
    };
    add_event_listener(
        state_cell.clone(),
        &canvas,
        "touchstart",
        listener_builder(true),
    )?;
    add_event_listener(
        state_cell.clone(),
        &document,
        "touchend",
        listener_builder(false),
    )?;

    // Register event listener for dragging on a touchscreen device
    let listener = move |state: &mut super::MainState, event: web_sys::TouchEvent| {
        if let Some(touch) = get_primary_touch(state, &event) {
            state
                .event_tx
                .send(super::Event::Egui(egui::Event::PointerMoved(egui::pos2(
                    touch.client_x() as f32 / state.zoom_factor,
                    touch.client_y() as f32 / state.zoom_factor,
                ))))
                .unwrap();
        }
        for touch in
            (0..event.changed_touches().length()).filter_map(|i| event.changed_touches().get(i))
        {
            state
                .event_tx
                .send(super::Event::Egui(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId::from(touch.identifier()),
                    phase: egui::TouchPhase::Move,
                    pos: egui::pos2(
                        touch.client_x() as f32 / state.zoom_factor,
                        touch.client_y() as f32 / state.zoom_factor,
                    ),
                    force: Some(touch.force()),
                }))
                .unwrap();
        }
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "touchmove", listener)?;

    // Register event listener for a touch being cancelled on a touchscreen device (e.g. the
    // browser window lost focus while the user was dragging on the screen)
    let listener = move |state: &mut super::MainState, event: web_sys::TouchEvent| {
        for touch in
            (0..event.changed_touches().length()).filter_map(|i| event.changed_touches().get(i))
        {
            state
                .event_tx
                .send(super::Event::Egui(egui::Event::Touch {
                    device_id: egui::TouchDeviceId(0),
                    id: egui::TouchId::from(touch.identifier()),
                    phase: egui::TouchPhase::Cancel,
                    pos: egui::pos2(
                        touch.client_x() as f32 / state.zoom_factor,
                        touch.client_y() as f32 / state.zoom_factor,
                    ),
                    force: Some(touch.force()),
                }))
                .unwrap();
        }
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "touchcancel", listener)?;

    // Register event listener for scrolling
    let listener = move |state: &mut super::MainState, event: web_sys::WheelEvent| {
        let unit = match event.delta_mode() {
            web_sys::WheelEvent::DOM_DELTA_LINE => egui::MouseWheelUnit::Line,
            web_sys::WheelEvent::DOM_DELTA_PAGE => egui::MouseWheelUnit::Page,
            _ => egui::MouseWheelUnit::Point,
        };
        let delta = -egui::vec2(event.delta_x() as f32, event.delta_y() as f32);
        let _ = state
            .event_tx
            .send(super::Event::Egui(egui::Event::MouseWheel {
                unit,
                delta,
                modifiers: event.egui_modifiers(),
            }));
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &canvas, "wheel", listener)?;

    // Register event listener for keyboard key presses and releases
    let listener_builder = |pressed| {
        move |state: &mut super::MainState, event: web_sys::KeyboardEvent| {
            // TODO: handle input method editors
            let modifiers = event.egui_modifiers();
            state
                .event_tx
                .send(super::Event::Modifiers(modifiers))
                .unwrap();
            let key = event.key();
            if pressed && !modifiers.command && key.len() == 1 {
                state
                    .event_tx
                    .send(super::Event::Egui(egui::Event::Text(key.clone())))
                    .unwrap();
            }
            if pressed && (event.is_composing() || event.key_code() == 229) {
                return;
            }
            if let Some(key) = egui::Key::from_name(&key) {
                state
                    .event_tx
                    .send(super::Event::Egui(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    }))
                    .unwrap();
                if pressed
                    && (matches!(
                        key,
                        egui::Key::Tab
                            | egui::Key::Backspace
                            | egui::Key::ArrowDown
                            | egui::Key::ArrowLeft
                            | egui::Key::ArrowRight
                            | egui::Key::ArrowUp
                    ) || (modifiers.command
                        && matches!(
                            key,
                            egui::Key::P | egui::Key::S | egui::Key::O | egui::Key::F
                        )))
                {
                    event.stop_propagation();
                    event.prevent_default();
                }
            }
            if !pressed && key == "Control" || key == "Meta" {
                // https://github.com/emilk/egui/issues/4724
                state.event_tx.send(super::Event::ReleaseAllKeys).unwrap();
            }
        }
    };
    add_event_listener(
        state_cell.clone(),
        &document,
        "keydown",
        listener_builder(true),
    )?;
    add_event_listener(
        state_cell.clone(),
        &document,
        "keyup",
        listener_builder(false),
    )?;

    // Register event listener for text pasting
    let listener = move |state: &mut super::MainState, event: web_sys::ClipboardEvent| {
        if let Some(data) = event.clipboard_data() {
            if let Ok(text) = data.get_data("text") {
                if !text.is_empty() {
                    state
                        .event_tx
                        .send(super::Event::Egui(egui::Event::Paste(
                            text.replace("\r\n", "\n"),
                        )))
                        .unwrap();
                }
            }
        }
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "paste", listener)?;

    // Register event listener for text copying
    let listener = move |state: &mut super::MainState, event: web_sys::ClipboardEvent| {
        state
            .event_tx
            .send(super::Event::Egui(egui::Event::Copy))
            .unwrap();
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "copy", listener)?;

    // Register event listener for text cutting
    let listener = move |state: &mut super::MainState, event: web_sys::ClipboardEvent| {
        state
            .event_tx
            .send(super::Event::Egui(egui::Event::Cut))
            .unwrap();
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &document, "cut", listener)?;

    // Register event listener for window gaining/losing focus
    let listener_builder = |has_focus| {
        move |state: &mut super::MainState, _event: web_sys::Event| {
            state
                .event_tx
                .send(super::Event::Egui(egui::Event::WindowFocused(has_focus)))
                .unwrap();
            if !has_focus {
                state.event_tx.send(super::Event::Save).unwrap();
            }
        }
    };
    add_event_listener(
        state_cell.clone(),
        &document,
        "focus",
        listener_builder(true),
    )?;
    add_event_listener(
        state_cell.clone(),
        &document,
        "blur",
        listener_builder(false),
    )?;

    // Register event listener to intercept and cancel the context menu and print events so that
    // our other event listeners will be able to receive those instead
    let listener = move |_state: &mut super::MainState, event: web_sys::Event| {
        event.stop_propagation();
        event.prevent_default();
    };
    add_event_listener(state_cell.clone(), &canvas, "contextmenu", listener)?;
    add_event_listener(state_cell.clone(), &canvas, "afterprint", listener)?;

    // Set up handler for outputs sent from the worker thread
    wasm_bindgen_futures::spawn_local(async move {
        let body_style = window.document().unwrap().body().unwrap().style();

        loop {
            let output = output_rx.recv_async().await;

            if super::has_panicked() {
                break;
            }

            match output.unwrap() {
                super::Output::Egui {
                    output,
                    zoom_factor,
                    screen_reader_enabled,
                } => {
                    let _ = body_style.set_property(
                        "cursor",
                        match output.cursor_icon {
                            egui::CursorIcon::Default => "default",
                            egui::CursorIcon::None => "none",

                            egui::CursorIcon::ContextMenu => "context-menu",
                            egui::CursorIcon::Help => "help",
                            egui::CursorIcon::PointingHand => "pointer",
                            egui::CursorIcon::Progress => "progress",
                            egui::CursorIcon::Wait => "wait",

                            egui::CursorIcon::Cell => "cell",
                            egui::CursorIcon::Crosshair => "crosshair",
                            egui::CursorIcon::Text => "text",
                            egui::CursorIcon::VerticalText => "vertical-text",

                            egui::CursorIcon::Alias => "alias",
                            egui::CursorIcon::Copy => "copy",
                            egui::CursorIcon::Move => "move",
                            egui::CursorIcon::NoDrop => "no-drop",
                            egui::CursorIcon::NotAllowed => "not-allowed",
                            egui::CursorIcon::Grab => "grab",
                            egui::CursorIcon::Grabbing => "grabbing",

                            egui::CursorIcon::AllScroll => "all-scroll",
                            egui::CursorIcon::ResizeColumn => "col-resize",
                            egui::CursorIcon::ResizeRow => "row-resize",
                            egui::CursorIcon::ResizeNorth => "n-resize",
                            egui::CursorIcon::ResizeEast => "e-resize",
                            egui::CursorIcon::ResizeSouth => "s-resize",
                            egui::CursorIcon::ResizeWest => "w-resize",
                            egui::CursorIcon::ResizeNorthEast => "ne-resize",
                            egui::CursorIcon::ResizeNorthWest => "nw-resize",
                            egui::CursorIcon::ResizeSouthEast => "se-resize",
                            egui::CursorIcon::ResizeSouthWest => "sw-resize",
                            egui::CursorIcon::ResizeHorizontal => "ew-resize",
                            egui::CursorIcon::ResizeVertical => "ns-resize",
                            egui::CursorIcon::ResizeNwSe => "nwse-resize",
                            egui::CursorIcon::ResizeNeSw => "nesw-resize",

                            egui::CursorIcon::ZoomIn => "zoom-in",
                            egui::CursorIcon::ZoomOut => "zoom-out",
                        },
                    );

                    // TODO: handle input method editors and screen readers here
                    let _ = screen_reader_enabled;

                    if !output.copied_text.is_empty() {
                        if let Err(e) = wasm_bindgen_futures::JsFuture::from(
                            window
                                .navigator()
                                .clipboard()
                                .unwrap()
                                .write_text(&output.copied_text),
                        )
                        .await
                        {
                            tracing::warn!(
                                "Failed to copy to clipboard: {}",
                                e.unchecked_into::<js_sys::Error>().to_string()
                            );
                        }
                    }

                    if let Some(url) = output.open_url {
                        if let Err(e) = window.open_with_url_and_target(
                            &url.url,
                            if url.new_tab { "_blank" } else { "_self" },
                        ) {
                            tracing::warn!(
                                "Failed to open URL: {}",
                                e.unchecked_into::<js_sys::Error>().to_string()
                            );
                        }
                    }

                    state_cell.borrow_mut().zoom_factor = zoom_factor;
                }

                super::Output::StorageGet { key, oneshot_tx } => {
                    let _ = oneshot_tx.send(
                        async {
                            luminol_web::idb(
                                "eframe.storage",
                                luminol_web::IdbTransactionMode::Readonly,
                                |store| store.get_owned(key),
                            )
                            .await
                            .ok()?
                            .await
                            .ok()??
                            .as_string()
                        }
                        .await,
                    );
                }

                super::Output::StorageSet {
                    key,
                    value,
                    oneshot_tx,
                } => {
                    let _ = oneshot_tx.send(
                        luminol_web::idb(
                            "eframe.storage",
                            luminol_web::IdbTransactionMode::Readwrite,
                            |store| store.put_key_val_owned(key, &js_sys::JsString::from(value)),
                        )
                        .await
                        .is_ok(),
                    );
                }
            }
        }
    });

    Ok(())
}
