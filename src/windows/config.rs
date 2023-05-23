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

/// The confg window
pub struct Window {}

impl Window {}

impl window::WindowExt for Window {
    fn name(&self) -> String {
        "Local Luminol Config".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Local Luminol Config")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            let mut config = interfaces!().data_cache.config();

            ui.label("Project name");
            ui.text_edit_singleline(&mut config.project_name);
            ui.label("Scripts path");
            ui.text_edit_singleline(&mut config.scripts_path);
            ui.checkbox(&mut config.use_ron, "Use RON (Rusty Object Notation)");
            egui::ComboBox::from_label("RGSS Version")
                .selected_text(config.rgss_ver.to_string())
                .show_ui(ui, |ui| {
                    for ver in RGSSVer::iter() {
                        ui.selectable_value(&mut config.rgss_ver, ver, ver.to_string());
                    }
                });

            ui.label("Playtest Executable");
            ui.text_edit_singleline(&mut config.playtest_exe);
        });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

impl<'win> From<Window> for crate::Window<'win> {
    fn from(value: Window) -> Self {
        crate::Window::Config(value)
    }
}
