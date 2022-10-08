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
    components::tilemap::{Textures, Tilemap},
    data::rmxp_structs::rpg,
    load_image_hardware, load_image_software, UpdateInfo,
};

pub struct Map {
    pub id: i32,
    pub name: String,
    pub selected_layer: usize,
    pub toggled_layers: Vec<bool>,
    pub cursor_pos: Pos2,
    pub tilemap: Tilemap,
    pub selected_tile: i16,
    textures: Promise<Textures>,
}

impl Map {
    pub fn new(id: i32, name: String, info: &'static UpdateInfo) -> Option<Self> {
        Some(Self {
            id,
            name,
            selected_layer: 0,
            toggled_layers: Vec::new(),
            cursor_pos: Pos2::ZERO,
            tilemap: Tilemap::new(info),
            selected_tile: 0,
            textures: Promise::spawn_local(async move { Self::load_data(info, id).await.unwrap() }),
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
        if let Some(textures) = self.textures.ready() {
            // Get the map.
            let mut map = info.data_cache.get_map(self.id);

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
                        self.tilemap
                            .tilepicker(ui, textures, &mut self.selected_tile);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let response = self.tilemap.ui(
                        ui,
                        &map,
                        &mut self.cursor_pos,
                        textures,
                        &self.toggled_layers,
                        self.selected_layer,
                    );

                    let layers_max = map.data.len_of(Axis(0));
                    if response.dragged()
                        && self.selected_layer < layers_max
                        && !ui.input().modifiers.command
                    {
                        let map_x = self.cursor_pos.x as usize;
                        let map_y = self.cursor_pos.y as usize;
                        map.data[[self.selected_layer, map_y, map_x]] = self.selected_tile + 384;
                    }
                })
            });
        } else {
            // If not, just display a spinner.
            ui.centered_and_justified(|ui| {
                ui.spinner();
            });
        }
    }

    #[cfg(feature = "discord-rpc")]
    fn discord_display(&self) -> String {
        format!("Editing Map<{}>: {}", self.id, self.name)
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

impl Map {
    #[allow(unused_variables, unused_assignments)]
    async fn load_data(info: &'static UpdateInfo, id: i32) -> Result<Textures, String> {
        // Load the map.
        let tileset_name;
        let autotile_names;

        let event_names: Vec<_>;

        let fog_name;
        let fog_hue;
        let fog_zoom;

        let pano_name;
        let pano_hue;

        // We get all the variables we need from here so we don't borrow the refcell across an await.
        // This could be done with a RwLock or a Mutex but this is more practical.
        // We do have to clone variables but this is negligble compared to the alternative.
        {
            let map = info.data_cache.load_map(&info.filesystem, id).await?;
            // Get tilesets.
            let tilesets = info.data_cache.tilesets();

            // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
            let tileset = &tilesets
                .as_ref()
                .ok_or_else(|| "Tilesets not loaded".to_string())?[map.tileset_id as usize - 1];

            tileset_name = tileset.tileset_name.clone();
            autotile_names = tileset.autotile_names.clone();

            event_names = map
                .events
                .values()
                .map(|e| {
                    (
                        e.pages[0].graphic.character_name.clone(),
                        e.pages[0].graphic.character_hue,
                    )
                })
                .collect();

            fog_name = tileset.fog_name.clone();
            fog_hue = tileset.fog_hue;
            fog_zoom = tileset.fog_zoom;

            pano_name = tileset.panorama_name.clone();
            pano_hue = tileset.panorama_hue;
        }

        // Load tileset textures.
        let tileset_tex =
            load_image_hardware(format!("Graphics/Tilesets/{}", tileset_name), info).await?;

        // Create an async iter over the autotile textures.
        let autotile_texs_iter = autotile_names.iter().map(|str| async move {
            load_image_hardware(format!("Graphics/Autotiles/{}", str), info)
                .await
                .ok()
        });

        // Await all the futures.
        let autotile_texs = futures::future::join_all(autotile_texs_iter).await;

        // Similar deal as to the autotiles.
        let event_texs_iter = event_names.iter().map(|(char_name, hue)| async move {
            (
                (char_name.clone(), *hue),
                load_image_hardware(format!("Graphics/Characters/{}", char_name), info)
                    .await
                    .ok(),
            )
        });

        // Unfortunately since join_all produces a vec, we need to convert it to a hashmap.
        let event_texs: HashMap<_, _> = futures::future::join_all(event_texs_iter)
            .await
            .into_iter()
            .collect();

        // These two are pretty simple.
        let fog_tex = load_image_hardware(format!("Graphics/Fogs/{}", fog_name), info)
            .await
            .ok();

        let pano_tex = load_image_hardware(format!("Graphics/Panoramas/{}", pano_name), info)
            .await
            .ok();

        // Finally create and return the struct.
        Ok(Textures {
            autotile_texs,
            tileset_tex,
            event_texs,
            fog_tex,
            fog_zoom,
            pano_tex,
        })
    }
}
