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
use parking_lot::MappedMutexGuard;
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
    textures: Textures,
}

impl Map {
    pub fn new(id: i32, name: String, info: &UpdateInfo<'_>) -> Option<Self> {
        // Get the map.
        let map = match info.data_cache.load_map(info.filesystem.clone(), id) {
            Ok(m) => m,
            Err(e) => {
                info.toasts.error(e);
                return None;
            }
        };
        // Get tilesets.
        let tilesets = info.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets.as_ref().expect("Tilesets not loaded")[map.tileset_id as usize - 1];
        let layers_max = map.data.len_of(Axis(0)) + 1;

        Some(Self {
            id,
            name,
            selected_layer: layers_max,
            toggled_layers: vec![true; layers_max],
            cursor_pos: Pos2::ZERO,
            tilemap: Tilemap::new(),
            selected_tile: 0,
            textures: Self::load_textures(map, tileset, info),
        })
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    #[allow(unused_variables, unused_mut)]
    fn show(&mut self, ui: &mut egui::Ui, info: &crate::UpdateInfo<'_>) {
        // Get the map.
        let mut map = match info.data_cache.load_map(info.filesystem.clone(), self.id) {
            Ok(m) => m,
            Err(e) => {
                info.toasts.error(e);
                return;
            }
        };

        // Display the toolbar.
        self.toolbar(ui, &mut map);

        // Display the tilepicker.
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(256.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    self.tilemap
                        .tilepicker(ui, &self.textures, &mut self.selected_tile);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let response = self.tilemap.ui(
                    ui,
                    &map,
                    &mut self.cursor_pos,
                    &self.textures,
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
    }

    #[cfg(feature = "discord-rpc")]
    fn discord_display(&self) -> String {
        format!("Editing Map<{}>: {}", self.id, self.name)
    }
}

impl Map {
    fn load_textures(
        map: MappedMutexGuard<'_, rpg::Map>,
        tileset: &rpg::Tileset,
        info: &UpdateInfo<'_>,
    ) -> Textures {
        // Load tileset textures.
        let tileset_tex = load_image_software(
            format!("Graphics/Tilesets/{}", tileset.tileset_name),
            0,
            info.filesystem.clone(),
        )
        .unwrap();
        let autotile_texs: Vec<_> = tileset
            .autotile_names
            .iter()
            .map(|str| {
                load_image_software(
                    format!("Graphics/Autotiles/{}", str),
                    0,
                    info.filesystem.clone(),
                )
                .ok()
            })
            .collect();

        let event_texs: HashMap<_, _> = map
            .events
            .iter()
            .map(|(_, e)| {
                let graphic = &e.pages[0].graphic;
                let char_name = graphic.character_name.clone();

                (
                    (char_name.clone(), graphic.character_hue),
                    load_image_software(
                        format!("Graphics/Characters/{}", char_name),
                        graphic.character_hue,
                        info.filesystem.clone(),
                    )
                    .ok(),
                )
            })
            .collect();

        let fog_tex = load_image_software(
            format!("Graphics/Fogs/{}", tileset.fog_name),
            tileset.fog_hue,
            info.filesystem.clone(),
        )
        .ok();

        let pano_tex = load_image_software(
            format!("Graphics/Panoramas/{}", tileset.panorama_name),
            tileset.panorama_hue,
            info.filesystem.clone(),
        )
        .ok();

        Textures {
            autotile_texs,
            tileset_tex,
            event_texs,
            fog_tex,
            fog_zoom: tileset.fog_zoom,
            pano_tex,
        }
    }
}
