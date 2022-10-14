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
#![allow(unused_imports)]
use egui::Pos2;
use ndarray::Axis;
use poll_promise::Promise;
use std::{cell::RefMut, collections::HashMap};

use crate::{
    components::tilemap::{Tilemap, TilemapDef},
    data::rmxp_structs::rpg,
    windows::{event_edit::EventEdit, window::Windows},
    UpdateInfo,
};

/// The map editor.
pub struct Map {
    /// ID of the map that is being edited.
    pub id: i32,
    /// Name of the map.
    pub name: String,
    /// Selected layer.
    pub selected_layer: usize,
    /// Toggled layers.
    pub toggled_layers: Vec<bool>,
    /// The cursor position.
    pub cursor_pos: Pos2,
    /// The tilemap.
    pub tilemap: Tilemap,
    /// The selected tile in the tile picker.
    pub selected_tile: i16,
    event_windows: Windows,
}

impl Map {
    /// Create a new map editor.
    pub fn new(id: i32, name: String, info: &'static UpdateInfo) -> Option<Self> {
        Some(Self {
            id,
            name,
            selected_layer: 0,
            toggled_layers: Vec::new(),
            cursor_pos: Pos2::ZERO,
            tilemap: Tilemap::new(info, id),
            selected_tile: 0,
            event_windows: Default::default(),
        })
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    #[allow(unused_variables, unused_mut)]
    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
        // Are we done loading data?
        if self.tilemap.textures_loaded() {
            if let Err(e) = self.tilemap.load_result() {
                info.toasts.error(e);
                return;
            }

            // Get the map.
            let mut map = info.data_cache.get_map(self.id);
            let tileset = info.data_cache.tilesets();
            let tileset = &tileset.as_ref().unwrap()[map.tileset_id as usize - 1];

            // If there are no toggled layers (i.e we just loaded the map)
            // then fill up the vector with `true`;
            if self.toggled_layers.is_empty() {
                self.toggled_layers = vec![true; map.data.len_of(Axis(0)) + 1];
                self.selected_layer = map.data.len_of(Axis(0)) + 1;
            }

            // Display the toolbar.
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

            // Display the tilepicker.
            egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
                .default_width(256.)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        self.tilemap.tilepicker(ui, &mut self.selected_tile);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let response = self.tilemap.ui(
                        ui,
                        &map,
                        &mut self.cursor_pos,
                        &self.toggled_layers,
                        self.selected_layer,
                    );

                    let layers_max = map.data.len_of(Axis(0));
                    let map_x = self.cursor_pos.x as i32;
                    let map_y = self.cursor_pos.y as i32;

                    if response.dragged()
                        && self.selected_layer < layers_max
                        && !ui.input().modifiers.command
                    {
                        map.data[[self.selected_layer, map_y as usize, map_x as usize]] =
                            self.selected_tile + 384;
                    } else if response.double_clicked() && self.selected_layer >= layers_max {
                        if let Some((id, event)) = map
                            .events
                            .iter()
                            .find(|(id, event)| event.x == map_x && event.y == map_y)
                        {
                            self.event_windows.add_window(EventEdit::new(
                                *id,
                                self.id,
                                event.clone(),
                                tileset.tileset_name.clone(),
                                info,
                            ));
                        }
                    }
                })
            });
        } else {
            // If not, just display a spinner.
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
        }

        self.event_windows.update(ui.ctx(), info);
    }

    #[cfg(feature = "discord-rpc")]
    fn discord_display(&self) -> String {
        format!("Editing Map<{}>: {}", self.id, self.name)
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
