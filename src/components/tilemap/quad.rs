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
use super::vertices::Vertex;
use crate::prelude::*;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileQuad {
    pub pos: egui::Rect,
    pub tex_coords: egui::Rect,
    pub z: f32,
}

macro_rules! quad_to_vert {
    ($this:ident, $quadrant:ident) => {{
        let position = $this.pos.$quadrant();
        let tex_coords = $this.tex_coords.$quadrant();
        Vertex {
            position: [position.x, position.y, $this.z],
            tex_coords: [tex_coords.x, tex_coords.y],
        }
    }};
}

impl TileQuad {
    pub fn new(pos: egui::Rect, tex_coords: egui::Rect, z: f32) -> Self {
        Self { pos, tex_coords, z }
    }

    fn norm_tex_coords(self, extents: wgpu::Extent3d) -> Self {
        let scale = egui::vec2(extents.width as f32, extents.height as f32);
        let min = self.tex_coords.min.to_vec2() / scale;
        let max = self.tex_coords.max.to_vec2() / scale;

        Self {
            tex_coords: egui::Rect::from_min_max(min.to_pos2(), max.to_pos2()),
            ..self
        }
    }

    fn into_vertices(self) -> [Vertex; 4] {
        let top_left = quad_to_vert!(self, left_top);
        let top_right = quad_to_vert!(self, right_top);
        let bottom_right = quad_to_vert!(self, right_bottom);
        let bottom_left = quad_to_vert!(self, left_bottom);
        [top_left, top_right, bottom_right, bottom_left]
    }

    pub fn into_buffer(
        this: &[Self],
        extents: wgpu::Extent3d,
    ) -> (wgpu::Buffer, wgpu::Buffer, u32) {
        let render_state = &state!().render_state;

        let mut indices: Vec<u32> = vec![];
        let mut vertices: Vec<Vertex> = vec![];

        for quad in this {
            let quad = quad.norm_tex_coords(extents);
            for vert in quad.into_vertices() {
                let top_left = {
                    vertices.push(vert);
                    vertices.len() as u32
                };
                let top_right = {
                    vertices.push(vert);
                    vertices.len() as u32
                };
                let bottom_right = {
                    vertices.push(vert);
                    vertices.len() as u32
                };
                let bottom_left = {
                    vertices.push(vert);
                    vertices.len() as u32
                };

                // Tiles are made like this:
                // TL------TR
                // |  \ /  |
                // |  / \  |
                // BL-----BR
                indices.push(top_left);
                indices.push(top_right);
                indices.push(bottom_left);

                indices.push(top_right);
                indices.push(bottom_left);
                indices.push(bottom_right);
            }
        }

        let index_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap index buffer"),
                    usage: wgpu::BufferUsages::INDEX,
                    contents: bytemuck::cast_slice(&indices),
                });
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap vertex buffer"),
                    usage: wgpu::BufferUsages::VERTEX,
                    contents: bytemuck::cast_slice(&vertices),
                });

        (index_buffer, vertex_buffer, indices.len() as u32)
    }
}
