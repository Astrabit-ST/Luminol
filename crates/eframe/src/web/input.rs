use super::{canvas_content_rect, AppRunner, WebRunnerEvent};

pub fn pos_from_mouse_event(
    canvas: &web_sys::HtmlCanvasElement,
    event: &web_sys::MouseEvent,
    zoom_factor: f32,
) -> egui::Pos2 {
    let rect = canvas_content_rect(canvas);
    egui::Pos2 {
        x: (event.client_x() as f32 - rect.left()) / zoom_factor,
        y: (event.client_y() as f32 - rect.top()) / zoom_factor,
    }
}

pub fn button_from_mouse_event(event: &web_sys::MouseEvent) -> Option<egui::PointerButton> {
    match event.button() {
        0 => Some(egui::PointerButton::Primary),
        1 => Some(egui::PointerButton::Middle),
        2 => Some(egui::PointerButton::Secondary),
        3 => Some(egui::PointerButton::Extra1),
        4 => Some(egui::PointerButton::Extra2),
        _ => None,
    }
}

/// A single touch is translated to a pointer movement. When a second touch is added, the pointer
/// should not jump to a different position. Therefore, we do not calculate the average position
/// of all touches, but we keep using the same touch as long as it is available.
pub fn primary_touch_pos(
    state: &super::MainState,
    event: &web_sys::TouchEvent,
    zoom_factor: f32,
) -> Option<(egui::Pos2, web_sys::Touch)> {
    let all_touches: Vec<_> = (0..event.touches().length())
        .filter_map(|i| event.touches().get(i))
        // On touchend we don't get anything in `touches`, but we still get `changed_touches`, so include those:
        .chain((0..event.changed_touches().length()).filter_map(|i| event.changed_touches().get(i)))
        .collect();

    let mut inner = state.inner.borrow_mut();

    if let Some(primary_touch) = inner.touch_id {
        // Is the primary touch is gone?
        if !all_touches
            .iter()
            .any(|touch| primary_touch == egui::TouchId::from(touch.identifier()))
        {
            inner.touch_id = None;
            state
                .channels
                .send_custom(WebRunnerEvent::Touch(inner.touch_id));
        }
    }

    if inner.touch_id.is_none() {
        inner.touch_id = all_touches
            .first()
            .map(|touch| egui::TouchId::from(touch.identifier()));
        state
            .channels
            .send_custom(WebRunnerEvent::Touch(inner.touch_id));
    }

    let primary_touch = inner.touch_id;

    if let Some(primary_touch) = primary_touch {
        for touch in all_touches {
            if primary_touch == egui::TouchId::from(touch.identifier()) {
                let canvas_rect = canvas_content_rect(&state.canvas);
                return Some((pos_from_touch(canvas_rect, &touch, zoom_factor), touch));
            }
        }
    }

    None
}

fn pos_from_touch(canvas_rect: egui::Rect, touch: &web_sys::Touch, zoom_factor: f32) -> egui::Pos2 {
    egui::Pos2 {
        x: (touch.client_x() as f32 - canvas_rect.left()) / zoom_factor,
        y: (touch.client_y() as f32 - canvas_rect.top()) / zoom_factor,
    }
}

pub fn push_touches(
    state: &super::MainState,
    phase: egui::TouchPhase,
    event: &web_sys::TouchEvent,
) {
    let canvas_rect = canvas_content_rect(&state.canvas);
    for touch_idx in 0..event.changed_touches().length() {
        if let Some(touch) = event.changed_touches().item(touch_idx) {
            state.channels.send(egui::Event::Touch {
                device_id: egui::TouchDeviceId(0),
                id: egui::TouchId::from(touch.identifier()),
                phase,
                pos: pos_from_touch(canvas_rect, &touch, state.channels.zoom_factor()),
                force: Some(touch.force()),
            });
        }
    }
}

/// The text input from a keyboard event (e.g. `X` when pressing the `X` key).
pub fn text_from_keyboard_event(event: &web_sys::KeyboardEvent) -> Option<String> {
    let key = event.key();

    let is_function_key = key.starts_with('F') && key.len() > 1;
    if is_function_key {
        return None;
    }

    let is_control_key = matches!(
        key.as_str(),
        "Alt"
      | "ArrowDown"
      | "ArrowLeft"
      | "ArrowRight"
      | "ArrowUp"
      | "Backspace"
      | "CapsLock"
      | "ContextMenu"
      | "Control"
      | "Delete"
      | "End"
      | "Enter"
      | "Esc"
      | "Escape"
      | "GroupNext" // https://github.com/emilk/egui/issues/510
      | "Help"
      | "Home"
      | "Insert"
      | "Meta"
      | "NumLock"
      | "PageDown"
      | "PageUp"
      | "Pause"
      | "ScrollLock"
      | "Shift"
      | "Tab"
    );

    if is_control_key {
        return None;
    }

    Some(key)
}

/// Web sends all keys as strings, so it is up to us to figure out if it is
/// a real text input or the name of a key.
pub fn translate_key(key: &str) -> Option<egui::Key> {
    egui::Key::from_name(key)
}

macro_rules! modifiers {
    ($event:ident) => {
        egui::Modifiers {
            alt: $event.alt_key(),
            ctrl: $event.ctrl_key(),
            shift: $event.shift_key(),

            // Ideally we should know if we are running or mac or not,
            // but this works good enough for now.
            mac_cmd: $event.meta_key(),

            // Ideally we should know if we are running or mac or not,
            // but this works good enough for now.
            command: $event.ctrl_key() || $event.meta_key(),
        }
    };
}

pub fn modifiers_from_kb_event(event: &web_sys::KeyboardEvent) -> egui::Modifiers {
    modifiers!(event)
}

pub fn modifiers_from_mouse_event(event: &web_sys::MouseEvent) -> egui::Modifiers {
    modifiers!(event)
}

pub(super) fn modifiers_from_wheel_event(event: &web_sys::WheelEvent) -> egui::Modifiers {
    modifiers!(event)
}

pub(super) fn modifiers_from_touch_event(event: &web_sys::TouchEvent) -> egui::Modifiers {
    modifiers!(event)
}
