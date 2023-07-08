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

use crate::{fl, prelude::*};

/// The switch picker modal.
pub struct Modal {
    id: egui::Id,
}

impl Modal {
    /// Create a new modal.
    pub fn new(id: egui::Id) -> Self {
        Self { id }
    }
}

impl modal::Modal for Modal {
    type Data = usize;

    fn id(mut self, id: egui::Id) -> Self {
        self.id = id;
        self
    }

    fn button(mut self, ui: &mut egui::Ui, state: &mut bool, data: &mut Self::Data) -> Self {
        {
            let system = state!().data_cache.system();

            if ui
                .button(format!("{data}: {}", system.switches[*data - 1]))
                .clicked()
            {
                *state = true;
                ui.ctx().memory_mut(|m| {
                    m.data
                        .get_temp_mut_or(self.id, (*data, *data, String::new()));
                });
            }
        }

        if *state {
            self.show(ui.ctx(), state, data);
        }

        self
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, data: &mut Self::Data) {
        let mut win_open = true;
        egui::Window::new(fl!("modal_switch_title_label"))
            .id(self.id)
            .resizable(false)
            .open(&mut win_open)
            .show(ctx, |ui| {
                let system = state!().data_cache.system();

                let mut memory: (usize, usize, String) =
                    ctx.data_mut(|m| m.get_temp(self.id).unwrap());

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show(ui, |ui| {
                            for (id, name) in
                                system.switches.iter().enumerate().filter(|(id, s)| {
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
                    *open = !ui.button(fl!("ok")).clicked();
                    *open = !ui.button(fl!("cancel")).clicked();

                    if ui
                        .add(
                            egui::DragValue::new(&mut memory.0)
                                .clamp_range(0..=system.switches.len()),
                        )
                        .changed()
                    {
                        memory.1 = memory.0;
                    };
                    egui::TextEdit::singleline(&mut memory.2)
                        .hint_text(format!("{} 🔎", fl!("search")))
                        .show(ui);
                });

                ctx.data_mut(|m| {
                    m.insert_temp(self.id, memory);
                });
            });
        *open = *open && win_open;
    }
}
