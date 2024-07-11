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
use std::sync::Arc;

use crate::{
    BindGroupBuilder, BindGroupLayoutBuilder, Drawable, GraphicsState, Quad, Renderable, Texture,
    Transform, Viewport,
};

pub(crate) mod graphic;
pub(crate) mod shader;
mod vertices;

pub struct Sprite {
    pub graphic: graphic::Graphic,
    pub transform: Transform,
    pub blend_mode: luminol_data::BlendMode,

    // stored in an Arc so we can use it in rendering
    vertices: Arc<vertices::Vertices>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Sprite {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graphics_state: &GraphicsState,
        quad: Quad,
        hue: i32,
        opacity: i32,
        blend_mode: luminol_data::BlendMode,
        // arranged in order of use in bind group
        texture: &Texture,
        viewport: &Viewport,
        transform: Transform,
    ) -> Self {
        let vertices =
            vertices::Vertices::from_quads(&graphics_state.render_state, &[quad], texture.size());
        let graphic = graphic::Graphic::new(graphics_state, hue, opacity);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder
            .append_texture_view(&texture.view)
            .append_sampler(&graphics_state.nearest_sampler)
            .append_buffer(viewport.as_buffer())
            .append_buffer(transform.as_buffer())
            .append_buffer(graphic.as_buffer());

        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("sprite bind group"),
            &graphics_state.bind_group_layouts.sprite,
        );

        Self {
            graphic,
            blend_mode,
            transform,

            vertices: Arc::new(vertices),
            bind_group: Arc::new(bind_group),
        }
    }

    // like basic, but with a hue
    pub fn basic_hue(
        graphics_state: &GraphicsState,
        hue: i32,
        texture: &Texture,
        viewport: &Viewport,
    ) -> Self {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, texture.size_vec2());
        let quad = Quad::new(rect, rect);
        Self::basic_hue_quad(graphics_state, hue, quad, texture, viewport)
    }

    pub fn basic_hue_quad(
        graphics_state: &GraphicsState,
        hue: i32,
        quad: Quad,
        texture: &Texture,
        viewport: &Viewport,
    ) -> Self {
        Self::new(
            graphics_state,
            quad,
            hue,
            255,
            luminol_data::BlendMode::Normal,
            texture,
            viewport,
            Transform::unit(graphics_state),
        )
    }

    // takes the full size of a texture, has no hue, opacity, or blend mode, and uses the identity transform
    pub fn basic(graphics_state: &GraphicsState, texture: &Texture, viewport: &Viewport) -> Self {
        Self::basic_hue(graphics_state, 0, texture, viewport)
    }
}

pub struct Prepared {
    bind_group: Arc<wgpu::BindGroup>,
    vertices: Arc<vertices::Vertices>,
    graphics_state: Arc<GraphicsState>,
    blend_mode: luminol_data::BlendMode,
}

impl Renderable for Sprite {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &Arc<GraphicsState>) -> Self::Prepared {
        let bind_group = Arc::clone(&self.bind_group);
        let graphics_state = Arc::clone(graphics_state);
        let vertices = Arc::clone(&self.vertices);

        Prepared {
            bind_group,
            vertices,
            graphics_state,
            blend_mode: self.blend_mode,
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("sprite render");
        render_pass.set_pipeline(&self.graphics_state.pipelines.sprites[&self.blend_mode]);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        self.vertices.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
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

    Viewport::add_to_bind_group_layout(&mut builder);
    Transform::add_to_bind_group_layout(&mut builder);
    graphic::add_to_bind_group_layout(&mut builder);

    builder.build(&render_state.device, Some("sprite bind group layout"))
}
