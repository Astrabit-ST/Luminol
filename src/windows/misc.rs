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

use crate::fl;

use super::window::Window;
use crate::prelude::*;
/// Egui inspection window.
#[derive(Default)]
pub struct EguiInspection {}

impl Window for EguiInspection {
    fn name(&self) -> String {
        fl!("window_egui_inspec_title_label")
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Egui Inspection")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .show(ctx, |ui| ctx.inspection_ui(ui));
    }
}

/// Egui memory display.
#[derive(Default)]
pub struct EguiMemory {}

impl Window for EguiMemory {
    fn name(&self) -> String {
        fl!("window_egui_memory_title_label")
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Egui Memory")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .show(ctx, |ui| ctx.memory_ui(ui));
    }
}

#[derive(Default)]
pub struct FilesystemDebug {}

impl Window for FilesystemDebug {
    fn name(&self) -> String {
        "Filesystem Debug".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Filesystem Debug Window")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .open(open)
            .show(ctx, |ui| state!().filesystem.debug_ui(ui));
    }
}
