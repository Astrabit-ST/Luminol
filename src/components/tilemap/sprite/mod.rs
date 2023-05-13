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

mod graphic;
mod shader;
mod vertices;

use super::quad::Quad;
use super::vertex::Vertex;
use crate::prelude::*;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub enum BlendMode {
    Normal = 0,
    Add = 1,
    Subtract = 2,
}

impl TryFrom<i32> for BlendMode {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => BlendMode::Normal,
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            mode => return Err(format!("unexpected blend mode {mode}")),
        })
    }
}

#[derive(Debug)]
pub struct Sprite {
    pub texture: Arc<image_cache::WgpuTexture>,
    pub graphic: graphic::Graphic,
    pub vertices: vertices::Vertices,
    pub blend_mode: BlendMode,
}

impl Sprite {
    pub fn new(
        quads: &[Quad],
        texture: Arc<image_cache::WgpuTexture>,
        blend_mode: BlendMode,
        hue: i32,
        opacity: i32,
    ) -> Self {
        let vertices = vertices::Vertices::from_quads(quads, texture.size());
        let graphic = graphic::Graphic::new(hue, opacity);

        Self {
            texture,
            graphic,
            vertices,
            blend_mode,
        }
    }

    pub fn reupload_verts(&self, quads: &[Quad]) {
        let render_state = &state!().render_state;

        let (vertices, indices) = Quad::into_raw_verts(quads, self.texture.size());
        render_state.queue.write_buffer(
            &self.vertices.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
        render_state.queue.write_buffer(
            &self.vertices.index_buffer,
            0,
            bytemuck::cast_slice(&indices),
        );
        self.vertices.indices.store(indices.len() as u32);
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        shader::Shader::bind(self.blend_mode, render_pass);
        self.texture.bind(render_pass);
        self.graphic.bind(render_pass);
        self.vertices.draw(render_pass);
    }
}
