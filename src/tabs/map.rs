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

#![allow(unused_imports)]
use egui::Pos2;
use std::{cell::RefMut, collections::HashMap};

use crate::prelude::*;

pub struct Tab {
    /// ID of the map that is being edited.
    pub id: usize,
    /// The tilemap.
    pub view: MapView,
    pub tilepicker: Tilepicker,

    dragging_event: bool,
    event_windows: window::Windows,
    force_close: bool,
}

impl Tab {
    /// Create a new map editor.
    pub fn new(id: usize) -> Result<Self, String> {
        let map = state!().data_cache.map(id);
        let tilesets = state!().data_cache.tilesets();
        let tileset = &tilesets[map.tileset_id];

        Ok(Self {
            id,
            view: MapView::new(&map, tileset)?,
            tilepicker: Tilepicker::new(tileset)?,

            dragging_event: false,
            event_windows: window::Windows::default(),
            force_close: false,
        })
    }

    fn recompute_autotile(&self, map: &rpg::Map, position: (usize, usize, usize)) -> i16 {
        if map.data[position] >= 384 {
            return map.data[position];
        }

        let autotile = map.data[position] / 48;
        if autotile == 0 {
            return 0;
        }

        let x_array: [i8; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
        let y_array: [i8; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

        /*
         * 765
         * 0 4
         * 123
         */
        let mut bitfield = 0u8;

        // Loop through the 8 neighbors of this position
        for (x, y) in x_array.into_iter().zip(y_array.into_iter()) {
            bitfield <<= 1;
            // Out-of-bounds tiles always count as valid neighbors
            let is_out_of_bounds = ((x == -1 && position.0 == 0)
                || (x == 1 && position.0 + 1 == map.data.xsize()))
                || ((y == -1 && position.1 == 0) || (y == 1 && position.1 + 1 == map.data.ysize()));
            // Otherwise, we only consider neighbors that are autotiles of the same type
            let is_same_autotile = !is_out_of_bounds
                && map.data[(
                    if x == -1 {
                        position.0 - 1
                    } else {
                        position.0 + x as usize
                    },
                    if y == -1 {
                        position.1 - 1
                    } else {
                        position.1 + y as usize
                    },
                    position.2,
                )] / 48
                    == autotile;

            if is_out_of_bounds || is_same_autotile {
                bitfield |= 1
            }
        }

        // Check how many edges have valid neighbors
        autotile * 48
            + match (bitfield & 0b01010101).count_ones() {
                4 => {
                    // If the autotile is surrounded on all 4 edges,
                    // then the autotile variant is one of the first 16,
                    // depending on which corners are surrounded
                    let tl = (bitfield & 0b10000000 == 0) as u8;
                    let tr = (bitfield & 0b00100000 == 0) as u8;
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    let bl = (bitfield & 0b00000010 == 0) as u8;
                    tl | (tr << 1) | (br << 2) | (bl << 3)
                }

                3 => {
                    // Rotate the bitfield 90 degrees counterclockwise until
                    // the one edge that is not surrounded is at the left
                    let mut bitfield = bitfield;
                    let mut i = 16u8;
                    while bitfield & 0b00000001 != 0 {
                        bitfield = bitfield.rotate_left(2);
                        i += 4;
                    }
                    // Now, the variant is one of the next 16
                    let tr = (bitfield & 0b00100000 == 0) as u8;
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    i + (tr | (br << 1))
                }

                // Top and bottom edges
                2 if bitfield & 0b01000100 == 0b01000100 => 32,

                // Left and right edges
                2 if bitfield & 0b00010001 == 0b00010001 => 33,

                2 => {
                    // Rotate the bitfield 90 degrees counterclockwise until
                    // the two edges that are surrounded are at the right and bottom
                    let mut bitfield = bitfield;
                    let mut i = 34u8;
                    while bitfield & 0b00010100 != 0b00010100 {
                        bitfield = bitfield.rotate_left(2);
                        i += 2;
                    }
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    i + br
                }

                1 => {
                    // Rotate the bitfield 90 degrees clockwise until
                    // the edge is at the bottom
                    let mut bitfield = bitfield;
                    let mut i = 42u8;
                    while bitfield & 0b00000100 == 0 {
                        bitfield = bitfield.rotate_right(2);
                        i += 1;
                    }
                    i
                }

                0 => 46,

                _ => unreachable!(),
            } as i16
    }

    fn set_tile(&self, map: &mut rpg::Map, tile: SelectedTile, position: (usize, usize, usize)) {
        map.data[position] = match tile {
            SelectedTile::Autotile(i) => i * 48,
            SelectedTile::Tile(i) => i,
        };

        for y in -1i8..=1i8 {
            for x in -1i8..=1i8 {
                // Don't check tiles that are out of bounds
                if ((x == -1 && position.0 == 0) || (x == 1 && position.0 + 1 == map.data.xsize()))
                    || ((y == -1 && position.1 == 0)
                        || (y == 1 && position.1 + 1 == map.data.ysize()))
                {
                    continue;
                }
                let position = (
                    if x == -1 {
                        position.0 - 1
                    } else {
                        position.0 + x as usize
                    },
                    if y == -1 {
                        position.1 - 1
                    } else {
                        position.1 + y as usize
                    },
                    position.2,
                );
                let tile_id = self.recompute_autotile(map, position);
                map.data[position] = tile_id;
                self.view.map.set_tile(tile_id, position);
            }
        }
    }
}

impl tab::Tab for Tab {
    fn name(&self) -> String {
        let mapinfos = state!().data_cache.mapinfos();
        format!("Map {}: {}", self.id, mapinfos[&self.id].name)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_map").with(self.id)
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        // Display the toolbar.
        egui::TopBottomPanel::top(format!("map_{}_toolbar", self.id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.view.scale, 15.0..=300.)
                        .text("Scale")
                        .fixed_decimals(0),
                );

                ui.separator();

                ui.menu_button(
                    // Format the text based on what layer is selected.
                    match self.view.selected_layer {
                        SelectedLayer::Events => "Events ⏷".to_string(),
                        SelectedLayer::Tiles(layer) => format!("Layer {} ⏷", layer + 1),
                    },
                    |ui| {
                        // TODO: Add layer enable button
                        // Display all layers.
                        ui.columns(2, |columns| {
                            columns[1].visuals_mut().button_frame = true;
                            columns[0].label(egui::RichText::new("Panorama").underline());
                            columns[1].checkbox(&mut self.view.map.pano_enabled, "👁");

                            for (index, layer) in
                                self.view.map.enabled_layers.iter_mut().enumerate()
                            {
                                columns[0].selectable_value(
                                    &mut self.view.selected_layer,
                                    SelectedLayer::Tiles(index),
                                    format!("Layer {}", index + 1),
                                );
                                columns[1].checkbox(layer, "👁");
                            }

                            // Display event layer.
                            columns[0].selectable_value(
                                &mut self.view.selected_layer,
                                SelectedLayer::Events,
                                egui::RichText::new("Events").italics(),
                            );
                            columns[1].checkbox(&mut self.view.event_enabled, "👁");

                            columns[0].label(egui::RichText::new("Fog").underline());
                            columns[1].checkbox(&mut self.view.map.fog_enabled, "👁");
                        });
                    },
                );

                ui.separator();

                ui.checkbox(&mut self.view.visible_display, "Display Visible Area")
                    .on_hover_text("Display the visible area in-game (640x480)");
                ui.checkbox(&mut self.view.move_preview, "Preview event move routes")
                    .on_hover_text("Preview event page move routes");
                ui.checkbox(&mut self.view.snap_to_grid, "Snap to grid")
                    .on_hover_text("Snap's the viewport to the tile grid");
                ui.checkbox(
                    &mut self.view.darken_unselected_layers,
                    "Darken unselected layers",
                )
                .on_disabled_hover_text("Toggles darkening unselected layers");

                /*
                if ui.button("Save map preview").clicked() {
                    self.tilemap.save_to_disk();
                }

                if map.preview_move_route.is_some()
                && ui.button("Clear move route preview").clicked()
                {
                    map.preview_move_route = None;
                }
                */
            });
        });

        // Display the tilepicker.
        let spacing = ui.spacing();
        let tilepicker_default_width = 256.
            + 3. * spacing.window_margin.left
            + spacing.scroll_bar_inner_margin
            + spacing.scroll_bar_width
            + spacing.scroll_bar_outer_margin;
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(tilepicker_default_width)
            .max_width(tilepicker_default_width)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    self.tilepicker.ui(ui);
                    ui.separator();
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Get the map.
                let mut map = state!().data_cache.map(self.id);

                let response = self.view.ui(ui, &map, self.dragging_event);

                let layers_max = map.data.zsize();
                let map_x = self.view.cursor_pos.x as i32;
                let map_y = self.view.cursor_pos.y as i32;

                if let SelectedLayer::Tiles(tile_layer) = self.view.selected_layer {
                    if response.dragged_by(egui::PointerButton::Primary)
                        && !ui.input(|i| i.modifiers.command)
                    {
                        self.set_tile(
                            &mut map,
                            self.tilepicker.selected_tile,
                            (map_x as usize, map_y as usize, tile_layer),
                        );
                    }
                } else {
                    if let Some(selected_event_id) = self.view.selected_event_id {
                        if let Some(selected_event) = map.events.get_mut(selected_event_id) {
                            // Double-click on events to edit them
                            if response.double_clicked() {
                                self.event_windows.add_window(event_edit::Window::new(
                                    selected_event_id,
                                    self.id,
                                ));
                            }
                            // Press delete or backspace to delete the selected event
                            else if response.hovered()
                                && ui.memory(|m| m.focus().is_none())
                                && ui.input(|i| {
                                    i.key_pressed(egui::Key::Delete)
                                        || i.key_pressed(egui::Key::Backspace)
                                })
                            {
                                map.events.remove(selected_event_id);
                                self.view.events.try_remove(selected_event_id);
                            }
                        }
                    }
                }
            })
        });

        self.event_windows.update(ui.ctx());
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
