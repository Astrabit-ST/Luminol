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
                            ui.text_edit_multiline(&mut command.description)
                                .on_hover_text("Description for this command");

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
                                match command.kind {
                                    CommandKind::Multi(ref mut code) =>{
                                        ui.label("Cont. Code").on_hover_text("Luminol will assume that any following commands with this code are a part of this one");
                                        ui.add(egui::DragValue::new(code));
                                    }
                                    CommandKind::Branch(ref mut code ) => {
                                        ui.label("End Code").on_hover_text("Luminol will add this command to denote the end of the branch");
                                        ui.add(egui::DragValue::new(code));
                                    }
                                    _ => {}
                                }
                            });

                            ui.checkbox(&mut command.hidden, "Hide in menu");

                            ui.separator();

                            ui.collapsing("Parameters", |ui| {
                                let mut del_idx = None;
                                for (idx, parameter) in command.parameters.iter_mut().enumerate() {
                                    Self::parameter_ui(ui, parameter, idx, &mut del_idx)
                                }

                                if let Some(idx) = del_idx {
                                    command.parameters.remove(idx);
                                }

                                if ui
                                    .button(
                                        egui::RichText::new("+")
                                            .monospace()
                                            .color(egui::Color32::GREEN),
                                    )
                                    .clicked()
                                {
                                    command.parameters.push(Parameter::default());
                                }
                            });
                        });

                    if command.parameter_count() > 0 && ui.button("Preview UI").clicked() {
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
    fn parameter_ui(
        ui: &mut egui::Ui,
        parameter: &mut Parameter,
        idx: usize,
        del_idx: &mut Option<usize>,
    ) {
        ui.text_edit_singleline(&mut parameter.name);
        ui.horizontal(|ui| {
                ui.label("Position: ").on_hover_text_at_pointer("Position of this parameter, when not set it is assumed to be the index of the parameter");
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
                    *del_idx = Some(idx);
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

        ui.label("Description");
        ui.text_edit_multiline(&mut parameter.description)
            .on_hover_text("Description for this parameter");

        match parameter.kind {
            ParameterKind::Group { ref mut parameters } => {
                ui.collapsing("Grouped parameters", |ui| {
                    let mut del_idx = None;
                    for parameter in parameters.iter_mut() {
                        Self::parameter_ui(ui, parameter, idx, &mut del_idx)
                    }

                    if let Some(idx) = del_idx {
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
                })
                .header_response
                .on_hover_text("This parameter groups together other parameters");
            }
            ParameterKind::Selection { ref mut parameters } => {
                ui.collapsing("Subparameters", |ui| {
                    let mut del_idx = None;
                    for (id, parameter) in parameters.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(id));

                            ui.vertical(|ui| Self::parameter_ui(ui, parameter, idx, &mut del_idx));
                        });
                    }

                    if let Some(idx) = del_idx {
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
                        parameters.push((0, Parameter::default()));
                    }
                })
                .header_response
                .on_hover_text("This parameter selects one of the following parameters");
            }
            ParameterKind::StringMulti { ref mut highlight } => {
                ui.checkbox(highlight, "Enable ruby syntax highlighting");
            }
            ParameterKind::Enum { ref mut variants } => {
                ui.collapsing("Variants", |ui| {
                    let mut del_idx = None;
                    for (name, id) in variants.iter_mut() {
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(name);
                            ui.add(egui::DragValue::new(id));

                            if ui
                                .button(
                                    egui::RichText::new("-")
                                        .monospace()
                                        .color(egui::Color32::RED),
                                )
                                .clicked()
                            {
                                del_idx = Some(idx);
                            }
                        });
                    }

                    if let Some(idx) = del_idx {
                        variants.remove(idx);
                    }

                    if ui
                        .button(
                            egui::RichText::new("+")
                                .monospace()
                                .color(egui::Color32::GREEN),
                        )
                        .clicked()
                    {
                        variants.push(("".to_string(), 0));
                    }
                })
                .header_response
                .on_disabled_hover_text("Variants for the enum");
            }
            _ => {}
        }
        ui.separator();
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
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(&self.command.name).monospace())
                .on_hover_text(&self.command.description);

            ui.separator();

            let mut index = 0;
            for parameter in &mut self.command.parameters {
                Self::parameter_ui(ui, parameter, &mut index);
            }
        });
        open
    }

    fn parameter_ui(ui: &mut egui::Ui, parameter: &mut Parameter, index: &mut u8) {
        if let ParameterKind::Dummy = parameter.kind {
            *index += 1;

            return;
        }

        ui.label(format!(
            "[{}]: {}",
            parameter.index.unwrap_or(*index),
            parameter.name,
        ))
        .on_hover_text(&parameter.description);

        *index += 1;

        match parameter.kind {
            ParameterKind::Selection { ref mut parameters } => {
                for (_, parameter) in parameters {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut false, "");
                        ui.vertical(|ui| {
                            ui.add_enabled_ui(false, |ui| Self::parameter_ui(ui, parameter, index));
                        });
                    });
                }
            }
            ParameterKind::Group { ref mut parameters } => {
                ui.group(|ui| {
                    for parameter in parameters {
                        Self::parameter_ui(ui, parameter, index);
                    }
                });
            }
            ParameterKind::Switch => {
                ui.button("Switch: [000: EXAMPLE]").clicked();
            }
            ParameterKind::Variable => {
                ui.button("Variable [000: EXAMPLE]").clicked();
            }
            ParameterKind::String => {
                ui.text_edit_singleline(&mut "".to_string());
            }
            ParameterKind::StringMulti { .. } => {
                ui.text_edit_multiline(&mut "".to_string());
            }
            ParameterKind::Int => {
                ui.add(egui::DragValue::new(&mut 0i16));
            }
            ParameterKind::IntBool => {
                ui.checkbox(&mut false, "");
            }
            ParameterKind::Enum { ref variants } => {
                let (first_name, mut first_id) = variants.first().unwrap();
                ui.menu_button(format!("{first_name} ⏷"), |ui| {
                    for (name, id) in variants.iter() {
                        ui.selectable_value(&mut first_id, *id, name);
                    }
                });
            }
            ParameterKind::SelfSwitch => {
                ui.menu_button("A ⏷", |ui| {
                    for char in ['A', 'B', 'C', 'D'] {
                        ui.selectable_value(&mut 'A', char, char.to_string());
                    }
                });
            }
            ParameterKind::Dummy => unreachable!(),
        }
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
