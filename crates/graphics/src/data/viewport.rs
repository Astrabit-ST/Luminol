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

use crossbeam::atomic::AtomicCell;
use wgpu::util::DeviceExt;

use crate::{BindGroupLayoutBuilder, GraphicsState};

#[derive(Debug)]
pub struct Viewport {
    data: AtomicCell<glam::Mat4>,
    uniform: Option<wgpu::Buffer>,
}

impl Viewport {
    pub fn new(graphics_state: &GraphicsState, width: f32, height: f32) -> Self {
        Self::new_proj(
            graphics_state,
            glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0),
        )
    }

    pub fn new_proj(graphics_state: &GraphicsState, proj: glam::Mat4) -> Self {
        let uniform = (!graphics_state.push_constants_supported()).then(|| {
            graphics_state.render_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap viewport buffer"),
                    contents: bytemuck::cast_slice(&[proj]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                },
            )
        });

        Self {
            data: AtomicCell::new(proj),
            uniform,
        }
    }

    pub fn set_width_height(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        width: f32,
        height: f32,
    ) {
        self.set_proj(
            render_state,
            glam::Mat4::orthographic_rh(0.0, width, height, 0.0, -1.0, 1.0),
        )
    }

    pub fn set_proj(&self, render_state: &luminol_egui_wgpu::RenderState, proj: glam::Mat4) {
        let data = self.data.load();
        if data != proj {
            self.data.store(proj);
            self.regen_buffer(render_state);
        }
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<glam::Mat4>()] {
        bytemuck::cast(self.data.load())
    }

    pub fn as_buffer(&self) -> Option<&wgpu::Buffer> {
        self.uniform.as_ref()
    }

    fn regen_buffer(&self, render_state: &luminol_egui_wgpu::RenderState) {
        if let Some(uniform) = &self.uniform {
            render_state
                .queue
                .write_buffer(uniform, 0, bytemuck::cast_slice(&[self.data.load()]));
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
}
