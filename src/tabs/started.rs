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

use crate::filesystem::Filesystem;

/// The Luminol "get started screen" similar to vscode's.
#[derive(Default)]
pub struct Started {
    load_project_promise: Option<poll_promise::Promise<()>>,
}

impl Started {
    /// Create a new starting screen.
    #[must_use] pub fn new() -> Self {
        Default::default()
    }
}

impl super::tab::Tab for Started {
    fn name(&self) -> String {
        "Get Started".to_string()
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
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
                info.windows
                    .add_window(crate::windows::new_project::NewProjectWindow::default());
            }
            if ui
                .button(egui::RichText::new("Open Project").size(20.))
                .clicked()
            {
                self.load_project_promise = Some(poll_promise::Promise::spawn_local(async move {
                    if let Err(e) = info.filesystem.try_open_project(info).await {
                        info.toasts.error(format!("Error loading project: {e}"));
                    }
                }));
            }

            ui.add_space(100.);

            #[cfg(not(target_arch = "wasm32"))]
            ui.heading("Recent");

            #[cfg(not(target_arch = "wasm32"))]
            for path in &info.saved_state.borrow().recent_projects {
                if ui.button(path).clicked() {
                    let _path = path.clone();
                    self.load_project_promise =
                        Some(poll_promise::Promise::spawn_local(async move {
                            // FIXME: re-add feature
                            // if let Err(e) =
                            //     info.filesystem.load_project(&path, &info.data_cache).await
                            // {
                            //     info.toasts
                            //         .error(format!("Error loading project {path}: {e}"));
                            // }
                        }));
                }
            }
        }
    }
}
