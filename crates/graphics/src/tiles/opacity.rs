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

#[derive(Debug)]
pub struct Opacity {
    data: AtomicCell<[f32; 4]>, // length has to be a multiple of 4
    uniform: Option<OpacityUniform>,
}

#[derive(Debug)]
struct OpacityUniform {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Opacity {
    pub fn new(graphics_state: &crate::GraphicsState, use_push_constants: bool) -> Self {
        let opacity = [1.; 4];

        let uniform =
            if !use_push_constants {
                let buffer = graphics_state.render_state.device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("tilemap opacity buffer"),
                        contents: bytemuck::cast_slice(&[opacity]),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    },
                );
                let bind_group = graphics_state.render_state.device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        label: Some("tilemap opacity bind group"),
                        layout: &graphics_state.bind_group_layouts.tile_layer_opacity,
                        entries: &[wgpu::BindGroupEntry {
                            binding: 0,
                            resource: buffer.as_entire_binding(),
                        }],
                    },
                );
                Some(OpacityUniform { buffer, bind_group })
            } else {
                None
            };

        Self {
            data: AtomicCell::new(opacity),
            uniform,
        }
    }

    pub fn opacity(&self, layer: usize) -> f32 {
        self.data.load()[layer]
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
            render_state.queue.write_buffer(
                &uniform.buffer,
                0,
                bytemuck::cast_slice(&[self.data.load()]),
            );
        }
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        if let Some(uniform) = &self.uniform {
            render_pass.set_bind_group(3, &uniform.bind_group, &[]);
        }
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
    render_state
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("tilemap opacity bind group layout"),
        })
}
