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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use crate::lumi::Lumi;
#[cfg(feature = "steamworks")]
use crate::steam::Steamworks;

mod top_bar;

/// Custom implementation of `eframe::Frame` for Luminol.
/// We need this because the normal `eframe::App` uses a struct with private fields in its
/// definition of `update()`, and that prevents us from implementing custom app runners.
pub struct CustomFrame<'a>(
    #[cfg(not(target_arch = "wasm32"))] pub &'a mut eframe::Frame,
    #[cfg(target_arch = "wasm32")] pub std::marker::PhantomData<&'a ()>,
);

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::Deref for CustomFrame<'_> {
    type Target = eframe::Frame;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl std::ops::DerefMut for CustomFrame<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

/// Custom implementation of `eframe::App` for Luminol.
/// We need this because the normal `eframe::App` uses a struct with private fields in its
/// definition of `update()`, and that prevents us from implementing custom app runners.
pub trait CustomApp
where
    Self: eframe::App,
{
    fn custom_update(&mut self, ctx: &egui::Context, frame: &mut CustomFrame<'_>);
}

#[macro_export]
macro_rules! app_use_custom_update {
    () => {
        fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
            #[cfg(not(target_arch = "wasm32"))]
            self.custom_update(ctx, &mut CustomFrame(frame))
        }
    };
}

/// The main Luminol struct. Handles rendering, GUI state, that sort of thing.
pub struct App {
    top_bar: top_bar::TopBar,
    lumi: Lumi,

    audio: luminol_audio::Audio,

    graphics: std::sync::Arc<luminol_graphics::GraphicsState>,
    filesystem: luminol_filesystem::project::FileSystem,
    data: luminol_core::Data,

    toasts: luminol_core::Toasts,

    windows: luminol_core::Windows,
    tabs: luminol_core::Tabs,

    global_config: luminol_config::global::Config,
    project_config: Option<luminol_config::project::Config>,

    toolbar: luminol_core::ToolbarState,

    #[cfg(feature = "steamworks")]
    steamworks: Steamworks,
}

impl App {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        try_load_path: Option<std::ffi::OsString>,
        #[cfg(target_arch = "wasm32")] audio_wrapper: crate::audio::AudioWrapper,
        #[cfg(feature = "steamworks")] steamworks: Steamworks,
    ) -> Self {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("wgpu backend not enabled");
        let graphics = std::sync::Arc::new(luminol_graphics::GraphicsState::new(render_state));

        let storage = cc.storage.unwrap();

        let mut global_config = eframe::get_value(storage, "SavedState").unwrap_or_default();
        let mut project_config = None;

        let mut filesystem = luminol_filesystem::project::FileSystem::new();
        let mut data = luminol_core::Data::default();

        let mut toasts = luminol_core::Toasts::default();

        #[cfg(not(target_arch = "wasm32"))]
        if let Some(path) = try_load_path {
            let path = camino::Utf8PathBuf::from_path_buf(std::path::PathBuf::from(path))
                .expect("project path not utf-8");

            match filesystem.load_project_from_path(&mut project_config, &mut global_config, path) {
                Ok(_) => {
                    if let Err(e) = data.load(&filesystem, project_config.as_mut().unwrap()) {
                        toasts.error(e.to_string())
                    }
                }
                Err(e) => toasts.error(e.to_string()),
            }
        }

        let style =
            eframe::get_value(storage, "EguiStyle").map_or_else(|| cc.egui_ctx.style(), |s| s);
        cc.egui_ctx.set_style(style.clone());

        #[cfg(not(debug_assertions))]
        state!()
            .render_state
            .device
            .on_uncaptured_error(Box::new(|e| {
                use std::fmt::Write;

                let mut message_description = String::new();
                match e {
                    wgpu::Error::OutOfMemory { source } => {
                        message_description.push_str("wgpu error: Out of memory\n");
                        writeln!(message_description, "{source:#?}").unwrap();
                    }
                    wgpu::Error::Validation {
                        source,
                        description,
                    } => {
                        message_description.push_str("wgpu error: Validation error\n");
                        writeln!(message_description, "{source}").unwrap();
                        writeln!(message_description, "---------").unwrap();
                        writeln!(message_description, "{}", source.source().unwrap()).unwrap();
                        writeln!(message_description, "---------").unwrap();
                        writeln!(message_description, "{source:#?}").unwrap();
                        writeln!(message_description, "---------").unwrap();
                        message_description.push_str(&description);
                    }
                }
                rfd::MessageDialog::new()
                    .set_title("Luminol has crashed!")
                    .set_level(rfd::MessageLevel::Error)
                    .set_description(&message_description)
                    .show();

                let backtrace = std::backtrace::Backtrace::force_capture();
                rfd::MessageDialog::new()
                    .set_title("Backtrace")
                    .set_level(rfd::MessageLevel::Error)
                    .set_description(&backtrace.to_string())
                    .show();

                std::process::abort();
            }));

        let lumi = Lumi::new().expect("failed to load lumi images");

        Self {
            top_bar: top_bar::TopBar::default(),
            lumi,

            audio: luminol_audio::Audio::default(),
            graphics,
            filesystem,
            data,
            toasts,
            windows: luminol_core::Windows::default(),
            tabs: luminol_core::Tabs::new_with_tabs(
                "luminol_main_tabs",
                vec![luminol_ui::tabs::started::Tab::default()],
            ),
            global_config,
            project_config,
            toolbar: luminol_core::ToolbarState::default(),

            #[cfg(feature = "steamworks")]
            steamworks,
        }
    }
}

