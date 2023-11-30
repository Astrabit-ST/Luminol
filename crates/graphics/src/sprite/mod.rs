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

use crate::{
    quad::Quad,
    viewport::{self, Viewport},
    BindGroupBuilder, BindGroupLayoutBuilder, GraphicsState, Texture,
};

pub(crate) mod graphic;
pub(crate) mod shader;
mod vertices;

#[derive(Debug)]
pub struct Sprite {
    pub texture: Arc<Texture>,
    pub graphic: graphic::Graphic,
    pub vertices: vertices::Vertices,
    pub blend_mode: luminol_data::BlendMode,
    pub viewport: Arc<Viewport>,

    pub bind_group: wgpu::BindGroup,
}

impl Sprite {
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: Arc<Viewport>,
        quad: Quad,
        texture: Arc<Texture>,
        blend_mode: luminol_data::BlendMode,
        hue: i32,
        opacity: i32,
    ) -> Self {
        let vertices =
            vertices::Vertices::from_quads(&graphics_state.render_state, &[quad], texture.size());
        let graphic = graphic::Graphic::new(graphics_state, hue, opacity);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder
            .append_texture_view(&texture.view)
            .append_sampler(&graphics_state.nearest_sampler);
        if !graphics_state.push_constants_supported() {
            bind_group_builder
                .append_buffer(viewport.as_buffer().unwrap())
                .append_buffer(graphic.as_buffer().unwrap());
        }
        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("sprite bind group"),
            &graphics_state.bind_group_layouts.sprite,
        );

        Self {
            texture,
            graphic,
            vertices,
            blend_mode,
            viewport,

            bind_group,
        }
    }

    pub fn reupload_verts(&self, render_state: &egui_wgpu::RenderState, quads: &[Quad]) {
        let vertices = Quad::into_vertices(quads, self.texture.size());
        render_state.queue.write_buffer(
            &self.vertices.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );
    }

    pub fn draw<'rpass>(
        &'rpass self,
        graphics_state: &'rpass GraphicsState,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        render_pass.push_debug_group("sprite render");
        render_pass.set_pipeline(&graphics_state.pipelines.sprites[&self.blend_mode]);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        if graphics_state.push_constants_supported() {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                &self.viewport.as_bytes(),
            );
            render_pass.set_push_constants(
                wgpu::ShaderStages::FRAGMENT,
                64,
                &self.graphic.as_bytes(),
            );
        }

        self.vertices.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(render_state: &egui_wgpu::RenderState) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();
    builder
        .append(
            wgpu::ShaderStages::FRAGMENT,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            None,
        )
        .append(
            wgpu::ShaderStages::FRAGMENT,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            None,
        );

    if !crate::push_constants_supported(render_state) {
        viewport::add_to_bind_group_layout(&mut builder);
        graphic::add_to_bind_group_layout(&mut builder);
    }

    builder.build(&render_state.device, Some("sprite bind group layout"))
}
