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

use crate::UpdateInfo;

/// A basic about window.
/// Shows some info on Luminol, along with an icon.
pub struct About {
    icon: egui_extras::RetainedImage,
}

impl About {
    pub fn new() -> Self {
        Self {
            // We load the icon here so it isn't loaded every frame. That would be bad if we did.
            // It would be better to load the image at compile time and only use one image instance
            // (as we load the image once at start for the icon) but this is the best I can do.
            icon: egui_extras::RetainedImage::from_image_bytes("icon", crate::ICON)
                .expect("Failed to load Icon data."),
        }
    }
}

impl super::window::Window for About {
    fn name(&self) -> String {
        "About".to_string()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, _: &UpdateInfo<'_>) {
        // Show the window. Name it "About Luminol"
        egui::Window::new("About Luminol")
            // Open is passed in. egui sets it to false if the window is closed.
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                // Center the widgets vertically for cleanliness.
                ui.vertical_centered(|ui| {
                    self.icon.show_scaled(ui, 0.5); // We scale the icon down since it's pretty huge.
                    ui.heading("Luminol");

                    ui.separator();
                    ui.label(format!("Luminol version {}", env!("CARGO_PKG_VERSION")));
                    ui.separator();

                    ui.label("Luminol is a FOSS version of the RPG Maker XP editor.");
                    ui.separator();

                    ui.label(format!(
                        "Authors: \n{}",
                        env!("CARGO_PKG_AUTHORS").replace(':', "\n")
                    ))
                })
            });
    }
}
