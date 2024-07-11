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

/// The Luminol "get started screen" similar to vscode's.
#[derive(Default)]
pub struct Tab {}

impl Tab {
    /// Create a new starting screen.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl luminol_core::Tab for Tab {
    fn name(&self, _update_state: &luminol_core::UpdateState<'_>) -> String {
        "Get Started".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_started_tab")
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        _is_focused: bool,
    ) {
        ui.label(
            egui::RichText::new("Luminol")
                .size(40.)
                .color(egui::Color32::LIGHT_GRAY),
        );

        ui.add_space(100.);

        ui.heading("Start");

        if update_state
            .project_manager
            .load_filesystem_promise
            .is_some()
        {
            ui.spinner();
        }

        ui.add_enabled_ui(
            update_state
                .project_manager
                .load_filesystem_promise
                .is_none(),
            |ui| {
                if ui
                    .button(egui::RichText::new("New Project").size(20.))
                    .clicked()
                {
                    update_state
                        .edit_windows
                        .add_window(crate::windows::new_project::Window::default());
                }
                if ui
                    .button(egui::RichText::new("Open Project").size(20.))
                    .clicked()
                {
                    update_state.project_manager.open_project_picker();
                }
            },
        );

        ui.add_space(100.);

        ui.heading("Recent");

        for path in update_state.global_config.recent_projects.clone() {
            #[cfg(target_arch = "wasm32")]
            let (path, idb_key) = path;

            if ui.button(&path).clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                update_state.project_manager.load_recent_project(path);
                #[cfg(target_arch = "wasm32")]
                update_state.project_manager.load_recent_project(idb_key);
            }
        }
    }
}
