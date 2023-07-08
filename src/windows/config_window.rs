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

use crate::{fl, prelude::*};

/// The confg window
pub struct Window {}

impl Window {}

impl window::Window for Window {
    fn name(&self) -> String {
        fl!("window_config_title_label")
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Local Luminol Config")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            let mut config = project_config!();

            ui.label(fl!("window_config_proj_name_label"));
            ui.text_edit_singleline(&mut config.project_name);
            ui.label(fl!("window_config_scripts_path_label"));
            ui.text_edit_singleline(&mut config.scripts_path);
            ui.checkbox(&mut config.use_ron, fl!("window_config_use_ron_cb"));
            egui::ComboBox::from_label(fl!("window_config_rgss_ver_label"))
                .selected_text(config.rgss_ver.to_string())
                .show_ui(ui, |ui| {
                    for ver in config::RGSSVer::iter() {
                        ui.selectable_value(&mut config.rgss_ver, ver, ver.to_string());
                    }
                });

            ui.label(fl!("window_config_playtest_exe_btn"));
            ui.text_edit_singleline(&mut config.playtest_exe);
        });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
