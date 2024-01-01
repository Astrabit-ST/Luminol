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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

pub struct Window {
    term: luminol_term::Terminal,
}

impl Window {
    pub fn new(
        ctx: &egui::Context,
        command: luminol_term::CommandBuilder,
    ) -> Result<Self, luminol_term::Error> {
        Ok(Self {
            term: luminol_term::Terminal::new(ctx, command)?,
        })
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        self.term.title()
    }

    fn id(&self) -> egui::Id {
        self.term.id()
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new(self.name())
            .id(self.term.id())
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(egui::RichText::new("KILL").color(egui::Color32::RED))
                        .clicked()
                    {
                        if let Err(e) = self.term.kill() {
                            luminol_core::error!(
                                update_state.toasts,
                                e.wrap_err("Error killing child"),
                            );
                        }
                    }

                    let mut resize = false;
                    let (mut cols, mut rows) = self.term.size();

                    resize |= ui.add(egui::DragValue::new(&mut cols)).changed();
                    ui.label("×");
                    resize |= ui.add(egui::DragValue::new(&mut rows)).changed();

                    if resize {
                        self.term.set_size(update_state, cols, rows);
                    }
                });

                ui.add_space(ui.spacing().item_spacing.y);

                if let Err(e) = self.term.ui(ui) {
                    luminol_core::error!(
                        update_state.toasts,
                        e.wrap_err("Error displaying terminal"),
                    );
                }
            });
    }
}
