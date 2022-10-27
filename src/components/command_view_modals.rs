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

use crate::UpdateInfo;

use super::command_view::{CommandView, Memory};

impl<'co> CommandView<'co> {
    pub(crate) fn modals(
        &mut self,
        ui: &mut egui::Ui,
        memory: &mut Memory,
        info: &'static UpdateInfo,
    ) {
        let Memory {
            move_route_modal,
            map_id,
            ..
        } = memory;

        let mut route_open = move_route_modal.1.is_some();
        egui::Window::new("Preview Move Route")
            .id(egui::Id::new(format!(
                "move_route_preview_{}",
                self.custom_id_source
            )))
            .open(&mut route_open)
            .show(ui.ctx(), |ui| {
                let mut map = info.data_cache.get_map(map_id.unwrap());

                ui.label("Starting Direction");
                ui.radio_value(&mut move_route_modal.0, 2, "Up");
                ui.radio_value(&mut move_route_modal.0, 8, "Down");
                ui.radio_value(&mut move_route_modal.0, 4, "Left");
                ui.radio_value(&mut move_route_modal.0, 6, "Right");

                ui.horizontal(|ui| {
                    if ui.button("Ok").clicked() {
                        map.preview_move_route =
                            Some((move_route_modal.0, move_route_modal.1.take().unwrap()))
                    }

                    if ui.button("Apply").clicked() {
                        map.preview_move_route =
                            Some((move_route_modal.0, move_route_modal.1.clone().unwrap()))
                    }

                    if ui.button("Cancel").clicked() {
                        move_route_modal.1 = None;
                    }
                });
            });

        if !route_open {
            move_route_modal.1 = None;
        }
    }
}
