use eframe::wgpu::util::DeviceExt;

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
use super::Shader;
use crate::prelude::*;
use crossbeam::atomic::AtomicCell;

pub struct Uniform {
    data: AtomicCell<Data>,
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, PartialEq)]
struct Data {
    /// The tilemap pan.
    pan: egui::Vec2,
    /// The scale of the tilemap.
    scale: f32,
}

#[allow(unsafe_code)]
unsafe impl bytemuck::Pod for Data {}

impl Uniform {
    pub fn new() -> Self {
        let data = Data {
            pan: egui::Vec2::ZERO,
            scale: 100.,
        };
        let render_state = &state!().render_state;
        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("tilemap uniform buffer"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tilemap uniform bind group"),
                layout: Shader::uniform_layout(),
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });

        Uniform {
            data: AtomicCell::new(data),
            buffer,
            bind_group,
        }
    }

    pub fn scale(&self) -> f32 {
        self.data.load().scale
    }

    pub fn set_scale(&self, scale: f32) {
        self.data.store(Data {
            scale,
            ..self.data.load()
        });
        self.regen_buffer();
    }

    fn regen_buffer(&self) {
        let render_state = &state!().render_state;
        render_state
            .queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.data.load()]));
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_bind_group(1, &self.bind_group, &[]);
    }
}
