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
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        // Show the window. Name it "About Luminol"
        egui::Window::new("About Luminol")
            // Open is passed in. egui sets it to false if the window is closed.
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    self.icon.show_scaled(ui, 0.5);
                    ui.heading("Luminol");

                    ui.separator();
                    ui.label(format!("Luminol version {}", env!("CARGO_PKG_VERSION")));
                    ui.separator();

                    ui.label("Luminol is a FOSS version of the RPG Maker XP editor.");
                    ui.separator();

                    ui.label(format!(
                        "Authors: \n{}",
                        env!("CARGO_PKG_AUTHORS").replace(':', ",\n")
                    ))
                })
            });
    }
}
