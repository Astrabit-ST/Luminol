// Copyright (C) 2022 Lily Lyons
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

use std::collections::VecDeque;

use crate::prelude::*;

/// The main Luminol struct. Handles rendering, GUI state, that sort of thing.
pub struct Luminol {
    top_bar: TopBar,
    info: &'static UpdateInfo,
    style: Arc<egui::Style>,
}

impl Luminol {
    /// Called once before the first frame.
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage = cc.storage.unwrap();

        let state = eframe::get_value(storage, "SavedState").map_or_else(
            || {
                let mut state = crate::SavedState {
                    recent_projects: VecDeque::new(),
                };
                state.recent_projects.reserve(10);
                state
            },
            |s| s,
        );

        let style =
            eframe::get_value(storage, "EguiStyle").map_or_else(|| cc.egui_ctx.style(), |s| s);
        cc.egui_ctx.set_style(style.clone());

        Self {
            top_bar: TopBar::default(),
            info: Box::leak(Box::new(UpdateInfo::new(
                cc.gl.as_ref().unwrap().clone(),
                state,
            ))),
            style,
        }
    }
}

impl eframe::App for Luminol {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value::<crate::SavedState>(
            storage,
            "SavedState",
            &self.info.saved_state.borrow(),
        );
        eframe::set_value::<Arc<egui::Style>>(storage, "EguiStyle", &self.style);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(self.info, ui, &mut self.style, frame);
            });
        });

        // Central panel with tabs.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.info.tabs.ui(ui, self.info);
        });

        {
            // Update all windows.
            self.info.windows.update(ctx, self.info);
        }

        // Show toasts.
        {
            self.info.toasts.show(ctx);
        }

        // Tick futures.
        #[cfg(not(target_arch = "wasm32"))]
        {
            poll_promise::tick_local();
        }
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
