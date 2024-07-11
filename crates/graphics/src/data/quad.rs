// Copyright (C) 2024 Melody Madeline Lyons
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
use super::vertex::Vertex;
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quad {
    pub pos: egui::Rect,
    pub tex_coords: egui::Rect,
}

impl Quad {
    pub const fn new(pos: egui::Rect, tex_coords: egui::Rect) -> Self {
        Self { pos, tex_coords }
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

    fn into_corners(self) -> [Vertex; 4] {
        let Self { pos, tex_coords } = self;
        let top_left = {
            let position = pos.left_top();
            let tex_coords = tex_coords.left_top();
            Vertex {
                position: glam::vec2(position.x, position.y),
                tex_coords: glam::vec2(tex_coords.x, tex_coords.y),
            }
        };
        let top_right = {
            let position = pos.right_top();
            let tex_coords = tex_coords.right_top();
            Vertex {
                position: glam::vec2(position.x, position.y),
                tex_coords: glam::vec2(tex_coords.x, tex_coords.y),
            }
        };
        let bottom_right = {
            let position = pos.right_bottom();
            let tex_coords = tex_coords.right_bottom();
            Vertex {
                position: glam::vec2(position.x, position.y),
                tex_coords: glam::vec2(tex_coords.x, tex_coords.y),
            }
        };
        let bottom_left = {
            let position = pos.left_bottom();
            let tex_coords = tex_coords.left_bottom();
            Vertex {
                position: glam::vec2(position.x, position.y),
                tex_coords: glam::vec2(tex_coords.x, tex_coords.y),
            }
        };
        [top_left, top_right, bottom_right, bottom_left]
    }

    pub fn into_vertices(this: &[Self], extents: wgpu::Extent3d) -> Vec<Vertex> {
        let mut vertices = Vec::with_capacity(this.len() * 6);

        // Quads are made like this:
        // TL------TR
        // |  \ /  |
        // |  / \  |
        // BL-----BR
        for quad in this {
            let quad = quad.norm_tex_coords(extents);
            let quad_verts = quad.into_corners();
            // Top left
            vertices.push(quad_verts[0]);
            // Top right
            vertices.push(quad_verts[1]);
            // Bottom left
            vertices.push(quad_verts[3]);

            // Top right
            vertices.push(quad_verts[1]);
            // Bottom right
            vertices.push(quad_verts[3]);
            // Bottom left
            vertices.push(quad_verts[2]);
        }

        vertices
    }

    pub fn into_buffer(
        render_state: &luminol_egui_wgpu::RenderState,
        this: &[Self],
        extents: wgpu::Extent3d,
    ) -> (wgpu::Buffer, usize) {
        let vertices = Self::into_vertices(this, extents);

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("quad vertex buffer"),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                contents: bytemuck::cast_slice(&vertices),
            });

        (buffer, vertices.len())
    }
}
