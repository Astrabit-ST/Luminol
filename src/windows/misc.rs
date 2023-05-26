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

use super::window::Window;

/// Egui inspection window.
#[derive(Default)]
pub struct EguiInspection {}

impl Window for EguiInspection {
    fn name(&self) -> String {
        "Egui Inspection".to_string()
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
        "Egui Memory".to_string()
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
