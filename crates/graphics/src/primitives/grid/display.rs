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
pub struct Display {
    data: Data,
    uniform: wgpu::Buffer,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    pixels_per_point: f32,
    inner_thickness_in_points: f32,
    map_size: [u32; 2],
}

impl Display {
    pub fn new(graphics_state: &GraphicsState, map_width: u32, map_height: u32) -> Self {
        let data = Data {
            pixels_per_point: 1.,
            inner_thickness_in_points: 1.,
            map_size: [map_width, map_height],
        };

        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("grid display buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            },
        );

        Display { data, uniform }
    }

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.uniform
    }

    pub fn set_inner_thickness(
        &mut self,
        render_state: &luminol_egui_wgpu::RenderState,
        inner_thickness_in_points: f32,
    ) {
        if self.data.inner_thickness_in_points != inner_thickness_in_points {
            self.data.inner_thickness_in_points = inner_thickness_in_points;
            self.regen_buffer(render_state);
        }
    }

    pub fn set_pixels_per_point(
        &mut self,
        render_state: &luminol_egui_wgpu::RenderState,
        pixels_per_point: f32,
    ) {
        if self.data.pixels_per_point != pixels_per_point {
            self.data.pixels_per_point = pixels_per_point;
            self.regen_buffer(render_state);
        }
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
