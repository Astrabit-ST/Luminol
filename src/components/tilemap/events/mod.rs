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
use crate::prelude::*;

mod shader;
mod uniform;

#[derive(Debug)]
pub struct Event {
    texture: Arc<image_cache::WgpuTexture>,
    blend_mode: BlendMode,
    vertices: Vertices,
    pub uniform: uniform::Uniform,
}

#[derive(Debug)]
enum BlendMode {
    Normal,
    Add,
    Subtract,
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

impl Event {
    pub fn new(event: &rpg::Event, atlas: &Arc<image_cache::WgpuTexture>) -> Result<Self, String> {
        let Some(page) = event.pages.first() else {
            return Err("event does not have first page".to_string())
        };

        let texture = if page.graphic.tile_id.is_positive() {
            atlas.clone()
        } else {
            state!()
                .image_cache
                .load_wgpu_image("Graphics/Characters", &page.graphic.character_name)?
        };

        let tex_coords = if page.graphic.tile_id.is_positive() {
            // FIXME: Calc tile coordinates
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(32., 32.))
        } else {
            let cw = texture.width() as f32 / 4.;
            let ch = texture.height() as f32 / 4.;
            egui::Rect::from_min_size(
                egui::pos2(
                    page.graphic.pattern as f32 * cw,
                    (page.graphic.direction as f32 - 2.) / 2. * ch,
                ),
                egui::vec2(texture.width() as f32 / 4., texture.height() as f32 / 4.),
            )
        };

        let cw = texture.width() as f32 / 4.;
        let ch = texture.height() as f32 / 4.;
        let (index_buffer, vertex_buffer, indices) = Quad::into_buffer(
            &[Quad::new(
                egui::Rect::from_min_size(
                    egui::pos2(
                        (event.x as f32 * 32.) + (16. - (cw / 2.)),
                        (event.y as f32 * 32.) + (32. - ch),
                    ),
                    tex_coords.size(),
                ),
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

        let uniform = uniform::Uniform::new();

        let blend_mode = match page.graphic.blend_type {
            0 => BlendMode::Normal,
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            mode => return Err(format!("unexpected blend mode {mode}")),
        };

        Ok(Self {
            texture,
            blend_mode,
            vertices,
            uniform,
        })
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap event renderer");
        shader::Shader::bind(render_pass);
        self.texture.bind(render_pass);
        self.uniform.bind(render_pass);
        self.vertices.draw(render_pass);
        render_pass.pop_debug_group();
    }
}
