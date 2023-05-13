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

use super::quad::Quad;
use super::vertex::Vertex;
use super::BlendMode;
use crate::prelude::*;

mod graphic;
mod shader;

#[derive(Debug)]
pub struct Plane {
    texture: Arc<image_cache::WgpuTexture>,
    pub blend_mode: BlendMode,
    vertices: Vertices,
    graphic: graphic::Graphic,
}

#[derive(Debug)]
pub struct Vertices {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices: u32,
}

impl Vertices {
    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw_indexed(0..self.indices, 0, 0..1);
    }
}

impl Plane {
    pub fn new(
        texture: Arc<image_cache::WgpuTexture>,
        hue: i32,
        zoom: i32,
        blend_mode: BlendMode,
        opacity: i32,
    ) -> Self {
        let tex_coords = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(texture.width() as f32, texture.height() as f32),
        );

        let (index_buffer, vertex_buffer, indices) = Quad::into_buffer(
            &[Quad::new(
                egui::Rect::from_min_size(egui::pos2(0.0, 0.0), tex_coords.size()),
                tex_coords,
                0.0,
            )],
            texture.size(),
        );
        let vertices = Vertices {
            vertex_buffer,
            index_buffer,
            indices,
        };

        let graphic = graphic::Graphic::new(hue, opacity);

        Self {
            texture,
            blend_mode,
            vertices,
            graphic,
        }
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        shader::Shader::bind(self.blend_mode, render_pass);
        self.vertices.draw(render_pass);
    }
}
