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

use wasm_bindgen::JsCast;

mod events;
mod worker;

// ensure that AtomicF64 is using atomic ops (otherwise it would use global locks, and that would be bad)
const _: [(); 0 - !{
    const ASSERT: bool = portable_atomic::AtomicBool::is_always_lock_free()
        && portable_atomic::AtomicF64::is_always_lock_free();
    ASSERT
} as usize] = [];

static PANIC_HOOK_INSTALLED: portable_atomic::AtomicBool = portable_atomic::AtomicBool::new(false);
static HAS_PANICKED: portable_atomic::AtomicBool = portable_atomic::AtomicBool::new(false);

fn has_panicked() -> bool {
    HAS_PANICKED.load(portable_atomic::Ordering::Acquire)
}

/// Commands that can be emitted by the main thread that the worker thread needs to respond to
enum Event {
    /// Miscellaneous egui events
    Egui(egui::Event),
    /// This should be sent whenever the browser window's interior size or resolution changes
    ScreenResize {
        /// New interior width of the window in points
        inner_width: u32,
        /// New interior height of the window in points
        inner_height: u32,
        /// New length of a pixel divided by new length of a point
        device_pixel_ratio: f32,
    },
    /// This should be sent whenever a modifier key is pressed or released
    Modifiers(egui::Modifiers),
    /// The browser detected a touchstart or touchmove event, with or without a touch ID
    Touch(Option<egui::TouchId>),
    /// This should be sent whenever the app needs to save immediately
    Save,
    /// This should be sent whenever the app needs to release all held keyboard keys
    ReleaseAllKeys,
}

/// Commands that can be emitted by the worker thread that the main thread needs to respond to
enum Output {
    /// Miscellaneous egui output events
    Egui {
        /// The inner egui output event
        output: egui::PlatformOutput,
        /// The current egui zoom factor (`ctx.zoom_factor()`)
        zoom_factor: f32,
        /// Whether or not the screen reader was enabled in the egui context when the egui output
        /// event was emitted (`ctx.options(|o| o.screen_reader)`)
        screen_reader_enabled: bool,
    },
    /// The runner wants to read a key from storage
    StorageGet {
        /// The key to read from storage
        key: String,
        /// The sender that will be used to send the read value back to the runner
        oneshot_tx: oneshot::Sender<Option<String>>,
    },
    /// The runner wants to write a key to storage
    StorageSet {
        /// The key to write to storage
        key: String,
        /// The value to write to storage at the given key
        value: String,
        /// The sender that will be used to send the result (true if the write succeeded, false if
        /// the write failed) back to the runner
        oneshot_tx: oneshot::Sender<bool>,
    },
}

/// The halves of the web runner's channels that belong to the worker thread
struct WorkerChannels {
    /// The receiver used to receive egui events from the main thread
    event_rx: flume::Receiver<Event>,
    /// The sender used to send outputs to the main thread
    output_tx: flume::Sender<Output>,
    /// The sender used to inform the main thread when a panic occurs
    panic_tx: Option<oneshot::Sender<()>>,
}

/// Handle to a key-value store where persistent data can be stored by the web runner
#[derive(Clone)]
struct Storage(flume::Sender<Output>);

impl eframe::Storage for Storage {
    fn get_string(&self, key: &str) -> Option<String> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.0
            .send(Output::StorageGet {
                key: key.to_string(),
                oneshot_tx,
            })
            .unwrap();
        oneshot_rx.recv().unwrap()
    }

    fn set_string(&mut self, key: &str, value: String) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.0
            .send(Output::StorageSet {
                key: key.to_string(),
                value: value.to_string(),
                oneshot_tx,
            })
            .unwrap();
        if !oneshot_rx.recv().unwrap() {
            tracing::warn!("Failed to save to local storage key {key}");
        }
    }

    fn flush(&mut self) {}
}

/// Custom egui web runner for Luminol that runs an egui app in a worker thread
pub struct Runner {
    channels: WorkerChannels,
    prefers_color_scheme_dark: Option<bool>,
}

/// State of the web runner that belongs to the main thread
struct MainState {
    /// The sender used to send egui events to the worker thread
    event_tx: flume::Sender<Event>,
    /// The HTML canvas element that the web runner is rendering to
    canvas: web_sys::HtmlCanvasElement,
    /// The HTML input element that handles input method editors and mobile keyboards
    input: web_sys::HtmlInputElement,
    /// Value of the `ime` field of the egui platform output from the previous frame
    ime: Option<egui::output::IMEOutput>,
    /// The current egui zoom factor (`ctx.zoom_factor()`)
    zoom_factor: f32,
    /// JavaScript touch ID currently being tracked by the web runner
    touch_id: Option<egui::TouchId>,
}

