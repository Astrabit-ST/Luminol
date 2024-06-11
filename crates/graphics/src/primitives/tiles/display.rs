// Copyright (C) 2024 Lily Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use parking_lot::RwLock;
use wgpu::util::DeviceExt;

use crate::{BindGroupLayoutBuilder, GraphicsState};

#[derive(Debug)]
pub struct Display {
    data: RwLock<LayerData>,
    uniform: wgpu::Buffer,
}

#[derive(Debug)]
struct LayerData {
    data: Vec<u8>,
    min_alignment_size: u32,
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Data {
    opacity: f32,
    _pad: [u8; 4],
    map_size: [u32; 2],
}

impl Data {
    fn aligned_size_of(min_alignment_size: u32) -> usize {
        wgpu::util::align_to(std::mem::size_of::<Self>(), min_alignment_size as usize)
    }
}

impl LayerData {
    fn range_of_layer(&self, layer: usize) -> std::ops::Range<usize> {
        let data_size = Data::aligned_size_of(self.min_alignment_size);
        let start = layer * data_size;
        let end = start + std::mem::size_of::<Data>();
        start..end
    }

    fn bytes_of_layer(&self, layer: usize) -> &[u8] {
        let range = self.range_of_layer(layer);
        &self.data[range]
    }

    fn bytes_of_layer_mut(&mut self, layer: usize) -> &mut [u8] {
        let range = self.range_of_layer(layer);
        &mut self.data[range]
    }

    fn read_data_at(&self, layer: usize) -> &Data {
        bytemuck::from_bytes(self.bytes_of_layer(layer))
    }

    fn read_data_at_mut(&mut self, layer: usize) -> &mut Data {
        bytemuck::from_bytes_mut(self.bytes_of_layer_mut(layer))
    }
}

impl Display {
    pub fn new(
        graphics_state: &GraphicsState,
        map_width: u32,
        map_height: u32,
        layers: usize,
    ) -> Self {
        let limits = graphics_state.render_state.device.limits();
        let min_alignment_size = limits.min_uniform_buffer_offset_alignment;

        let data_size = Data::aligned_size_of(min_alignment_size);
        let mut layer_data = LayerData {
            data: vec![0; data_size * layers],
            min_alignment_size,
        };

        for layer in 0..layers {
            *layer_data.read_data_at_mut(layer) = Data {
                opacity: 1.0,
                _pad: [0; 4],
                map_size: [map_width, map_height],
            };
        }

        let uniform = graphics_state.render_state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("tilemap display buffer"),
                contents: bytemuck::cast_slice(&layer_data.data),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            },
        );

        Self {
            data: RwLock::new(layer_data),
            uniform,
        }
    }

    pub fn as_buffer(&self) -> &wgpu::Buffer {
        &self.uniform
    }

    pub fn bytes_of_layer(&self, layer: usize) -> impl std::ops::Deref<Target = [u8]> + '_ {
        parking_lot::RwLockReadGuard::map(self.data.read(), |d| d.bytes_of_layer(layer))
    }

    pub fn opacity(&self, layer: usize) -> f32 {
        self.data.read().read_data_at(layer).opacity
    }

    pub fn set_opacity(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        opacity: f32,
        layer: usize,
    ) {
        let mut data = self.data.write();
        let layer_data = data.read_data_at_mut(layer);
        if layer_data.opacity != opacity {
            layer_data.opacity = opacity;
            self.regen_buffer(render_state, &data.data);
        }
    }

    pub fn aligned_layer_size(&self) -> usize {
        let data = self.data.read();
        Data::aligned_size_of(data.min_alignment_size)
    }

    pub fn layer_offset(&self, layer: usize) -> u32 {
        self.data.read().range_of_layer(layer).start as u32
    }

    fn regen_buffer(&self, render_state: &luminol_egui_wgpu::RenderState, data: &[u8]) {
        render_state.queue.write_buffer(self.as_buffer(), 0, data);
    }
}

pub fn add_to_bind_group_layout(
    layout_builder: &mut BindGroupLayoutBuilder,
) -> &mut BindGroupLayoutBuilder {
    layout_builder.append(
        wgpu::ShaderStages::VERTEX_FRAGMENT,
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: true,
            min_binding_size: None,
        },
        None,
    )
}
