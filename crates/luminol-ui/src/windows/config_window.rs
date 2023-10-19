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

use strum::IntoEnumIterator;

/// The confg window
pub struct Window {}

impl Window {}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        "Local Luminol Config".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Local Luminol Config")
    }

    fn show<W, T>(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_, W, T>,
    ) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            let config = update_state
                .project_config
                .as_mut()
                .expect("project not open");

            ui.label("Project name");
            ui.text_edit_singleline(&mut config.project.project_name);
            ui.label("Scripts path");
            ui.text_edit_singleline(&mut config.project.scripts_path);
            ui.checkbox(
                &mut config.project.use_ron,
                "Use RON (Rusty Object Notation)",
            );
            egui::ComboBox::from_label("RGSS Version")
                .selected_text(config.project.rgss_ver.to_string())
                .show_ui(ui, |ui| {
                    for ver in luminol_config::RGSSVer::iter() {
                        ui.selectable_value(&mut config.project.rgss_ver, ver, ver.to_string());
                    }
                });

            ui.label("Playtest Executable");
            ui.text_edit_singleline(&mut config.project.playtest_exe);
        });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
