// Copyright (C) 2022 Lily Lyons
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

use std::hash::Hash;

use crate::UpdateInfo;

use super::modal::Modal;

/// The variable picker modal.
pub struct VariableModal {
    id: egui::Id,
}

impl VariableModal {
    /// Create a new modal.
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
        }
    }
}

impl Modal for VariableModal {
    type Data = usize;

    fn id(mut self, id: egui::Id) -> Self {
        self.id = id;
        self
    }

    fn button(
        mut self,
        ui: &mut egui::Ui,
        state: &mut bool,
        data: &mut Self::Data,
        info: &'static UpdateInfo,
    ) -> Self {
        {
            let system = info.data_cache.system();
            let system = system.as_ref().unwrap();

            if ui
                .button(format!("{data}: {}", system.variables[*data]))
                .clicked()
            {
                *state = true;

                // (selected value, value to scroll to, filter text)
                ui.ctx()
                    .memory()
                    .data
                    .get_temp_mut_or(self.id, (*data, *data, "".to_string()));
            }
        }

        if *state {
            self.show(ui.ctx(), state, data, info);
        }

        self
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        data: &mut Self::Data,
        info: &'static UpdateInfo,
    ) {
        let mut win_open = true;
        egui::Window::new("Variable Picker")
            .id(self.id)
            .resizable(false)
            .open(&mut win_open)
            .show(ctx, |ui| {
                let system = info.data_cache.system();
                let system = system.as_ref().unwrap();

                // (selected value, value to scroll to, filter text)
                let mut memory: (usize, usize, String) = ctx.data().get_temp(self.id).unwrap();

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show(ui, |ui| {
                            for (id, name) in
                                system.variables.iter().enumerate().filter(|(id, s)| {
                                    (id + 1).to_string().contains(&memory.2)
                                        || s.contains(&memory.2)
                                })
                            {
                                let id = id + 1;
                                let mut text = egui::RichText::new(format!("{id}: {name}"));

                                if memory.0 == id {
                                    text = text.color(egui::Color32::YELLOW);
                                }

                                let response = ui.selectable_value(data, id, text);

                                if memory.1 == id {
                                    memory.1 = usize::MAX;
                                    memory.0 = id;

                                    response.scroll_to_me(None);
                                }

                                if response.double_clicked() {
                                    *open = false;
                                }
                            }
                        })
                });

                ui.horizontal(|ui| {
                    *open = !ui.button("Ok").clicked();
                    *open = !ui.button("Cancel").clicked();

                    if ui
                        .add(
                            egui::DragValue::new(&mut memory.0)
                                .clamp_range(0..=system.variables.len()),
                        )
                        .changed()
                    {
                        memory.1 = memory.0;
                    };
                    egui::TextEdit::singleline(&mut memory.2)
                        .hint_text("Search 🔎")
                        .show(ui);
                });

                ctx.data().insert_temp(self.id, memory);
            });
        *open = *open && win_open;
    }
}
