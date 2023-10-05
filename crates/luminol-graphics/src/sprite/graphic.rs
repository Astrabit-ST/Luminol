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
pub struct Graphic {
    data: AtomicCell<Data>,
    uniform: Option<GraphicUniform>,
}

#[derive(Debug)]
struct GraphicUniform {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    hue: f32,
    opacity: f32,
    opacity_multiplier: f32,
}

impl Graphic {
    pub fn new(hue: i32, opacity: i32, use_push_constants: bool) -> Self {
        let hue = (hue % 360) as f32 / 360.0;
        let opacity = opacity as f32 / 255.;
        let data = Data {
            hue,
            opacity,
            opacity_multiplier: 1.,
        };

        let uniform = if !use_push_constants {
            let render_state = &state!().render_state;
            let buffer =
                render_state
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("tilemap sprite graphic buffer"),
                        contents: bytemuck::cast_slice(&[data]),
                        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    });
            let bind_group = render_state
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("tilemap sprite graphic bind group"),
                    layout: &LAYOUT,
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }],
                });
            Some(GraphicUniform { buffer, bind_group })
        } else {
            None
        };

        Self {
            data: AtomicCell::new(data),
            uniform,
        }
    }

    pub fn hue(&self) -> i32 {
        (self.data.load().hue * 360.) as i32
    }

    pub fn set_hue(&self, hue: i32) {
        let hue = (hue % 360) as f32 / 360.0;
        let data = self.data.load();

        if data.hue != hue {
            self.data.store(Data { hue, ..data });
            self.regen_buffer();
        }
    }

    pub fn opacity(&self) -> i32 {
        (self.data.load().opacity * 255.) as i32
    }

    pub fn set_opacity(&self, opacity: i32) {
        let opacity = opacity as f32 / 255.0;
        let data = self.data.load();

        if data.opacity != opacity {
            self.data.store(Data { opacity, ..data });
            self.regen_buffer();
        }
    }

    pub fn opacity_multiplier(&self) -> f32 {
        self.data.load().opacity_multiplier
    }

    pub fn set_opacity_multiplier(&self, opacity_multiplier: f32) {
        let data = self.data.load();

        if data.opacity_multiplier != opacity_multiplier {
            self.data.store(Data {
                opacity_multiplier,
                ..data
            });
            self.regen_buffer();
        }
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
        bytemuck::cast(self.data.load())
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
            render_pass.set_bind_group(2, &uniform.bind_group, &[]);
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
            label: Some("tilemap sprite graphic bind group layout"),
        })
});
