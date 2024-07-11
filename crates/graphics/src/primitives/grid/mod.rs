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

use display::Display;
use instance::Instances;

pub mod display;
mod instance;
pub(crate) mod shader;

#[derive(Debug)]
pub struct Grid {
    pub instances: Instances,
    pub display: display::Display,
    pub transform: Transform,
    // in an Arc so we can use it in rendering
    pub bind_group: Arc<wgpu::BindGroup>,
}

impl Grid {
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: &Viewport,
        transform: Transform,
        map_width: u32,
        map_height: u32,
    ) -> Self {
        let instances = Instances::new(map_width, map_height);
        let display = Display::new(graphics_state, map_width, map_height);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder.append_buffer(viewport.as_buffer());
        bind_group_builder.append_buffer(transform.as_buffer());
        bind_group_builder.append_buffer(display.as_buffer());
        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("grid bind group"),
            &graphics_state.bind_group_layouts.grid,
        );

        Self {
            instances,
            display,
            transform,
            bind_group: Arc::new(bind_group),
        }
    }
}

pub struct Prepared {
    bind_group: Arc<wgpu::BindGroup>,
    instances: Instances,
    graphics_state: Arc<GraphicsState>,
}

impl Renderable for Grid {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &Arc<GraphicsState>) -> Self::Prepared {
        let bind_group = Arc::clone(&self.bind_group);
        let graphics_state = Arc::clone(graphics_state);
        let instances = self.instances;

        Prepared {
            bind_group,
            instances,
            graphics_state,
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap grid renderer");
        render_pass.set_pipeline(&self.graphics_state.pipelines.grid);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();

    Viewport::add_to_bind_group_layout(&mut builder);
    Transform::add_to_bind_group_layout(&mut builder);
    display::add_to_bind_group_layout(&mut builder);

    builder.build(&render_state.device, Some("grid bind group layout"))
}
