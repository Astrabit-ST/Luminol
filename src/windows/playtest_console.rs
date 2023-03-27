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

use std::io::prelude::*;
use std::process::Child;

pub struct PlaytestConsole {
    child: Child,

    stdout_buf: String,
}

impl PlaytestConsole {
    #[must_use]
    pub fn new(child: Child) -> Self {
        Self {
            child,
            stdout_buf: String::with_capacity(4096),
        }
    }
}

impl super::window::Window for PlaytestConsole {
    fn name(&self) -> String {
        "Playtest Console".to_string()
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            if let Some(ref mut stdout) = self.child.stdout {
                if let Err(e) = stdout.read_to_string(&mut self.stdout_buf) {
                    info.toasts
                        .error(format!("Error reading process stdout {e:?}"));
                }
            }

            egui::ScrollArea::both().show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Monospace),
                self.stdout_buf.lines().count(),
                |ui, rows| {
                    for line in self
                        .stdout_buf
                        .lines()
                        .skip(rows.start)
                        .take(rows.end - rows.start)
                    {
                        ui.label(egui::RichText::new(line).monospace());
                    }
                },
            );

            if ui
                .button(egui::RichText::new("KILL").color(egui::Color32::RED))
                .clicked()
            {
                if let Err(e) = self.child.kill() {
                    info.toasts
                        .error(format!("Error killing playtest process: {e:?}"));
                }
            }
        });

        if !*open {
            if let Err(e) = self.child.kill() {
                info.toasts
                    .error(format!("Error killing playtest process: {e:?}"));
            }
        }
    }
}
