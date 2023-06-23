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
pub struct Autotiles {
    data: AtomicCell<Data>,

    buffer: wgpu::Buffer,

    bind_group: wgpu::BindGroup,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    ani_index: u32,
    autotile_region_width: u32,
    autotile_frames: [u32; super::AUTOTILE_AMOUNT as usize],
}

impl Autotiles {
    pub fn new(atlas: &super::Atlas) -> Self {
        let autotiles = Data {
            autotile_frames: atlas.autotile_frames,
            autotile_region_width: atlas.autotile_width,
            ani_index: 0,
        };

        let render_state = &state!().render_state;

        let autotile_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap autotile buffer"),
                    contents: bytemuck::cast_slice(&[autotiles]),
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::UNIFORM,
                });

        let bind_group = render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("tilemap autotiles bind group"),
                layout: &LAYOUT,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: autotile_buffer.as_entire_binding(),
                }],
            });

        Autotiles {
            data: AtomicCell::new(autotiles),
            buffer: autotile_buffer,
            bind_group,
        }
    }

    pub fn inc_ani_index(&self) {
        let data = self.data.load();
        self.data.store(Data {
            ani_index: data.ani_index.wrapping_add(1),
            ..data
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
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("tilemap autotiles bind group layout"),
        })
});
