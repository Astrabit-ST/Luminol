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

use super::{Atlas, AUTOTILE_AMOUNT, MAX_SIZE, TILESET_WIDTH, TOTAL_AUTOTILE_HEIGHT, UNDER_HEIGHT};
use crate::prelude::*;

use wgpu::util::DeviceExt;

pub struct TileVertices {
    pub buffer: wgpu::Buffer,
    pub vertices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
    pub const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl TileVertices {
    pub fn new(map: &rpg::Map, atlas: &Atlas) -> Self {
        let render_state = &state!().render_state;

        let mut vertices: Vec<Vertex> = vec![];

        let tile_width = 32. / atlas.atlas_texture.width() as f32;
        let tile_height = 32. / atlas.atlas_texture.height() as f32;
        for (index, tile_id) in map.data.iter().copied().enumerate() {
            let tile_id = tile_id as u32;
            if tile_id < 48 {
                continue;
            }

            // We reset the x every xsize elements.
            let x = (index % map.data.xsize()) as f32;
            // We reset the y every ysize elements, but only increment it every xsize elements.
            let y = ((index / map.data.xsize()) % map.data.ysize()) as f32;
            // We change the z every xsize * ysize elements.
            let z = (index / (map.data.xsize() * map.data.ysize())) as f32;

            let x = x - map.data.xsize() as f32;
            let y = map.data.ysize() as f32 - y;

            if tile_id >= 384 {
                // These are the coordinates of the tile we want, divided by 32
                let tex_x;
                let tex_y;
                // If we can fit the tileset into MAX_SIZE, we don't need complex math
                if TOTAL_AUTOTILE_HEIGHT + atlas.tileset_height < MAX_SIZE {
                    let tile_id = tile_id - 384;
                    // tile 384 starts at 0, AUTOTILE_AMOUNT * 4 (autotiles are 4 high!)
                    tex_x = tile_id % 8;
                    tex_y = (tile_id / 8) + (AUTOTILE_AMOUNT * 4);
                } else {
                    // The tile atlas is laid out like this:
                    /*
                    ? *------------------*-----------------*
                    ? |   Autotiles      |     |     |     |
                    ? |                  |     |     |     |
                    ? |                  |     |     |     |
                    ? |                  |     |     |     |
                    ? *------------------*     |     |     |
                    ? |     |     |     ||     |     |     |
                    ? |     |     |     ||     |     |     |
                    ? |     |     |     ||     |     |     |
                    ? |     |     |     ||     |     |     |
                    ? *------------------------------------*
                    ! The autotile region is TOTAL_AUTOTILE_HEIGHT * atlas.autotile_width
                    ! The tilesets under autotiles start at TOTAL_AUTOTILE_HEIGHT, and are UNDER_HEIGHT * TILESET_WIDTH
                    */
                    // Figure out how many rows are under the autotiles
                    let rows_under = u32::min(
                        atlas.tileset_height.div_ceil(UNDER_HEIGHT),
                        atlas.autotile_width.div_ceil(TILESET_WIDTH),
                    );
                    // Figure out how many rows are next to the autotiles
                    let rows_side = (atlas
                        .tileset_height
                        .saturating_sub(rows_under * UNDER_HEIGHT))
                    .div_ceil(MAX_SIZE);
                    tex_x = tile_id % 8;
                    tex_y = (tile_id / 8) + (AUTOTILE_AMOUNT * 4);
                }
                // Convert from pixel coords to texture coords
                let tex_x = tex_x as f32 * tile_width;
                let tex_y = tex_y as f32 * tile_height;

                // Tiles are made like this:
                // C-----D
                // | \ / |
                // | / \ |
                // A-----B

                // FIRST TRIANGLE
                // C
                // | \
                // |   \
                // A-----B

                // A
                vertices.push(Vertex {
                    position: [x, y, z],
                    tex_coords: [tex_x, tex_y + tile_height],
                });
                // B
                vertices.push(Vertex {
                    position: [x + 1., y, z],
                    tex_coords: [tex_x + tile_width, tex_y + tile_height],
                });
                // C
                vertices.push(Vertex {
                    position: [x, y + 1., z],
                    tex_coords: [tex_x, tex_y],
                });

                // SECOND TRIANGLE
                // C-----D
                //   \   |
                //     \ |
                //       B

                // B
                vertices.push(Vertex {
                    position: [x + 1., y, z],
                    tex_coords: [tex_x + tile_width, tex_y + tile_height],
                });
                // C
                vertices.push(Vertex {
                    position: [x, y + 1., z],
                    tex_coords: [tex_x, tex_y],
                });
                // D
                vertices.push(Vertex {
                    position: [x + 1., y + 1., z],
                    tex_coords: [tex_x + tile_width, tex_y],
                });
            }
        }

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("map_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        TileVertices {
            buffer,
            vertices: vertices.len() as u32,
        }
    }
}
