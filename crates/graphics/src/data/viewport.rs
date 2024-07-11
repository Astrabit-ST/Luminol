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

use wgpu::util::DeviceExt;

use crate::{BindGroupLayoutBuilder, GraphicsState};

#[derive(Debug)]
pub struct Viewport {
    data: Data,
    uniform: wgpu::Buffer,
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(16))]
struct Data {
    viewport_size: glam::Vec2,
    viewport_translation: glam::Vec2,
    viewport_scale: glam::Vec2,
    _pad: [u32; 2],
}

impl Viewport {
    pub fn new(graphics_state: &GraphicsState, viewport_size: glam::Vec2) -> Self {
        let data = Data {
            viewport_size,
            viewport_translation: glam::Vec2::ZERO,
            viewport_scale: glam::Vec2::ONE,
            _pad: [0; 2],
        };
        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("viewport buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            },
        );

        Self { data, uniform }
    }

    pub fn set_size(&mut self, render_state: &luminol_egui_wgpu::RenderState, size: glam::Vec2) {
        self.data.viewport_size = size;
        self.regen_buffer(render_state);
    }

    pub fn set(
        &mut self,
        render_state: &luminol_egui_wgpu::RenderState,
        size: glam::Vec2,
        translation: glam::Vec2,
        scale: glam::Vec2,
    ) {
        self.data.viewport_size = size;
        self.data.viewport_translation = translation;
        self.data.viewport_scale = scale;
        self.regen_buffer(render_state);
    }

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.uniform
    }

    fn regen_buffer(&self, render_state: &luminol_egui_wgpu::RenderState) {
        render_state
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.data));
    }

    pub fn add_to_bind_group_layout(
        layout_builder: &mut BindGroupLayoutBuilder,
    ) -> &mut BindGroupLayoutBuilder {
        layout_builder.append(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            None,
        )
    }
}
