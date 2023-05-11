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
    autotiles::AUTOTILES, quad::TileQuad, Atlas, AUTOTILE_AMOUNT, MAX_SIZE, TILESET_WIDTH,
    TOTAL_AUTOTILE_HEIGHT, UNDER_HEIGHT,
};
use crate::prelude::*;

use wgpu::util::DeviceExt;

pub struct TileVertices {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
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

        let mut quads = Vec::with_capacity(map.data.len());
        for (index, tile) in map.data.iter().copied().enumerate() {
            // We reset the x every xsize elements.
            let map_x = index % map.data.xsize();
            // We reset the y every ysize elements, but only increment it every xsize elements.
            let map_y = (index / map.data.xsize()) % map.data.ysize();
            // We change the z every xsize * ysize elements.
            let map_z = index / (map.data.xsize() * map.data.ysize());

            match tile {
                0..=47 => {
                    continue;
                }
                48..=384 => {}
                tile if atlas.tileset_height + TOTAL_AUTOTILE_HEIGHT <= MAX_SIZE => {}
                tile => {}
            }
        }
        let (index_buffer, vertex_buffer, indices) =
            TileQuad::into_buffer(&quads, atlas.atlas_texture.size());

        TileVertices {
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
