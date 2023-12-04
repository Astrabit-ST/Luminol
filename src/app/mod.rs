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

use std::sync::Arc;

use crate::lumi::Lumi;
#[cfg(feature = "steamworks")]
use crate::steam::Steamworks;

mod top_bar;

/// The main Luminol struct. Handles rendering, GUI state, that sort of thing.
pub struct App {
    top_bar: top_bar::TopBar,
    lumi: Lumi,

    #[cfg(not(target_arch = "wasm32"))]
    audio: luminol_audio::Audio,
    #[cfg(target_arch = "wasm32")]
    audio: luminol_audio::AudioWrapper,

    graphics: Arc<luminol_graphics::GraphicsState>,
    filesystem: luminol_filesystem::project::FileSystem,
    data: luminol_core::Data,
    bytes_loader: Arc<luminol_filesystem::egui_bytes_loader::Loader>,

    toasts: luminol_core::Toasts,

    windows: luminol_core::Windows,
    tabs: luminol_core::Tabs,

    global_config: luminol_config::global::Config,
    project_config: Option<luminol_config::project::Config>,

    toolbar: luminol_core::ToolbarState,

    modified: luminol_core::ModifiedState,

    #[cfg(not(target_arch = "wasm32"))]
    _runtime: tokio::runtime::Runtime,

    #[cfg(feature = "steamworks")]
    steamworks: Steamworks,
}

impl App {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(
        cc: &luminol_eframe::CreationContext<'_>,
        modified: luminol_core::ModifiedState,
        #[cfg(not(target_arch = "wasm32"))] try_load_path: Option<std::ffi::OsString>,
        #[cfg(target_arch = "wasm32")] audio: luminol_audio::AudioWrapper,
        #[cfg(feature = "steamworks")] steamworks: Steamworks,
    ) -> Self {
        let render_state = cc
            .wgpu_render_state
            .clone()
            .expect("wgpu backend not enabled");

        // Add custom fallback fonts for glyphs that egui's default font doesn't support
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            String::from("Source Han Sans Regular"),
            egui::FontData::from_owned(
                zstd::bulk::decompress(
                    include_bytes!("../../assets/SourceHanSans-Regular.ttc.zst"),
                    19485724,
                )
                .unwrap(),
            ),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .push("Source Han Sans Regular".to_owned());
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .push("Source Han Sans Regular".to_owned());
        cc.egui_ctx.set_fonts(fonts);

        #[cfg(not(debug_assertions))]
        render_state.device.on_uncaptured_error(Box::new(|e| {
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

        let graphics = std::sync::Arc::new(luminol_graphics::GraphicsState::new(render_state));

        egui_extras::install_image_loaders(&cc.egui_ctx);

        let storage = cc.storage.unwrap();

        let mut global_config =
            luminol_eframe::get_value(storage, "SavedState").unwrap_or_default();
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

        let style = luminol_eframe::get_value(storage, "EguiStyle")
            .map_or_else(|| cc.egui_ctx.style(), |s| s);
        cc.egui_ctx.set_style(style.clone());

        let bytes_loader = Arc::new(luminol_filesystem::egui_bytes_loader::Loader::new());
        cc.egui_ctx.add_bytes_loader(bytes_loader.clone());

        let lumi = Lumi::new().expect("failed to load lumi images");

        #[cfg(not(target_arch = "wasm32"))]
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2) // TODO use single threaded runtime
            .enable_all()
            .build()
            .expect("failed to build tokio runtime");
        //enter the runtime permanently
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::mem::forget(runtime.enter());
        }

        #[cfg(not(target_arch = "wasm32"))]
        let audio = luminol_audio::Audio::default();

        Self {
            top_bar: top_bar::TopBar::default(),
            lumi,

            audio,
            graphics,
            filesystem,
            data,
            bytes_loader,

            toasts,
            windows: luminol_core::Windows::default(),
            tabs: luminol_core::Tabs::new_with_tabs(
                "luminol_main_tabs",
                vec![luminol_ui::tabs::started::Tab::default()],
                true,
            ),
            global_config,
            project_config,
            toolbar: luminol_core::ToolbarState::default(),

            modified,

            #[cfg(not(target_arch = "wasm32"))]
            _runtime: runtime,

            #[cfg(feature = "steamworks")]
            steamworks,
        }
    }
}

impl luminol_eframe::App for App {
    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut luminol_eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        ctx.input(|i| {
            if let Some(f) = i.raw.dropped_files.first() {
                let path = f.path.clone().expect("dropped file has no path");
                let path = camino::Utf8PathBuf::from_path_buf(path).expect("path was not utf8");

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

        let mut update_state = luminol_core::UpdateState {
            audio: &mut self.audio,
            graphics: self.graphics.clone(),
            filesystem: &mut self.filesystem,
            data: &mut self.data,
            bytes_loader: self.bytes_loader.clone(),
            edit_windows: &mut luminol_core::EditWindows::default(),
            edit_tabs: &mut luminol_core::EditTabs::default(),
            toasts: &mut self.toasts,
            project_config: &mut self.project_config,
            global_config: &mut self.global_config,
            toolbar: &mut self.toolbar,
            modified: self.modified.clone(),
        };

        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(ui, frame, &mut update_state);

                // Process edit tabs for any changes made by top bar.
                // If we don't do this before displaying windows and tabs, any changes made by the top bar will be delayed a frame.
                // This means closing the project, for example, won't close tabs until the frame after.
                self.tabs
                    .process_edit_tabs(std::mem::take(update_state.edit_tabs));
                self.windows
                    .process_edit_windows(std::mem::take(update_state.edit_windows));
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
        self.tabs
            .process_edit_tabs(std::mem::take(update_state.edit_tabs));
        self.windows
            .process_edit_windows(std::mem::take(update_state.edit_windows));

        // Show toasts.
        self.toasts.show(ctx);

        self.lumi.ui(ctx);

        self.bytes_loader.load_unloaded_files(ctx, &self.filesystem);

        #[cfg(feature = "steamworks")]
        self.steamworks.update()
    }

    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn luminol_eframe::Storage) {
        luminol_eframe::set_value(storage, "SavedState", &self.global_config);
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }
}
