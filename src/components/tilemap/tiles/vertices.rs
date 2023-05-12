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

use super::{
    autotiles::AUTOTILES, AUTOTILE_AMOUNT, AUTOTILE_HEIGHT, MAX_SIZE, TILESET_WIDTH,
    TOTAL_AUTOTILE_HEIGHT, UNDER_HEIGHT,
};
use super::{Atlas, Quad};
use crate::prelude::*;

#[derive(Debug)]
pub struct Vertices {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices: u32,
}

impl Vertices {
    pub fn new(map: &rpg::Map, atlas: &Atlas) -> Self {
        let mut quads = Vec::with_capacity(map.data.len());
        for (index, tile) in map.data.iter().copied().enumerate() {
            // We reset the x every xsize elements.
            let map_x = index % map.data.xsize();
            // We reset the y every ysize elements, but only increment it every xsize elements.
            let map_y = (index / map.data.xsize()) % map.data.ysize();
            // We change the z every xsize * ysize elements.
            let map_z = index / (map.data.xsize() * map.data.ysize());
            // let map_z = map.data.zsize() - map_z;

            // There are 4 cases we need to handle here:
            match tile {
                // The tile is blank
                0..=47 => continue,
                // The tile is an autotile
                48..=384 => {
                    let autotile_id = (tile / 48) - 1;
                    for s_a in 0..2 {
                        for s_b in 0..2 {
                            let pos = egui::Rect::from_min_size(
                                egui::pos2(
                                    map_x as f32 * 32. + (s_a as f32 * 16.),
                                    map_y as f32 * 32. + (s_b as f32 * 16.),
                                ),
                                egui::vec2(16., 16.),
                            );

                            let ti = AUTOTILES[tile as usize % 48][s_a + (s_b * 2)];
                            // let tile_x = ti % 6;
                            let tile_x = (ti % 6 * 16) as f32;

                            let tile_y = (ti / 6 * 16) as f32
                                + (AUTOTILE_HEIGHT * autotile_id as u32) as f32;

                            let tex_coords = egui::Rect::from_min_size(
                                egui::pos2(tile_x, tile_y),
                                egui::vec2(16., 16.),
                            );

                            quads.push(Quad::new(
                                pos,
                                tex_coords,
                                map_z as f32 / map.data.zsize() as f32,
                            ));
                        }
                    }
                }
                // The tileset does not wrap
                tile if atlas.tileset_height + TOTAL_AUTOTILE_HEIGHT <= MAX_SIZE => {
                    let tile = tile - 384;

                    let pos = egui::Rect::from_min_size(
                        egui::pos2(map_x as f32 * 32., map_y as f32 * 32.),
                        egui::vec2(32., 32.),
                    );

                    let tile_x = (tile % 8) as f32 * 32.;
                    let tile_y = (tile as u32 / 8 + AUTOTILE_AMOUNT * 4) as f32 * 32.;
                    let tex_coords =
                        egui::Rect::from_min_size(egui::pos2(tile_x, tile_y), egui::vec2(32., 32.));

                    quads.push(Quad::new(
                        pos,
                        tex_coords,
                        map_z as f32 / map.data.zsize() as f32,
                    ));
                }
                // The tileset *does* wrap
                tile => {
                    let tile = tile - 384;

                    let pos = egui::Rect::from_min_size(
                        egui::pos2(map_x as f32 * 32., map_y as f32 * 32.),
                        egui::vec2(32., 32.),
                    );

                    let tile_x = (tile as u32 % 8) * 32;
                    let tile_y = (tile as u32 / 8 + AUTOTILE_AMOUNT * 4) * 32;

                    let h1 = AUTOTILE_HEIGHT + UNDER_HEIGHT * atlas.columns_under;
                    let mut wrapped_x;
                    let mut wrapped_y;
                    if tile_y < h1 {
                        // you're underneath the autotiles
                        wrapped_x =
                            tile_x + (tile_y - AUTOTILE_HEIGHT) / UNDER_HEIGHT * TILESET_WIDTH;
                        wrapped_y = AUTOTILE_HEIGHT + (tile_y - AUTOTILE_HEIGHT) % UNDER_HEIGHT;
                    } else {
                        // you're to the right
                        // first wrap immediately to the right of the autotiles
                        wrapped_x = tile_x + atlas.autotile_width;
                        wrapped_y = tile_y - h1;
                        // then wrap further
                        wrapped_x += wrapped_y / MAX_SIZE * TILESET_WIDTH;
                        wrapped_y %= MAX_SIZE;
                    }

                    let tex_coords = egui::Rect::from_min_size(
                        egui::pos2(wrapped_x as f32, wrapped_y as f32),
                        egui::vec2(32., 32.),
                    );

                    quads.push(Quad::new(
                        pos,
                        tex_coords,
                        map_z as f32 / map.data.zsize() as f32,
                    ));
                }
            }
        }
        let (index_buffer, vertex_buffer, indices) =
            Quad::into_buffer(&quads, atlas.atlas_texture.size());

        Vertices {
            vertex_buffer,
            index_buffer,
            indices,
        }
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}
