/// A basic about window.
pub struct About {}

impl About {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::window::Window for About {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        // Show the window. Name it "About Luminol"
        egui::Window::new("About Luminol")
            // Open is passed in. egui sets it to false if the window is closed.
            .open(open)
            .show(ctx, |ui| {
                ui.heading("Luminol");
                ui.label(format!("Luminol version: {}", env!("CARGO_PKG_VERSION")));

                ui.separator();

                ui.label("Luminol is a FOSS version of the RPG Maker XP editor.");

                ui.separator();

                ui.label(format!(
                    "Authors: \n{}",
                    env!("CARGO_PKG_AUTHORS").replace(':', ",\n")
                ))
            });
    }
}
