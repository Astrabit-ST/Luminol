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

use std::collections::HashMap;

use egui_extras::RetainedImage;

use crate::{components::tilemap::Tilemap, load_image, UpdateInfo};

pub struct Map {
    pub id: i32,
    pub name: String,
    pub selected_layer: usize,
    pub tilemap: Tilemap,
    tileset_tex: RetainedImage,
    autotile_texs: Vec<Option<RetainedImage>>,
    event_texs: HashMap<String, Option<RetainedImage>>,
}

impl Map {
    pub fn new(id: i32, name: String, info: &UpdateInfo<'_>) -> Self {
        // Get the map.
        let map = info.data_cache.load_map(info.filesystem, id);
        // Get tilesets.
        let tilesets = info.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets.as_ref().expect("Tilesets not loaded")[map.tileset_id as usize - 1];

        // Load tileset textures.
        let tileset_tex = load_image(
            format!("Graphics/Tilesets/{}", tileset.tileset_name),
            info.filesystem,
        )
        .unwrap();
        let autotile_texs = tileset
            .autotile_names
            .iter()
            .map(|str| load_image(format!("Graphics/Autotiles/{}", str), info.filesystem).ok())
            .collect();

        let event_texs = map
            .events
            .iter()
            .map(|(_, e)| {
                let char_name = e.pages[0].graphic.character_name.clone();

                (
                    char_name.clone(),
                    load_image(
                        format!("Graphics/Characters/{}", char_name),
                        info.filesystem,
                    )
                    .ok(),
                )
            })
            .collect();

        Self {
            id,
            name,
            selected_layer: 0,
            tilemap: Tilemap::new(),
            tileset_tex,
            autotile_texs,
            event_texs,
        }
    }
}

impl super::tab::Tab for Map {
    fn name(&self) -> String {
        format!("Map {}: {}", self.id, self.name)
    }

    #[allow(unused_variables, unused_mut)]
    fn show(&mut self, ui: &mut egui::Ui, info: &crate::UpdateInfo<'_>) {
        // Get the map.
        let mut map = info.data_cache.load_map(info.filesystem, self.id);

        // Display the toolbar.
        self.toolbar(ui, &mut map);

        // Display the tilepicker.
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(256.)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    self.tileset_tex.show(ui);
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            self.tilemap.ui(
                ui,
                &mut map,
                self.id,
                &self.tileset_tex,
                &self.autotile_texs,
                &self.event_texs,
            )
        });
    }
}
