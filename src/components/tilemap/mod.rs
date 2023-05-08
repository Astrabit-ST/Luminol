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
mod atlas;
mod shader;
mod vertices;

use atlas::Atlas;
use shader::Shader;
use vertices::TileVertices;

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
    tile_vertices: Arc<TileVertices>,
}

struct Textures {
    atlas: Atlas,
    event_texs: HashMap<String, Arc<WgpuTexture>>,
    fog_tex: Option<Arc<WgpuTexture>>,
    pano_tex: Option<Arc<WgpuTexture>>,
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

const MAX_SIZE: u32 = 8192; // Max texture size in one dimension
const TILE_SIZE: u32 = 32; // Tiles are 32x32
const TILESET_WIDTH: u32 = TILE_SIZE * 8; // Tilesets are 8 tiles across

const AUTOTILE_HEIGHT: u32 = TILE_SIZE * 4; // Autotiles are 4 tiles high
const AUTOTILE_AMOUNT: u32 = 7; // There are 7 autotiles per tileset
const TOTAL_AUTOTILE_HEIGHT: u32 = AUTOTILE_HEIGHT * AUTOTILE_AMOUNT;
const UNDER_HEIGHT: u32 = MAX_SIZE - TOTAL_AUTOTILE_HEIGHT;

/// Hardcoded list of tiles from r48 and old python Luminol.
/// There seems to be very little pattern in autotile IDs so this is sadly
/// the best we can do.
const AUTOTILES: [[i32; 4]; 48] = [
    [26, 27, 32, 33],
    [4, 27, 32, 33],
    [26, 5, 32, 33],
    [4, 5, 32, 33],
    [26, 27, 32, 11],
    [4, 27, 32, 11],
    [26, 5, 32, 11],
    [4, 5, 32, 11],
    [26, 27, 10, 33],
    [4, 27, 10, 33],
    [26, 5, 10, 33],
    [4, 5, 10, 33],
    [26, 27, 10, 11],
    [4, 27, 10, 11],
    [26, 5, 10, 11],
    [4, 5, 10, 11],
    [24, 25, 30, 31],
    [24, 5, 30, 31],
    [24, 25, 30, 11],
    [24, 5, 30, 11],
    [14, 15, 20, 21],
    [14, 15, 20, 11],
    [14, 15, 10, 21],
    [14, 15, 10, 11],
    [28, 29, 34, 35],
    [28, 29, 10, 35],
    [4, 29, 34, 35],
    [4, 29, 10, 35],
    [38, 39, 44, 45],
    [4, 39, 44, 45],
    [38, 5, 44, 45],
    [4, 5, 44, 45],
    [24, 29, 30, 35],
    [14, 15, 44, 45],
    [12, 13, 18, 19],
    [12, 13, 18, 11],
    [16, 17, 22, 23],
    [16, 17, 10, 23],
    [40, 41, 46, 47],
    [4, 41, 46, 47],
    [36, 37, 42, 43],
    [36, 5, 42, 43],
    [12, 17, 18, 23],
    [12, 13, 42, 43],
    [36, 41, 42, 47],
    [16, 17, 46, 47],
    [12, 17, 42, 47],
    [0, 1, 6, 7],
];

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        // Load the map.
        let map = state!().data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state!().data_cache.tilesets();
        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let textures = Arc::new(Self::load_data(&map, tileset)?);

        let vertex_buffer = TileVertices::new(&map, &textures.atlas);
        let vertex_buffer = Arc::new(vertex_buffer);

        Ok(Self {
            pan: egui::Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,

            textures,
            tile_vertices: vertex_buffer,
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

        let textures = self.textures.clone();
        let tile_vertices = self.tile_vertices.clone();
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |device, queue, _encoder, paint_callback_resources| {
                        //
                        paint_callback_resources.insert(textures.clone());
                        paint_callback_resources.insert(tile_vertices.clone());
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                        let textures: &Arc<Textures> = paint_callback_resources
                            .get()
                            .expect("failed to get tileset textures");
                        let tile_vertices: &Arc<TileVertices> = paint_callback_resources
                            .get()
                            .expect("failed to get vertex buffer");

                        Shader::bind(render_pass);
                        render_pass.set_bind_group(0, &textures.atlas.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, tile_vertices.buffer.slice(..));

                        render_pass.draw(0..tile_vertices.vertices, 0..1);
                    }),
            ),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(
                TILESET_WIDTH as f32,
                self.textures.atlas.tileset_height as f32,
            ),
            egui::Sense::click(),
        );

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
    fn load_data(map: &rpg::Map, tileset: &rpg::Tileset) -> Result<Textures, String> {
        let state = state!();

        let atlas = Atlas::new(tileset)?;

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
}
