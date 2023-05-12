use once_cell::sync::Lazy;

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
pub struct Hue {
    hue: AtomicCell<f32>,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl Hue {
    pub fn new(hue: i32) -> Self {
        let hue = (hue % 360) as f32 / 360.0;
        let render_state = &state!().render_state;

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tilemap event hue buffer"),
                contents: bytemuck::cast_slice(&[hue]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tilemap event hue bind group"),
                layout: &LAYOUT,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Self {
            bind_group,
            buffer,
            hue: AtomicCell::new(hue),
        }
    }

    pub fn hue(&self) -> i32 {
        (self.hue.load() * 360.) as i32
    }

    pub fn set_hue(&self, hue: i32) {
        let hue = (hue % 360) as f32 / 360.0;
        if self.hue.load() != hue {
            self.hue.store(hue);

            state!()
                .render_state
                .queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[hue]));
        }
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_bind_group(2, &self.bind_group, &[]);
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
            label: Some("tilemap event hue bind group layout"),
        })
});
