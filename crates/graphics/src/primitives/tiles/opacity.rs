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
pub struct Opacity {
    data: AtomicCell<[f32; 4]>, // length has to be a multiple of 4
    uniform: Option<wgpu::Buffer>,
}

impl Opacity {
    pub fn new(graphics_state: &GraphicsState) -> Self {
        let opacity = [1.; 4];

        let uniform = (!graphics_state.push_constants_supported()).then(|| {
            graphics_state.render_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap opacity buffer"),
                    contents: bytemuck::cast_slice(&[opacity]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                },
            )
        });

        Self {
            data: AtomicCell::new(opacity),
            uniform,
        }
    }

    pub fn opacity(&self, layer: usize) -> f32 {
        self.data.load()[layer]
    }

    pub fn as_buffer(&self) -> Option<&wgpu::Buffer> {
        self.uniform.as_ref()
    }

    pub fn set_opacity(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        layer: usize,
        opacity: f32,
    ) {
        let mut data = self.data.load();
        if data[layer] != opacity {
            data[layer] = opacity;
            self.data.store(data);
            self.regen_buffer(render_state);
        }
    }

    fn regen_buffer(&self, render_state: &luminol_egui_wgpu::RenderState) {
        if let Some(uniform) = &self.uniform {
            render_state
                .queue
                .write_buffer(uniform, 0, bytemuck::cast_slice(&[self.data.load()]));
        }
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