impl CustomApp for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn custom_update(&mut self, ctx: &eframe::egui::Context, frame: &mut CustomFrame<'_>) {
        ctx.input(|i| {
            if let Some(f) = i.raw.dropped_files.first() {
                let path = f.path.clone().expect("dropped file has no path");
                let path = camino::Utf8PathBuf::from_path_buf(path).expect("path was not utf8");

                #[cfg(not(target_arch = "wasm32"))]
                if let Err(e) = self.filesystem.load_project_from_path(
                    &mut self.project_config,
                    &mut self.global_config,
                    path,
                ) {
                    self.toasts
                        .error(format!("Error opening dropped project: {e}"))
                } else {
                    self.toasts.info(format!(
                        "Successfully opened {:?}",
                        self.filesystem.project_path().expect("project not open")
                    ));
                }
            }
        });

        let mut edit_windows = luminol_core::EditWindows::default();
        let mut edit_tabs = luminol_core::EditTabs::default();
        let mut update_state = luminol_core::UpdateState {
            audio: &mut self.audio,
            graphics: self.graphics.clone(),
            filesystem: &mut self.filesystem,
            data: &mut self.data,
            edit_windows: &mut edit_windows,
            edit_tabs: &mut edit_tabs,
            toasts: &mut self.toasts,
            project_config: &mut self.project_config,
            global_config: &mut self.global_config,
            toolbar: &mut self.toolbar,
        };

        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(ui, frame, &mut update_state);
            });
        });

        // Central panel with tabs.
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                ui.group(|ui| self.tabs.ui_without_edit(ui, &mut update_state));
            });

        // Update all windows.
        self.windows.display_without_edit(ctx, &mut update_state);

        // If we don't do this tabs added by windows won't be added.
        // It also cleans up code nicely.
        self.tabs.process_edit_tabs(edit_tabs);
        self.windows.process_edit_windows(edit_windows);

        // Show toasts.
        self.toasts.show(ctx);

        #[cfg(not(target_arch = "wasm32"))]
        poll_promise::tick_local();

        self.lumi.ui(ctx);

        #[cfg(feature = "steamworks")]
        self.steamworks.update()
    }
}

impl eframe::App for App {
    app_use_custom_update!();

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "SavedState", &self.global_config);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
