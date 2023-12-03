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
pub struct Autotiles {
    data: AtomicCell<Data>,
    uniform: Option<wgpu::Buffer>,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    autotile_frames: [u32; 7],
    _array_padding: u32,
    ani_index: u32,
    autotile_region_width: u32,
    _end_padding: u64,
}

impl Autotiles {
    pub fn new(graphics_state: &GraphicsState, atlas: &super::Atlas) -> Self {
        let autotiles = Data {
            autotile_frames: atlas.autotile_frames,
            autotile_region_width: atlas.autotile_width,
            ani_index: 0,
            _array_padding: 0,
            _end_padding: 0,
        };

        let uniform = (!graphics_state.push_constants_supported()).then(|| {
            graphics_state.render_state.device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap autotile buffer"),
                    contents: bytemuck::cast_slice(&[autotiles]),
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                },
            )
        });

        Autotiles {
            data: AtomicCell::new(autotiles),
            uniform,
        }
    }

    pub fn inc_ani_index(&self, render_state: &luminol_egui_wgpu::RenderState) {
        let data = self.data.load();
        self.data.store(Data {
            ani_index: data.ani_index.wrapping_add(1),
            ..data
        });
        self.regen_buffer(render_state);
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
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
