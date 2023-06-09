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

use crate::prelude::*;

/// The Luminol "get started screen" similar to vscode's.
#[derive(Default)]
pub struct Tab {
    load_project_promise: Option<poll_promise::Promise<()>>,
}

// FIXME
#[allow(unsafe_code)]
unsafe impl Send for Tab {}

impl Tab {
    /// Create a new starting screen.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl tab::Tab for Tab {
    fn name(&self) -> String {
        "Get Started".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_started_tab")
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        let state = state!();
        ui.label(
            egui::RichText::new("Luminol")
                .size(40.)
                .color(egui::Color32::LIGHT_GRAY),
        );

        ui.add_space(100.);

        ui.heading("Start");

        if self
            .load_project_promise
            .as_ref()
            .is_some_and(|p| p.ready().is_none())
        {
            ui.spinner();
        } else {
            if ui
                .button(egui::RichText::new("New Project").size(20.))
                .clicked()
            {
                state!()
                    .windows
                    .add_window(crate::windows::new_project::Window::default());
            }
            if ui
                .button(egui::RichText::new("Open Project").size(20.))
                .clicked()
            {
                self.load_project_promise = Some(Promise::spawn_local(async move {
                    if let Err(e) = state.filesystem.spawn_project_file_picker().await {
                        state
                            .toasts
                            .error(format!("Error loading the project: {e}"));
                    }
                }));
            }

            ui.add_space(100.);

            ui.heading("Recent");

            for path in &global_config!().recent_projects {
                if ui.button(path).clicked() {
                    let path = path.clone();

                    self.load_project_promise = Some(Promise::spawn_local(async move {
                        if let Err(why) = state.filesystem.load_project(path) {
                            state
                                .toasts
                                .error(format!("Error loading the project: {why}"));
                        } else {
                            state!().toasts.info(format!(
                                "Successfully opened {:?}",
                                state!()
                                    .filesystem
                                    .project_path()
                                    .expect("project not open")
                            ));
                        }
                    }));
                }
            }
        }

        state!().filesystem.debug_ui(ui);
    }
}
