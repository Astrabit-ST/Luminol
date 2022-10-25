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
    dragged_event: usize,
    dragging_event: bool,
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
            dragged_event: 0,
            dragging_event: false,
            event_windows: Default::default(),
        })
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
        // Are we done loading data?
        if self.tilemap.textures_loaded() {
            if let Err(e) = self.tilemap.load_result() {
                info.toasts.error(e);
                // FIXME: This will not! close the map tab
                return;
            }

            // Get the map.
            let mut map = info.data_cache.get_map(self.id);
            let tileset = info.data_cache.tilesets();
            let tileset = &tileset.as_ref().unwrap()[map.tileset_id as usize - 1];

            // If there are no toggled layers (i.e we just loaded the map)
            // then fill up the vector with `true`;
            if self.toggled_layers.is_empty() {
                self.toggled_layers = vec![true; map.data.zsize() + 1];
                self.selected_layer = map.data.zsize() + 1;
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
                    let layers = map.data.zsize();
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
                    let response =
                        self.tilemap
                            .ui(ui, &map, &mut self.cursor_pos, &self.toggled_layers);

                    let layers_max = map.data.zsize();
                    let map_x = self.cursor_pos.x as i32;
                    let map_y = self.cursor_pos.y as i32;

                    if response.dragged()
                        && self.selected_layer < layers_max
                        && !ui.input().modifiers.command
                    {
                        map.data[(map_x as usize, map_y as usize, self.selected_layer)] =
                            self.selected_tile + 384;
                    } else if self.selected_layer >= layers_max {
                        if response.double_clicked() {
                            if let Some((id, event)) = map
                                .events
                                .iter()
                                .find(|(_, event)| event.x == map_x && event.y == map_y)
                            {
                                self.event_windows.add_window(EventEdit::new(
                                    id,
                                    self.id,
                                    event.clone(),
                                    tileset.tileset_name.clone(),
                                    info,
                                ));
                            } else {
                                let id = map.events.vacant_key();
                                let event = rpg::event::Event::new(map_x, map_y, id);

                                map.events.insert(event.clone());

                                self.event_windows.add_window(EventEdit::new(
                                    id,
                                    self.id,
                                    event,
                                    tileset.tileset_name.clone(),
                                    info,
                                ));
                            }
                        } else if response.drag_started() {
                            if let Some((id, _)) = map
                                .events
                                .iter()
                                .find(|(_, event)| event.x == map_x && event.y == map_y)
                            {
                                self.dragged_event = id;
                                self.dragging_event = true;
                            }
                        } else if response.dragged() && self.dragging_event {
                            map.events[self.dragged_event].x = map_x;
                            map.events[self.dragged_event].y = map_y;
                        } else if response.drag_released() {
                            self.dragging_event = false;
                        }
                    }

                    if ui.input().key_pressed(egui::Key::Delete)
                        || ui.input().key_pressed(egui::Key::Backspace)
                    {
                        if let Some((id, _)) = map
                            .events
                            .iter()
                            .find(|(_, event)| event.x == map_x && event.y == map_y)
                        {
                            map.events.remove(id);
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
