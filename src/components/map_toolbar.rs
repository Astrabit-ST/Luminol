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

use crate::{data::rmxp_structs::rpg, tabs::map::Map};
use ndarray::Axis;

impl Map {
    pub fn toolbar(&mut self, ui: &mut egui::Ui, map: &mut rpg::Map) {
        egui::TopBottomPanel::top(format!("map_{}_toolbar", self.id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Map {}: {}", self.name, self.id));

                ui.separator();

                ui.add(
                    egui::Slider::new(&mut self.tilemap.scale, 15.0..=200.)
                        .text("Scale")
                        .fixed_decimals(0),
                );

                ui.separator();

                // Find the number of layers.
                let layers = map.data.len_of(Axis(0));
                ui.menu_button(
                    // Format the text based on what layer is selected.
                    if self.selected_layer > layers {
                        "Events ⏷".to_string()
                    } else {
                        format!("Layer {} ⏷", self.selected_layer + 1)
                    },
                    |ui| {
                        // TODO: Add layer enable button
                        // Display all layers.
                        ui.columns(2, |columns| {
                            columns[1].visuals_mut().button_frame = true;

                            for layer in 0..layers {
                                columns[0].selectable_value(
                                    &mut self.selected_layer,
                                    layer,
                                    format!("Layer {}", layer + 1),
                                );
                                columns[1].checkbox(&mut self.toggled_layers[layer], "👁");
                            }
                            // Display event layer.
                            columns[0].selectable_value(
                                &mut self.selected_layer,
                                layers + 1,
                                "Events",
                            );
                            columns[1].checkbox(&mut self.toggled_layers[layers], "👁");
                        });
                    },
                );

                ui.separator();

                ui.checkbox(&mut self.tilemap.visible_display, "Display Visible Area");
            });
        });
    }
}
