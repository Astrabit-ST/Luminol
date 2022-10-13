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

pub struct SwitchModal {
    id: egui::Id,
}

impl SwitchModal {
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
        }
    }
}

impl Modal for SwitchModal {
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
                .button(format!("{}: {}", data, system.switches[*data]))
                .clicked()
            {
                *state = true
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
        egui::Window::new("Switch Picker")
            .id(self.id)
            .resizable(false)
            .open(&mut win_open)
            .show(ctx, |ui| {
                let system = info.data_cache.system();
                let system = system.as_ref().unwrap();

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show_rows(
                            ui,
                            ui.text_style_height(&egui::TextStyle::Body),
                            system.switches.len(),
                            |ui, rows| {
                                for id in rows {
                                    let response = ui.selectable_value(
                                        data,
                                        id,
                                        format!("{}: {}", id, system.switches[id]),
                                    );

                                    if response.double_clicked() {
                                        *open = false;
                                    }
                                }
                            },
                        )
                });

                ui.horizontal(|ui| *open = !ui.button("Ok").clicked())
            });
        *open = *open && win_open;
    }
}
