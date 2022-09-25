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

use egui::Pos2;
use egui_extras::RetainedImage;
use ndarray::Axis;

use crate::{components::tilemap::Tilemap, load_image, UpdateInfo};

pub struct Map {
    pub id: i32,
    pub name: String,
    pub selected_layer: usize,
    pub toggled_layers: Vec<bool>,
    pub cursor_pos: Pos2,
    pub tilemap: Tilemap,
    pub selected_tile: i16,
    tileset_tex: RetainedImage,
    autotile_texs: Vec<Option<RetainedImage>>,
    event_texs: HashMap<String, Option<RetainedImage>>,
}

impl Map {
    pub fn new(id: i32, name: String, info: &UpdateInfo<'_>) -> Option<Self> {
        // Get the map.
        let map = match info.data_cache.load_map(info.filesystem, id) {
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

        let layers_max = map.data.len_of(Axis(0)) + 1;
        Some(Self {
            id,
            name,
            selected_layer: layers_max,
            toggled_layers: vec![true; layers_max],
            cursor_pos: Pos2::ZERO,
            tilemap: Tilemap::new(),
            selected_tile: 0,
            tileset_tex,
            autotile_texs,
            event_texs,
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
        let mut map = match info.data_cache.load_map(info.filesystem, self.id) {
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
                    let (rect, response) =
                        ui.allocate_exact_size(self.tileset_tex.size_vec2(), egui::Sense::click());

                    egui::Image::new(
                        self.tileset_tex.texture_id(ui.ctx()),
                        self.tileset_tex.size_vec2(),
                    )
                    .paint_at(ui, rect);

                    if response.clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let mut pos = (pos - rect.min) / 32.;
                            pos.x = pos.x.floor();
                            pos.y = pos.y.floor();
                            self.selected_tile = (pos.x + pos.y * 8.) as i16;
                        }
                    }
                    let cursor_x = self.selected_tile % 8 * 32;
                    let cursor_y = self.selected_tile / 8 * 32;
                    ui.painter().rect_stroke(
                        egui::Rect::from_min_size(
                            rect.min + egui::vec2(cursor_x as f32, cursor_y as f32),
                            egui::Vec2::splat(32.),
                        ),
                        5.0,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let response = self.tilemap.ui(
                    ui,
                    &map,
                    &mut self.cursor_pos,
                    &self.tileset_tex,
                    &self.autotile_texs,
                    &self.event_texs,
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
}
