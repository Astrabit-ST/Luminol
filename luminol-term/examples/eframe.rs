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

use luminol_term::Terminal;

struct App {
    command_name: String,
    terminals: Vec<Terminal>,
}

impl App {
    fn new() -> Self {
        Self {
            command_name: String::new(),
            terminals: vec![],
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Command name");
            ui.text_edit_singleline(&mut self.command_name);

            if ui.button("Run").clicked() {
                match std::process::Command::new(&self.command_name)
                    .stdout(std::process::Stdio::piped())
                    .stdin(std::process::Stdio::piped())
                    .spawn()
                {
                    Ok(child) => match Terminal::new(child) {
                        Ok(t) => self.terminals.push(t),
                        Err(e) => eprintln!("error creating terminal: {e:?}"),
                    },
                    Err(e) => eprintln!("error starting process: {e:?}"),
                }
            }
        });

        for terminal in self.terminals.iter_mut() {
            egui::Window::new(terminal.title()).show(ctx, |ui| {});
        }
    }
}

fn main() {
    eframe::run_native(
        "Luminol Terminal Example",
        Default::default(),
        Box::new(|_| Box::new(App::new())),
    )
    .expect("failed to start eframe");
}
