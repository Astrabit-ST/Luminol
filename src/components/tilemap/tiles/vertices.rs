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
            let z = map_z as f32 / map.data.zsize() as f32;
            // let map_z = map.data.zsize() - map_z;
            atlas.calc_quads(tile, map_x, map_y, z, &mut quads);
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
