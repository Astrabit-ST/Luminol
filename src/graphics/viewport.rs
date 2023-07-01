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
use once_cell::sync::Lazy;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Viewport {
    data: AtomicCell<Data>,
    bind_group: wgpu::BindGroup,
    buffer: wgpu::Buffer,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq)]
struct Data {
    proj: cgmath::Matrix4<f32>,
}

// SAFETY:
//
//
#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Data {}
#[allow(unsafe_code)]
unsafe impl bytemuck::Zeroable for Data {}

impl Viewport {
    pub fn new() -> Self {
        let data = Data {
            proj: cgmath::ortho(0.0, 800.0, 600.0, 0.0, -1.0, 1.0),
        };
        let render_state = &state!().render_state;

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tilemap viewport buffer"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tilemap viewport uniform bind group"),
                layout: &LAYOUT,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            data: AtomicCell::new(data),
            bind_group,
            buffer,
        }
    }

    pub fn set_proj(&self, proj: cgmath::Matrix4<f32>) {
        let data = self.data.load();
        if data.proj != proj {
            self.data.store(Data { proj });
            self.regen_buffer();
        }
    }

    fn regen_buffer(&self) {
        let render_state = &state!().render_state;
        render_state
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data.load()]));
    }

    pub fn layout() -> &'static wgpu::BindGroupLayout {
        &LAYOUT
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
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
            label: Some("tilemap viewport bind group layout"),
        })
});
