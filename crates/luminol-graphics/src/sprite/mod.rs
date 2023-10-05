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
use std::sync::Arc;

mod graphic;
mod shader;
mod vertices;

#[derive(Debug)]
pub struct Sprite {
    pub texture: Arc<crate::image_cache::WgpuTexture>,
    pub graphic: graphic::Graphic,
    pub vertices: vertices::Vertices,
    pub blend_mode: luminol_data::BlendMode,,
    pub use_push_constants: bool,
}

impl Sprite {
    pub fn new(
        render_state: &egui_wgpu::RenderState,
        quad: crate::quad::Quad,
        texture: Arc<crate::image_cache::WgpuTexture>,
        blend_mode: luminol_data::BlendMode,
        hue: i32,
        opacity: i32,
        use_push_constants: bool,
    ) -> Self {
        let vertices = vertices::Vertices::from_quads(render_state, &[quad], texture.size());
        let graphic = graphic::Graphic::new(hue, opacity, use_push_constants);

        Self {
            texture,
            graphic,
            vertices,
            blend_mode,
            use_push_constants,
        }
    }

    pub fn reupload_verts(
        &self,
        render_state: &egui_wgpu::RenderState,
        quads: &[crate::quad::Quad],
    ) {
        let vertices = crate::quad::Quad::into_vertices(quads, self.texture.size());
        render_state.queue.write_buffer(
            &self.vertices.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    pub fn draw<'rpass>(
        &'rpass self,
        render_state: &crate::GraphicsState,
        viewport: &crate::viewport::Viewport,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        render_pass.set_pipeline(&render_state.pipelines.sprites[&self.blend_mode]);

        if self.use_push_constants {
            render_pass.set_push_constants(wgpu::ShaderStages::VERTEX, 0, &viewport.as_bytes());
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                64,
                &self.graphic.as_bytes(),
            );
        }

        self.texture.bind(render_pass);
        self.graphic.bind(render_pass);
        self.vertices.draw(render_pass);
    }
}
