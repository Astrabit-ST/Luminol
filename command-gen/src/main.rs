use command_lib::{CommandDescription, CommandKind, Parameter, ParameterKind};
use eframe::egui;

use strum::IntoEnumIterator;

struct App {
    commands: Vec<CommandDescription>,
    path: std::path::PathBuf,

    ui_examples: Vec<UiExample>,
}

impl App {
    fn new(commands: Option<Vec<CommandDescription>>, path: impl Into<std::path::PathBuf>) -> Self {
        App {
            commands: commands.unwrap_or_default(),
            path: path.into(),
            ui_examples: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mut del_index = None;
                for (idx, command) in self.commands.iter_mut().enumerate() {
                    let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        format!("command_{idx}").into(),
                        false,
                    );
                    header
                        .show_header(ui, |ui| {
                            ui.add(egui::DragValue::new(&mut command.code));

                            ui.text_edit_singleline(&mut command.name);

                            if ui
                                .button(
                                    egui::RichText::new("-")
                                        .monospace()
                                        .color(egui::Color32::RED),
                                )
                                .clicked()
                            {
                                del_index = Some(idx)
                            }
                        })
                        .body(|ui| {
                            ui.label("Description");
                            ui.text_edit_multiline(&mut command.description);

                            ui.separator();

                            ui.label("Type");
                            ui.horizontal(|ui| {
                                ui.menu_button(
                                    format!("{} ⏷", <&str>::from(command.kind)),
                                    |ui| {
                                        for kind in CommandKind::iter() {
                                            ui.selectable_value(
                                                &mut command.kind,
                                                kind,
                                                <&str>::from(kind),
                                            );
                                        }
                                    },
                                );
                                if let CommandKind::Multi(ref mut code) = command.kind {
                                    ui.label("Code");
                                    ui.add(egui::DragValue::new(code));
                                }
                            });

                            ui.separator();

                            ui.collapsing("Parameters", |ui| {
                                Self::parameter_ui(ui, &mut command.parameters);
                            });
                        });

                    if ui.button("Preview UI").clicked() {
                        self.ui_examples.push(UiExample::new(command));
                    }

                    ui.separator();
                }

                if let Some(idx) = del_index {
                    self.commands.remove(idx);
                }

                if ui
                    .button(
                        egui::RichText::new("+")
                            .monospace()
                            .color(egui::Color32::GREEN),
                    )
                    .clicked()
                {
                    self.commands.push(CommandDescription::default());
                }
            });
        });

        self.ui_examples.retain_mut(|e| e.update(ctx));
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        std::fs::write(
            &self.path,
            ron::ser::to_string_pretty(
                &self.commands,
                ron::ser::PrettyConfig::new().struct_names(true),
            )
            .unwrap(),
        )
        .unwrap();
    }
}

impl App {
    fn parameter_ui(ui: &mut egui::Ui, parameters: &mut Vec<Parameter>) {
        let mut del_index = None;
        for (idx, parameter) in parameters.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label("Index: ");
                if let Some(ref mut idx) = parameter.index {
                    ui.add(egui::DragValue::new(idx));
                } else {
                    let mut override_idx = idx;
                    if ui.add(egui::DragValue::new(&mut override_idx)).changed() {
                        parameter.index = Some(override_idx as u8);
                    }
                }

                if ui
                    .button(
                        egui::RichText::new("-")
                            .monospace()
                            .color(egui::Color32::RED),
                    )
                    .clicked()
                {
                    del_index = Some(idx);
                }
            });

            ui.horizontal(|ui| {
                ui.label("Type: ");
                ui.menu_button(format!("{} ⏷", <&str>::from(&parameter.kind)), |ui| {
                    for kind in ParameterKind::iter() {
                        let text: &str = (&kind).into();
                        ui.selectable_value(&mut parameter.kind, kind, text);
                    }
                });
            });

            if let ParameterKind::Group { ref mut parameters }
            | ParameterKind::Selection { ref mut parameters } = parameter.kind
            {
                ui.collapsing("Subparameters", |ui| {
                    Self::parameter_ui(ui, parameters);
                });
            }
            ui.separator();
        }

        if let Some(idx) = del_index {
            parameters.remove(idx);
        }

        if ui
            .button(
                egui::RichText::new("+")
                    .monospace()
                    .color(egui::Color32::GREEN),
            )
            .clicked()
        {
            parameters.push(Parameter::default());
        }
    }
}

struct UiExample {
    command: CommandDescription,
}

impl UiExample {
    fn new(desc: &CommandDescription) -> Self {
        Self {
            command: desc.clone(),
        }
    }

    fn update(&mut self, ctx: &egui::Context) -> bool {
        let mut open = true;
        egui::Window::new(format!(
            "[{}] {} ui example",
            self.command.code, self.command.name
        ))
        .open(&mut open)
        .show(ctx, |ui| {});
        open
    }
}

fn main() {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Error: No path specified");

        return;
    };

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
