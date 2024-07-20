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
    BindGroupBuilder, BindGroupLayoutBuilder, Drawable, GraphicsState, Renderable, Transform,
    Viewport,
};

pub use atlas::*;

use display::Display;
use instance::Instances;

mod atlas;
pub(crate) mod display;
mod instance;
pub(crate) mod shader;

pub struct Cells {
    pub display: Display,
    pub atlas: Atlas,
    pub transform: Transform,

    instances: Arc<Instances>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Cells {
    pub fn new(
        graphics_state: &GraphicsState,
        cells: &luminol_data::Table2,
        // in order of use in bind group
        atlas: Atlas,
        viewport: &Viewport,
        transform: Transform,
    ) -> Self {
        let instances = Instances::new(&graphics_state.render_state, cells);
        let display = Display::new(graphics_state, cells.xsize() as u32);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder
            .append_texture_view(&atlas.texture().view)
            .append_sampler(&graphics_state.nearest_sampler)
            .append_buffer(viewport.as_buffer())
            .append_buffer(transform.as_buffer())
            .append_buffer(display.as_buffer());

        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("cells bind group"),
            &graphics_state.bind_group_layouts.cells,
        );

        Self {
            display,
            atlas,
            transform,

            instances: Arc::new(instances),
            bind_group: Arc::new(bind_group),
        }
    }

    pub fn set_cell(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        cell_id: i16,
        position: (usize, usize),
    ) {
        self.instances.set_cell(render_state, cell_id, position)
    }
}

pub struct Prepared {
    bind_group: Arc<wgpu::BindGroup>,
    instances: Arc<Instances>,
    graphics_state: Arc<GraphicsState>,
}

impl Renderable for Cells {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &Arc<GraphicsState>) -> Self::Prepared {
        let bind_group = Arc::clone(&self.bind_group);
        let graphics_state = Arc::clone(graphics_state);
        let instances = Arc::clone(&self.instances);

        Prepared {
            bind_group,
            instances,
            graphics_state,
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("cells renderer");
        render_pass.set_pipeline(&self.graphics_state.pipelines.cells);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();
    builder
        .append(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
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
    display::add_to_bind_group_layout(&mut builder);

    builder.build(&render_state.device, Some("cells bind group layout"))
}