/// State of the web runner that belongs to the worker thread
struct WorkerState {
    /// The egui app that the web runner is running
    app: Box<dyn crate::app::AppTrait>,
    /// A unique name for the egui app that the web runner is running, for purposes of making sure
    /// that different egui apps get their own storage in IndexedDB
    app_id: &'static str,
    /// A handle to the web worker that this part of the web runner is running in
    worker: web_sys::DedicatedWorkerGlobalScope,
    /// Channels used to send data to and receive data from the main thread
    channels: WorkerChannels,
    /// The JavaScript `OffscreenCanvas` that the web runner is rendering to
    canvas: web_sys::OffscreenCanvas,
    /// The egui context of the egui app that the web runner is running
    context: egui::Context,
    /// A handle to the persistent storage used by the egui app
    storage: Storage,
    /// The closure that will be called by the most recent call to `requestAnimationFrame()`
    runner_worker_closure: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
    /// The wgpu surface that the web runner is rendering to
    surface: wgpu::Surface<'static>,
    /// The configuration for the wgpu surface that the web runner is rendering to
    surface_configuration: wgpu::SurfaceConfiguration,
    /// The egui render state
    render_state: egui_wgpu::RenderState,
    /// Current width of the canvas in points
    width: u32,
    /// Current height of the canvas in points
    height: u32,
    /// Current length of a pixel divided by current length of a point
    native_pixels_per_point: f32,
    /// The touch ID of the most recent touch event received by the main thread
    touch: Option<egui::TouchId>,
    /// The input data that will be provided to the egui app the next time its `update()` function
    /// is called
    input: egui::RawInput,
    /// The next time in seconds since application startup the egui app should repaint
    repaint_time: std::sync::Arc<portable_atomic::AtomicF64>,
    /// Last time in seconds since application startup the egui app saved its persistent data
    save_time: f64,
}

