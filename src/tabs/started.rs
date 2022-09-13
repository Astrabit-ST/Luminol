pub struct Started {}

impl Started {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::tab::Tab for Started {
    fn name(&self) -> String {
        "Get Started".to_string()
    }

    fn show(&mut self, ui: &mut egui::Ui, _info: &crate::UpdateInfo<'_>) {
        ui.centered_and_justified(|ui| {
            ui.heading("Luminol");
        });
    }
}
