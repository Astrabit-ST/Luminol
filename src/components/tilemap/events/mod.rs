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

mod graphic;
mod shader;

#[derive(Debug)]
pub struct Events {
    events: Vec<Event>,
}

impl Events {
    pub fn new(map: &rpg::Map, atlas: &Arc<image_cache::WgpuTexture>) -> Result<Self, String> {
        let mut events: Vec<_> = map
            .events
            .iter()
            .filter(|(_, e)| {
                e.pages.first().is_some_and(|p| {
                    !p.graphic.character_name.is_empty() || p.graphic.tile_id.is_positive()
                })
            })
            .map(|(_, event)| Event::new(event, atlas))
            .try_collect()?;
        events.sort_unstable_by(|e1, e2| e1.blend_mode.cmp(&e2.blend_mode));

        Ok(Self { events })
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap event renderer");
        for event in self.events.iter() {
            event.draw(render_pass);
        }
        render_pass.pop_debug_group();
    }
}

#[derive(Debug)]
struct Event {
    texture: Arc<image_cache::WgpuTexture>,
    pub blend_mode: BlendMode,
    vertices: Vertices,
    graphic: graphic::Graphic,
}

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub enum BlendMode {
    Normal = 0,
    Add = 1,
    Subtract = 2,
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

        let blend_mode = match page.graphic.blend_type {
            0 => BlendMode::Normal,
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            mode => return Err(format!("unexpected blend mode {mode}")),
        };
        let graphic = graphic::Graphic::new(page.graphic.character_hue, page.graphic.opacity);

        Ok(Self {
            texture,
            blend_mode,
            vertices,
            graphic,
        })
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        shader::Shader::bind(self.blend_mode, render_pass);
        self.texture.bind(render_pass);
        self.graphic.bind(render_pass);
        self.vertices.draw(render_pass);
    }
}
