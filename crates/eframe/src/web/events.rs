use super::*;

// ------------------------------------------------------------------------

/// Calls `request_animation_frame` to schedule repaint.
///
/// It will only paint if needed, but will always call `request_animation_frame` immediately.
fn paint_and_schedule(runner_ref: &WebRunner) -> Result<(), JsValue> {
    // Only paint and schedule if there has been no panic
    if let Some(mut runner_lock) = runner_ref.try_lock() {
        paint_if_needed(&mut runner_lock)?;
        drop(runner_lock);
        request_animation_frame(runner_ref.clone())?;
    }

    Ok(())
}

fn paint_if_needed(runner: &mut AppRunner) -> Result<(), JsValue> {
    if runner.needs_repaint.when_to_repaint() <= now_sec() {
        runner.needs_repaint.clear();
        let (repaint_after, clipped_primitives) = runner.logic();
        runner.paint(&clipped_primitives)?;
        runner
            .needs_repaint
            .repaint_after(repaint_after.as_secs_f64());
        runner.auto_save_if_needed();
    }
    Ok(())
}

pub(crate) fn request_animation_frame(runner_ref: WebRunner) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();
    let closure = Closure::once(move || paint_and_schedule(&runner_ref));
    window.request_animation_frame(closure.as_ref().unchecked_ref())?;
    closure.forget(); // We must forget it, or else the callback is canceled on drop
    Ok(())
}

// ------------------------------------------------------------------------

pub(crate) fn install_document_events(runner_ref: &WebRunner) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();

    {
        // Avoid sticky modifier keys on alt-tab:
        for event_name in ["blur", "focus"] {
            let closure = move |_event: web_sys::MouseEvent, runner: &mut AppRunner| {
                let has_focus = event_name == "focus";

                if !has_focus {
                    // We lost focus - good idea to save
                    runner.save();
                }

                //runner.input.on_web_page_focus_change(has_focus);
                runner.egui_ctx().request_repaint();
                // log::debug!("{event_name:?}");
            };

            runner_ref.add_event_listener(&document, event_name, closure)?;
        }
    }

    runner_ref.add_event_listener(
        &document,
        "keydown",
        |event: web_sys::KeyboardEvent, runner| {
            if event.is_composing() || event.key_code() == 229 {
                // https://web.archive.org/web/20200526195704/https://www.fxsitecompat.dev/en-CA/docs/2018/keydown-and-keyup-events-are-now-fired-during-ime-composition/
                return;
            }

            let modifiers = modifiers_from_event(&event);
            runner.input.raw.modifiers = modifiers;

            let key = event.key();
            let egui_key = translate_key(&key);

            if let Some(key) = egui_key {
                runner.input.raw.events.push(egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false, // egui will fill this in for us!
                    modifiers,
                });
            }
            if !modifiers.ctrl
                && !modifiers.command
                && !should_ignore_key(&key)
                // When text agent is shown, it sends text event instead.
                && text_agent::text_agent().hidden()
            {
                runner.input.raw.events.push(egui::Event::Text(key));
            }
            runner.needs_repaint.repaint_asap();

            let egui_wants_keyboard = runner.egui_ctx().wants_keyboard_input();

            #[allow(clippy::if_same_then_else)]
            let prevent_default = if egui_key == Some(egui::Key::Tab) {
                // Always prevent moving cursor to url bar.
                // egui wants to use tab to move to the next text field.
                true
            } else if egui_key == Some(egui::Key::P) {
                #[allow(clippy::needless_bool)]
                if modifiers.ctrl || modifiers.command || modifiers.mac_cmd {
                    true // Prevent ctrl-P opening the print dialog. Users may want to use it for a command palette.
                } else {
                    false // let normal P:s through
                }
            } else if egui_wants_keyboard {
                matches!(
                    event.key().as_str(),
                    "Backspace" // so we don't go back to previous page when deleting text
                    | "ArrowDown" | "ArrowLeft" | "ArrowRight" | "ArrowUp" // cmd-left is "back" on Mac (https://github.com/emilk/egui/issues/58)
                )
            } else {
                // We never want to prevent:
                // * F5 / cmd-R (refresh)
                // * cmd-shift-C (debug tools)
                // * cmd/ctrl-c/v/x (or we stop copy/past/cut events)
                false
            };

            // log::debug!(
            //     "On key-down {:?}, egui_wants_keyboard: {}, prevent_default: {}",
            //     event.key().as_str(),
            //     egui_wants_keyboard,
            //     prevent_default
            // );

            if prevent_default {
                event.prevent_default();
                // event.stop_propagation();
            }
        },
    )?;

    runner_ref.add_event_listener(
        &document,
        "keyup",
        |event: web_sys::KeyboardEvent, runner| {
            let modifiers = modifiers_from_event(&event);
            runner.input.raw.modifiers = modifiers;
            if let Some(key) = translate_key(&event.key()) {
                runner.input.raw.events.push(egui::Event::Key {
                    key,
                    pressed: false,
                    repeat: false,
                    modifiers,
                });
            }
            runner.needs_repaint.repaint_asap();
        },
    )?;

    #[cfg(web_sys_unstable_apis)]
    runner_ref.add_event_listener(
        &document,
        "paste",
        |event: web_sys::ClipboardEvent, runner| {
            if let Some(data) = event.clipboard_data() {
                if let Ok(text) = data.get_data("text") {
                    let text = text.replace("\r\n", "\n");
                    if !text.is_empty() {
                        runner.input.raw.events.push(egui::Event::Paste(text));
                        runner.needs_repaint.repaint_asap();
                    }
                    event.stop_propagation();
                    event.prevent_default();
                }
            }
        },
    )?;

    #[cfg(web_sys_unstable_apis)]
    runner_ref.add_event_listener(&document, "cut", |_: web_sys::ClipboardEvent, runner| {
        runner.input.raw.events.push(egui::Event::Cut);
        runner.needs_repaint.repaint_asap();
    })?;

    #[cfg(web_sys_unstable_apis)]
    runner_ref.add_event_listener(&document, "copy", |_: web_sys::ClipboardEvent, runner| {
        runner.input.raw.events.push(egui::Event::Copy);
        runner.needs_repaint.repaint_asap();
    })?;

    Ok(())
}

