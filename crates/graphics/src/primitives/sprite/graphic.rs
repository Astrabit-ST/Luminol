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
pub struct Graphic {
    data: Data,
    uniform: wgpu::Buffer,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    hue: f32,
    opacity: f32,
    opacity_multiplier: f32,
    _padding: u32,
}

impl Graphic {
    pub fn new(graphics_state: &GraphicsState, hue: i32, opacity: i32) -> Self {
        let hue = (hue % 360) as f32 / 360.0;
        let opacity = opacity as f32 / 255.;
        let data = Data {
            hue,
            opacity,
            opacity_multiplier: 1.,
            _padding: 0,
        };

        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("tilemap sprite graphic buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        Self { data, uniform }
    }

    pub fn hue(&self) -> i32 {
        (self.data.hue * 360.) as i32
    }

    pub fn set_hue(&mut self, render_state: &luminol_egui_wgpu::RenderState, hue: i32) {
        let hue = (hue % 360) as f32 / 360.0;

        if self.data.hue != hue {
            self.data.hue = hue;
            self.regen_buffer(render_state);
        }
    }

    pub fn opacity(&self) -> i32 {
        (self.data.opacity * 255.) as i32
    }

    pub fn set_opacity(&mut self, render_state: &luminol_egui_wgpu::RenderState, opacity: i32) {
        let opacity = opacity as f32 / 255.0;

        if self.data.opacity != opacity {
            self.data.opacity = opacity;
            self.regen_buffer(render_state);
        }
    }

    pub fn opacity_multiplier(&self) -> f32 {
        self.data.opacity_multiplier
    }

    pub fn set_opacity_multiplier(
        &mut self,
        render_state: &luminol_egui_wgpu::RenderState,
        opacity_multiplier: f32,
    ) {
        if self.data.opacity_multiplier != opacity_multiplier {
            self.data.opacity_multiplier = opacity_multiplier;
            self.regen_buffer(render_state);
        }
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
