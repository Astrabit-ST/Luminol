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
pub struct Autotiles {
    data: Data,
    uniform: wgpu::Buffer,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    autotile_frames: [u32; 7],
    _array_padding: u32,
    ani_index: u32,
    max_frame_count: u32,
    _end_padding: u64,
}

impl Autotiles {
    pub fn new(graphics_state: &GraphicsState, atlas: &super::Atlas) -> Self {
        let data = Data {
            autotile_frames: atlas.autotile_frames,
            max_frame_count: atlas.autotile_width / super::atlas::AUTOTILE_FRAME_WIDTH,
            ani_index: 0,
            _array_padding: 0,
            _end_padding: 0,
        };

        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("tilemap autotile buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            },
        );

        Autotiles { data, uniform }
    }

    pub fn inc_ani_index(&mut self, render_state: &luminol_egui_wgpu::RenderState) {
        self.data.ani_index = self.data.ani_index.wrapping_add(1);
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
