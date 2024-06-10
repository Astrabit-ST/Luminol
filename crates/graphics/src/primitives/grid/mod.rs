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

use crate::{BindGroupBuilder, BindGroupLayoutBuilder, GraphicsState, Viewport};

use display::Display;
use instance::Instances;
use vertex::Vertex;

pub mod display;
mod instance;
pub(crate) mod shader;
mod vertex;

#[derive(Debug)]
pub struct Grid {
    pub instances: Instances,
    pub display: display::Display,
    pub viewport: Arc<Viewport>,

    pub bind_group: wgpu::BindGroup,
}

impl Grid {
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: Arc<Viewport>,
        map_width: usize,
        map_height: usize,
    ) -> Self {
        let instances = Instances::new(&graphics_state.render_state, map_width, map_height);
        let display = Display::new(graphics_state);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder.append_buffer(viewport.as_buffer());
        bind_group_builder.append_buffer(display.as_buffer());
        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("grid bind group"),
            &graphics_state.bind_group_layouts.grid,
        );

        Self {
            instances,
            display,
            viewport,
            bind_group,
        }
    }

    pub fn draw<'rpass>(
        &'rpass self,
        graphics_state: &'rpass GraphicsState,
        info: &egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct VertexPushConstant {
            viewport: [u8; 64],
            display: [u8; 16],
        }

        render_pass.push_debug_group("tilemap grid renderer");
        render_pass.set_pipeline(&graphics_state.pipelines.grid);

        render_pass.set_bind_group(0, &self.bind_group, &[]);

        self.display
            .update_viewport_size(&graphics_state.render_state, info);

        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();

    Viewport::add_to_bind_group_layout(&mut builder);
    display::add_to_bind_group_layout(&mut builder);

    builder.build(&render_state.device, Some("grid bind group layout"))
}
