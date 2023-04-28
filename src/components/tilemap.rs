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
#![allow(unsafe_code)]

use std::sync::Arc;

use crate::prelude::*;
use crate::Texture;
use glow::HasContext as _;
use itertools::Itertools;

pub struct Tilemap {
    /// The tilemap pan.
    pub pan: egui::Vec2,
    /// The scale of the tilemap.
    pub scale: f32,
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    textures: Textures,
}

struct Textures {
    tileset_tex: Option<Texture>,
    autotile_texs: Vec<Option<Texture>>,
    event_texs: HashMap<String, Texture>,
    fog_tex: Option<Texture>,
    pano_tex: Option<Texture>,
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

impl Drop for Textures {
    fn drop(&mut self) {
        unsafe {
            let gl = state!().gl.as_ref();
            if let Some(tex) = self.tileset_tex {
                gl.delete_texture(tex.raw)
            }
            for tex in self.autotile_texs.iter().flatten().copied() {
                gl.delete_texture(tex.raw)
            }
            for tex in self.event_texs.values().copied() {
                gl.delete_texture(tex.raw)
            }
            if let Some(tex) = self.fog_tex {
                gl.delete_texture(tex.raw)
            }
            if let Some(tex) = self.pano_tex {
                gl.delete_texture(tex.raw)
            }
        }
    }
}

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        let textures = Self::load_data(id)?;
        Ok(Self {
            pan: egui::Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,

            textures,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut egui::Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
        dragging_event: bool,
    ) -> egui::Response {
        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(eframe::egui_glow::CallbackFn::new(|_i, painter| {
                let gl = painter.gl();
            })),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        if let Some(tex) = self.textures.tileset_tex {
            let (canvas_rect, response) = ui.allocate_exact_size(
                egui::vec2(tex.width as f32, tex.height as f32),
                egui::Sense::click(),
            );

            ui.painter().add(egui::PaintCallback {
                rect: canvas_rect,
                callback: Arc::new(eframe::egui_glow::CallbackFn::new(|_i, painter| {
                    let gl = painter.gl();
                })),
            });
        }
    }

    #[allow(unused_variables, unused_assignments)]
    fn load_data(id: i32) -> Result<Textures, String> {
        let info = state!();
        // Load the map.

        let map = info.data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = info.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        // Load tileset textures.
        let tileset_tex = if tileset.tileset_name.is_empty() {
            None
        } else {
            crate::load_image_hardware(format!("Graphics/Tilesets/{}", tileset.tileset_name))
                .map(Some)?
        };

        // Create an async iter over the autotile textures.
        let autotile_texs = tileset
            .autotile_names
            .iter()
            .map(|s| {
                if s.is_empty() {
                    Ok(None)
                } else {
                    crate::load_image_hardware(format!("Graphics/Autotiles/{s}")).map(Some)
                }
            })
            .try_collect()?;

        // Await all the futures.

        let event_texs = map
            .events
            .iter()
            .filter_map(|(_, e)| e.pages.first().map(|p| p.graphic.character_name.clone()))
            .filter(|s| !s.is_empty())
            .dedup()
            .map(|char_name| {
                crate::load_image_hardware(format!("Graphics/Characters/{char_name}"))
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex =
            crate::load_image_hardware(format!("Graphics/Fogs/{}", tileset.fog_name)).ok();

        let pano_tex =
            crate::load_image_hardware(format!("Graphics/Panoramas/{}", tileset.panorama_name))
                .ok();

        // Finally create and return the struct.
        Ok(Textures {
            tileset_tex,
            autotile_texs,
            event_texs,
            fog_tex,

            pano_tex,
        })
    }
}
