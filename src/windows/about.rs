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

/// A basic about window.
/// Shows some info on Luminol, along with an icon.
pub struct Window {
    icon: egui_extras::RetainedImage,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            // We load the icon here so it isn't loaded every frame. That would be bad if we did.
            // It would be better to load the image at compile time and only use one image instance
            // (as we load the image once at start for the icon) but this is the best I can do.
            icon: egui_extras::RetainedImage::from_image_bytes("icon", crate::ICON)
                .expect("Failed to load Icon data."),
        }
    }
}

impl super::window::Window for Window {
    fn name(&self) -> String {
        "About".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("About Luminol")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
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
                    ui.label(format!("git-rev {}", git_version::git_version!()));
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
