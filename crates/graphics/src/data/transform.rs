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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use wgpu::util::DeviceExt;

use crate::{BindGroupLayoutBuilder, GraphicsState};

#[derive(Debug)]
pub struct Transform {
    data: Data,
    uniform: wgpu::Buffer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Data {
    position: glam::Vec2,
    scale: glam::Vec2,
}

impl Transform {
    pub fn new(graphics_state: &GraphicsState, position: glam::Vec2, scale: glam::Vec2) -> Self {
        let data = Data { position, scale };

        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("transform buffer"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            },
        );

        Self { data, uniform }
    }

    pub fn new_position(graphics_state: &GraphicsState, position: glam::Vec2) -> Self {
        Self::new(graphics_state, position, glam::Vec2::ONE)
    }

    pub fn unit(graphics_state: &GraphicsState) -> Self {
        Self::new(graphics_state, glam::Vec2::ZERO, glam::Vec2::ONE)
    }

    pub fn set_position(
        &mut self,
        render_state: &luminol_egui_wgpu::RenderState,
        position: glam::Vec2,
    ) {
        self.data.position = position;
        self.regen_buffer(render_state);
    }

    pub fn set_scale(&mut self, render_state: &luminol_egui_wgpu::RenderState, scale: glam::Vec2) {
        self.data.scale = scale;
        self.regen_buffer(render_state);
    }

    fn regen_buffer(&mut self, render_state: &luminol_egui_wgpu::RenderState) {
        render_state
            .queue
            .write_buffer(&self.uniform, 0, bytemuck::bytes_of(&self.data));
    }

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.uniform
    }

    pub fn add_to_bind_group_layout(layout_builder: &mut BindGroupLayoutBuilder) {
        layout_builder.append(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            None,
        );
    }
}
