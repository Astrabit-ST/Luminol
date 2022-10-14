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

/// The Luminol "get started screen" similar to vscode's.
#[derive(Default)]
pub struct Started {
    load_project_promise: Option<poll_promise::Promise<()>>,
}

impl Started {
    /// Create a new starting screen.
    pub fn new() -> Self {
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
        if ui
            .button(egui::RichText::new("New Project").size(20.))
            .clicked()
        {}
        if ui
            .button(egui::RichText::new("Open Project").size(20.))
            .clicked()
        {}

        ui.add_space(100.);

        ui.heading("Recent");
        if self
            .load_project_promise
            .is_some_and(|p| p.ready().is_none())
        {
            ui.spinner();
        } else {
            for path in info.saved_state.borrow().recent_projects.iter() {
                if ui.button(path).clicked() {
                    let path = path.clone();
                    self.load_project_promise =
                        Some(poll_promise::Promise::spawn_local(async move {
                            if let Err(e) = info
                                .filesystem
                                .load_project(path.into(), &info.data_cache)
                                .await
                            {
                                info.toasts.error(e);
                            }
                        }));
                }
            }
        }
    }
}