type AppCreator = dyn FnOnce(&eframe::CreationContext<'_>) -> Box<dyn crate::app::AppTrait>;

impl Runner {
    pub fn new(canvas: web_sys::HtmlCanvasElement) -> Result<Self, wasm_bindgen::JsValue> {
        let Some(window) = web_sys::window() else {
            panic!("cannot create a `luminol_launcher::Runner` outside of the main thread");
        };
        let Some(document) = window.document() else {
            panic!("cannot create a `luminol_launcher::Runner` outside of the main thread");
        };
        let Some(body) = document.body() else {
            panic!("cannot create a `luminol_launcher::Runner` outside of the main thread");
        };

        // Install a hook to set `HAS_PANICKED` to true when a panic occurs on any thread
        if !PANIC_HOOK_INSTALLED.load(portable_atomic::Ordering::Relaxed) {
            let old_panic_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                HAS_PANICKED.store(true, portable_atomic::Ordering::Release);
                old_panic_hook(info);
            }));
            if let Some(input) = document.get_element_by_id("luminol-ime") {
                input.remove();
            }
            PANIC_HOOK_INSTALLED.store(true, portable_atomic::Ordering::Relaxed);
        }

        let prefers_color_scheme_dark = window
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten()
            .map(|query| query.matches());

        // Initialize handler for input method editors
        let input = if let Some(input) = document.get_element_by_id("luminol-ime") {
            input.dyn_into()?
        } else {
            let input = document
                .create_element("input")?
                .unchecked_into::<web_sys::HtmlInputElement>();
            input.set_id("luminol-ime");
            input.set_type("text");
            let style = input.style();
            style.set_property("opacity", "0")?;
            style.set_property("width", "1px")?;
            style.set_property("height", "1px")?;
            style.set_property("position", "absolute")?;
            style.set_property("top", "0")?;
            style.set_property("left", "0")?;
            body.append_child(&input)?;
            input
        };

        let (event_tx, event_rx) = flume::unbounded();
        let (output_tx, output_rx) = flume::unbounded();
        let (panic_tx, panic_rx) = oneshot::channel();

        events::register_events(
            MainState {
                event_tx,
                canvas,
                input,
                ime: None,
                zoom_factor: 1.,
                touch_id: None,
            },
            output_rx,
            panic_rx,
            window,
            document,
        )?;

        Ok(Self {
            channels: WorkerChannels {
                event_rx,
                output_tx,
                panic_tx: Some(panic_tx),
            },
            prefers_color_scheme_dark,
        })
    }

    pub async fn run(
        self,
        app_creator: Box<AppCreator>,
        app_id: &'static str,
        canvas: web_sys::OffscreenCanvas,
        web_options: eframe::WebOptions,
    ) -> Result<(), egui_wgpu::WgpuError> {
        let Some(worker) = luminol_web::bindings::worker() else {
            panic!(
                "cannot use `luminol_launcher::Runner::run()` outside of a dedicated web worker"
            );
        };

        // We set the time of the last repaint to negative infinity so that a repaint will always
        // occur on the first frame after the web runner starts up
        let repaint_time = std::sync::Arc::new(portable_atomic::AtomicF64::new(f64::NEG_INFINITY));

        #[allow(clippy::arc_with_non_send_sync)]
        let instance = match web_options.wgpu_options.wgpu_setup {
            egui_wgpu::WgpuSetup::CreateNew {
                supported_backends,
                power_preference: _,
                device_descriptor: _,
            } => std::sync::Arc::new(wgpu::Instance::new(wgpu::InstanceDescriptor {
                backends: supported_backends,
                flags: wgpu::InstanceFlags::default(),
                dx12_shader_compiler: wgpu::Dx12Compiler::default(),
                gles_minor_version: wgpu::Gles3MinorVersion::default(),
            })),
            egui_wgpu::WgpuSetup::Existing {
                ref instance,
                adapter: _,
                device: _,
                queue: _,
            } => instance.clone(),
        };

        let surface =
            instance.create_surface(wgpu::SurfaceTarget::OffscreenCanvas(canvas.clone()))?;

        let render_state = egui_wgpu::RenderState::create(
            &web_options.wgpu_options,
            &instance,
            &surface,
            egui_wgpu::depth_format_from_bits(0, 0),
            1,
            web_options.dithering,
        )
        .await?;

        let location = worker.location();
        let integration_info = eframe::IntegrationInfo {
            web_info: eframe::WebInfo {
                user_agent: worker.navigator().user_agent().unwrap_or_default(),
                location: eframe::Location {
                    url: {
                        let href = location.href();
                        href.strip_suffix("/worker.js").unwrap_or(&href).to_string()
                    },
                    protocol: location.protocol(),
                    host: location.host(),
                    hostname: location.hostname(),
                    port: location.port(),
                    hash: String::new(),
                    query: String::new(),
                    query_map: Default::default(),
                    origin: location.origin(),
                },
            },
            cpu_usage: None,
        };

        let context = egui::Context::default();
        context.set_os(egui::os::OperatingSystem::from_user_agent(
            integration_info.web_info.user_agent.as_str(),
        ));
        {
            let repaint_time = repaint_time.clone();
            context.set_request_repaint_callback(move |repaint_info| {
                repaint_time.fetch_min(
                    if let Some(worker) = luminol_web::bindings::worker() {
                        worker.performance()
                    } else if let Some(window) = web_sys::window() {
                        window.performance()
                    } else {
                        panic!(
                            "can only request repaint from the main thread or a dedicated web worker"
                        );
                    }
                    .unwrap()
                    .now()
                        / 1000.
                        + repaint_info.delay.as_secs_f64(),
                    portable_atomic::Ordering::SeqCst,
                );
            });
        }

        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.channels
            .output_tx
            .send(Output::StorageGet {
                key: app_id.to_string(),
                oneshot_tx,
            })
            .unwrap();

        let is_new_memory = if let Some(memory) = oneshot_rx.await.ok().flatten() {
            match ron::from_str(&memory) {
                Ok(memory) => {
                    context.memory_mut(|m| *m = memory);
                    tracing::info!("Successfully restored memory for {app_id}");
                    false
                }
                Err(e) => {
                    tracing::warn!("Failed to restore memory for {app_id}: {e}");
                    true
                }
            }
        } else {
            tracing::warn!("No memory found for {app_id}");
            true
        };

        if is_new_memory {
            // Set default egui visuals depending on the user's dark mode preference
            if let Some(prefers_color_scheme_dark) = self.prefers_color_scheme_dark {
                context.set_visuals(
                    egui::Theme::from_dark_mode(prefers_color_scheme_dark).default_visuals(),
                );
            }
        }

        // Prevent Ctrl+Plus/Ctrl+Minus from changing egui's internal zoom factor
        context.options_mut(|o| o.zoom_with_keyboard = false);

        let storage = Storage(self.channels.output_tx.clone());

        worker::runner_worker(std::rc::Rc::new(std::cell::RefCell::new(WorkerState {
            app: app_creator(&eframe::CreationContext {
                egui_ctx: context.clone(),
                integration_info: integration_info.clone(),
                wgpu_render_state: Some(render_state.clone()),
                storage: Some(&storage),
            }),
            app_id,
            worker,
            channels: self.channels,
            canvas,
            context,
            storage,
            runner_worker_closure: None,
            surface,
            surface_configuration: wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: render_state.target_format,
                width: 0,
                height: 0,
                present_mode: web_options.wgpu_options.present_mode,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![render_state.target_format],
            },
            render_state,
            width: 0,
            height: 0,
            native_pixels_per_point: 1.,
            touch: None,
            input: egui::RawInput::default(),
            repaint_time,
            save_time: 0.,
        })));

        Ok(())
    }
}
