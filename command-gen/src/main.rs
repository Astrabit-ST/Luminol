struct App;

impl App {
    fn new() -> Self {
        App
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {}
}

fn main() {
    eframe::run_native(
        "Luminol Command Maker",
        Default::default(),
        Box::new(|_| Box::new(App::new())),
    )
    .unwrap();
}
