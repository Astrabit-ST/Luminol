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

use crate::image_cache::GlTexture;
use crate::prelude::*;
use glow::HasContext;

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
    tileset_tex: Option<Arc<GlTexture>>,
    autotile_texs: Vec<Option<Arc<GlTexture>>>,
    event_texs: HashMap<String, Arc<GlTexture>>,
    fog_tex: Option<Arc<GlTexture>>,
    pano_tex: Option<Arc<GlTexture>>,
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

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
        if let Some(ref tex) = self.textures.tileset_tex {
            let (canvas_rect, response) =
                ui.allocate_exact_size(tex.size_vec2(), egui::Sense::click());

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
        let state = state!();
        // Load the map.

        let map = state.data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        // Load tileset textures.
        let tileset_tex = if tileset.tileset_name.is_empty() {
            None
        } else {
            unsafe {
                state
                    .image_cache
                    .load_glow_image("Graphics/Tilesets", "tileset.tileset_name")
            }
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
                    unsafe {
                        state
                            .image_cache
                            .load_glow_image("Graphics/Autotiles", s)
                            .map(Some)
                    }
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
            .map(|char_name| unsafe {
                state
                    .image_cache
                    .load_glow_image("Graphics/Characters", &char_name)
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex = unsafe {
            state
                .image_cache
                .load_glow_image("Graphics/Fogs", &tileset.fog_name)
        }
        .ok();

        let pano_tex = unsafe {
            state
                .image_cache
                .load_glow_image("Graphics/Panoramas", &tileset.panorama_name)
        }
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
