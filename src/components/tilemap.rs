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

use crate::image_cache::WgpuTexture;
use crate::prelude::*;

pub struct Tilemap {
    /// The tilemap pan.
    pub pan: egui::Vec2,
    /// The scale of the tilemap.
    pub scale: f32,
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    textures: Arc<Textures>,
}

impl Drop for Tilemap {
    fn drop(&mut self) {}
}

struct Textures {
    atlas: WgpuTexture,
    event_texs: HashMap<String, Arc<WgpuTexture>>,
    fog_tex: Option<Arc<WgpuTexture>>,
    pano_tex: Option<Arc<WgpuTexture>>,
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

const MAX_SIZE: i32 = 8192; // Max texture size in one dimension
const TILE_SIZE: i32 = 32; // Tiles are 32x32
const TILESET_WIDTH: i32 = TILE_SIZE * 8; // Tilesets are 8 tiles across

const ANIM_FRAME_COUNT: i32 = 4; // Autotiles have 4 frames of animation
const AUTOTILE_WIDTH: i32 = TILE_SIZE * 3 * ANIM_FRAME_COUNT; // Each frame is 3 tiles wide
const AUTOTILE_HEIGHT: i32 = TILE_SIZE * 4; // Autotiles are 4 tiles high
const AUTOTILE_AMOUNT: i32 = 7; // There are 7 autotiles per tileset
const TOTAL_AUTOTILE_HEIGHT: i32 = AUTOTILE_HEIGHT * AUTOTILE_AMOUNT;

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        let textures = Arc::new(Self::load_data(id)?);

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
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(|device, queue, _encoder, paint_callback_resources| {
                        //
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                    }),
            ),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) =
            ui.allocate_exact_size(self.textures.atlas.size_vec2(), egui::Sense::click());

        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(|device, queue, _encoder, paint_callback_resources| {
                        //
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                    }),
            ),
        });
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

        let atlas = Self::build_atlas(tileset)?;

        let event_texs = map
            .events
            .iter()
            .filter_map(|(_, e)| e.pages.first().map(|p| p.graphic.character_name.clone()))
            .filter(|s| !s.is_empty())
            .dedup()
            .map(|char_name| {
                //
                state
                    .image_cache
                    .load_wgpu_image("Graphics/Characters", &char_name)
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Fogs", &tileset.fog_name)
            .ok();

        let pano_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Panoramas", &tileset.panorama_name)
            .ok();

        // Finally create and return the struct.
        Ok(Textures {
            atlas,
            event_texs,
            fog_tex,
            pano_tex,
        })
    }

    fn calc_atlas_dimensions(
        tileset: &rpg::Tileset,
        tileset_height: i32,
    ) -> Result<(i32, i32), String> {
        let mut width = AUTOTILE_WIDTH;
        let mut height = TOTAL_AUTOTILE_HEIGHT;
        println!("initial size {width}x{height}");
        height += tileset_height;
        println!("tilemap + initial size {width}x{height}");

        while height > MAX_SIZE {
            width += TILESET_WIDTH;
            height -= height % 8192;
            println!("resizing to {width}x{height}");
        }

        if width > MAX_SIZE || height > MAX_SIZE {
            Err("cannot fit tileset into an 8192x8192 texture".to_string())
        } else {
            Ok((width, height))
        }
    }

    fn build_atlas(tileset: &rpg::Tileset) -> Result<WgpuTexture, String> {
        Err("nyi".to_string())
    }
}
