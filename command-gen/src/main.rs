use command_lib::{CommandDescription, CommandKind, Index, Parameter};
use eframe::egui;

use strum::IntoEnumIterator;

mod ui_example;
use ui_example::UiExample;

mod parameter_ui;

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

    fn recalculate_parameter_index(parameter: &mut Parameter, passed_index: &mut u8) {
        match parameter {
            Parameter::Group { parameters, .. } => {
                for parameter in parameters.iter_mut() {
                    Self::recalculate_parameter_index(parameter, passed_index);
                }
            }
            Parameter::Selection {
                index, parameters, ..
            } => {
                if let Index::Assumed(ref mut assumed_index) = index {
                    *assumed_index = *passed_index;
                }

                *passed_index += 1;

                *passed_index = parameters
                    .iter_mut()
                    .map(|(_, parameter)| {
                        let mut passed_index = *passed_index;
                        Self::recalculate_parameter_index(parameter, &mut passed_index);
                        passed_index
                    })
                    .max()
                    .unwrap_or(0)
            }
            Parameter::Single { index, .. } => {
                if let Index::Assumed(ref mut assumed_index) = index {
                    *assumed_index = *passed_index;
                }

                *passed_index += 1;
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                let mut del_index = None;
                for (idx, command) in self.commands.iter_mut().enumerate() {
                        ui.push_id(command.guid, |ui| {
                        let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            format!("command_{idx}").into(),
                            false,
                        );
                        header
                            .show_header(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.text_edit_singleline(&mut command.name);

                                    ui.label("Code:");
                                    ui.add(egui::DragValue::new(&mut command.code));
                                });

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
                                ui.label("Description:");
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
                                    for (ele, parameter) in command.parameters.iter_mut().enumerate() {
                                        parameter_ui::parameter_ui(ui, parameter,  (ele, &mut del_idx));
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
                    });
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

        for command in self.commands.iter_mut() {
            let mut passed_index = 0;
            for parameter in command.parameters.iter_mut() {
                Self::recalculate_parameter_index(parameter, &mut passed_index);
            }
        }
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
