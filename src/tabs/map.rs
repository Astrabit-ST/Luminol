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
    load_image_software, UpdateInfo,
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
            tilemap: Tilemap::new(),
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

            // Display the toolbar.
            // self.toolbar(ui, &mut map);

            // Display the tilepicker.
            egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
                .default_width(256.)
                .show_inside(ui, |ui| {
                    egui::ScrollArea::both().show(ui, |ui| {
                        self.tilemap
                            .tilepicker(ui, &textures, &mut self.selected_tile);
                    });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let response = self.tilemap.ui(
                        ui,
                        &map,
                        &mut self.cursor_pos,
                        &textures,
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
}

impl Map {
    async fn load_data(info: &'static UpdateInfo, id: i32) -> Result<Textures, String> {
        // Load the map.
        let map = info
            .data_cache
            .load_map(&info.filesystem, id)
            .await?;
        // Get tilesets.
        let tilesets = info.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets.as_ref().ok_or("Tilesets not loaded".to_string())?
            [map.tileset_id as usize - 1];

        // Load tileset textures.
        let tileset_tex = load_image_software(
            format!("Graphics/Tilesets/{}", tileset.tileset_name),
            0,
            &info.filesystem,
        )
        .await?;

        // Create an async iter over the autotile textures.
        let autotile_texs_iter = tileset.autotile_names.iter().map(|str| async move {
            load_image_software(
                format!("Graphics/Autotiles/{}", str),
                0,
                &info.filesystem,
            )
            .await
            .ok()
        });

        // Await all the futures.
        let autotile_texs = futures::future::join_all(autotile_texs_iter).await;

        // Similar deal as to the autotiles.
        let event_texs_iter = map.events.iter().map(|(_, e)| async {
            let graphic = &e.pages[0].graphic;
            let char_name = graphic.character_name.clone();

            (
                (char_name.clone(), graphic.character_hue),
                load_image_software(
                    format!("Graphics/Characters/{}", char_name),
                    graphic.character_hue,
                    &info.filesystem,
                )
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
        let fog_tex = load_image_software(
            format!("Graphics/Fogs/{}", tileset.fog_name),
            tileset.fog_hue,
            &info.filesystem,
        )
        .await
        .ok();

        let pano_tex = load_image_software(
            format!("Graphics/Panoramas/{}", tileset.panorama_name),
            tileset.panorama_hue,
            &info.filesystem,
        )
        .await
        .ok();

        // Finally create and return the struct.
        Ok(Textures {
            autotile_texs,
            tileset_tex,
            event_texs,
            fog_tex,
            fog_zoom: tileset.fog_zoom,
            pano_tex,
        })
    }
}
