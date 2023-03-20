use command_lib::CommandDescription;

struct App {
    commands: Vec<CommandDescription>,
    path: std::path::PathBuf,
}

impl App {
    fn new(commands: Option<Vec<CommandDescription>>, path: impl Into<std::path::PathBuf>) -> Self {
        App {
            commands: commands.unwrap_or_default(),
            path: path.into(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {}
}

fn main() {
    let path = std::env::args_os().nth(1).unwrap();

    let commands = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| ron::from_str(&text).ok());

    eframe::run_native(
        "Luminol Command Maker",
        Default::default(),
        Box::new(|_| Box::new(App::new(commands, path))),
    )
    .unwrap();
}