pub(crate) fn install_window_events(runner_ref: &WebRunner) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();

    // Save-on-close
    runner_ref.add_event_listener(&window, "onbeforeunload", |_: web_sys::Event, runner| {
        runner.save();
    })?;

    for event_name in &["load", "pagehide", "pageshow", "resize"] {
        runner_ref.add_event_listener(&window, event_name, |_: web_sys::Event, runner| {
            runner.needs_repaint.repaint_asap();
        })?;
    }

    runner_ref.add_event_listener(&window, "hashchange", |_: web_sys::Event, runner| {
        // `epi::Frame::info(&self)` clones `epi::IntegrationInfo`, but we need to modify the original here
        runner.frame.info.web_info.location.hash = location_hash();
    })?;

    Ok(())
}

pub(crate) fn install_color_scheme_change_event(runner_ref: &WebRunner) -> Result<(), JsValue> {
    let window = web_sys::window().unwrap();

    if let Some(media_query_list) = prefers_color_scheme_dark(&window)? {
        runner_ref.add_event_listener::<web_sys::MediaQueryListEvent>(
            &media_query_list,
            "change",
            |event, runner| {
                let theme = theme_from_dark_mode(event.matches());
                runner.frame.info.system_theme = Some(theme);
                runner.egui_ctx().set_visuals(theme.egui_visuals());
                runner.needs_repaint.repaint_asap();
            },
        )?;
    }

    Ok(())
}

pub(crate) fn install_canvas_events(runner_ref: &WebRunner) -> Result<(), JsValue> {
    todo!();
}
