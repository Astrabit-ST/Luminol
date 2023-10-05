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
use crate::prelude::*;

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
    pub fn new(use_push_constants: bool) -> Self {
        let opacity = [1.; 4];

        let uniform = if !use_push_constants {
            let render_state = &state!().render_state;
            let buffer =
                render_state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tilemap opacity buffer"),
                        contents: bytemuck::cast_slice(&[opacity]),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    });
            let bind_group = render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tilemap opacity bind group"),
                    layout: &LAYOUT,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
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

    pub fn set_opacity(&self, layer: usize, opacity: f32) {
        let mut data = self.data.load();
        if data[layer] != opacity {
            data[layer] = opacity;
            self.data.store(data);
            self.regen_buffer();
        }
    }

    fn regen_buffer(&self) {
        if let Some(uniform) = &self.uniform {
            let render_state = &state!().render_state;
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

    pub fn layout() -> &'static wgpu::BindGroupLayout {
        &LAYOUT
    }
}

static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    let render_state = &state!().render_state;

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
});
