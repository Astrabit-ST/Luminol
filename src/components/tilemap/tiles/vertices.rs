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

use super::{Atlas, Quad};
use crate::prelude::*;

#[derive(Debug)]
pub struct Vertices {
    buffers: Vec<Buffer>,
}

#[derive(Debug)]
struct Buffer {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    indices: u32,
}

impl Vertices {
    pub fn new(map: &rpg::Map, atlas: &Atlas) -> Self {
        let mut buffers = Vec::with_capacity(map.data.zsize());

        for layer in map.data.iter_layers() {
            let mut quads = Vec::with_capacity(map.width * map.height);
            for (index, tile) in layer.iter().copied().enumerate() {
                // We reset the x every xsize elements.
                let map_x = index % map.data.xsize();
                // We reset the y every ysize elements, but only increment it every xsize elements.
                let map_y = (index / map.data.xsize()) % map.data.ysize();
                atlas.calc_quads(tile, map_x, map_y, &mut quads);
            }

            let (index_buffer, vertex_buffer, indices) =
                Quad::into_buffer(&quads, atlas.atlas_texture.size());
            buffers.push(Buffer {
                vertex_buffer,
                index_buffer,
                indices,
            });
        }

        Vertices { buffers }
    }

    pub fn draw<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        enabled_layers: &[bool],
    ) {
        for (index, buffer) in self.buffers.iter().enumerate() {
            if enabled_layers.get(index).copied().unwrap_or(true) {
                render_pass
                    .set_index_buffer(buffer.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_vertex_buffer(0, buffer.vertex_buffer.slice(..));
                render_pass.draw_indexed(0..buffer.indices, 0, 0..1);
            }
        }
    }
}
