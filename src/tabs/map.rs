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

    fn set_tile(&self, map: &mut rpg::Map, tile_id: i16, position: (usize, usize, usize)) {
        map.data[position] = tile_id;
        self.view.map.set_tile(tile_id, position);
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
        let state = state!();

        // Get the map.
        let mut map = state.data_cache.map(self.id);

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
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(256.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    self.tilepicker.ui(ui);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
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
                            match self.tilepicker.selected_tile {
                                SelectedTile::Autotile(tile) => tile * 48,
                                SelectedTile::Tile(tile) => tile,
                            },
                            (map_x as usize, map_y as usize, tile_layer),
                        );
                    }
                }

                if ui.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                }) {
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

        self.event_windows.update(ui.ctx());
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
