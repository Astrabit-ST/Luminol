// Copyright (C) 2024 Melody Madeline Lyons
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

use itertools::Itertools;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Instances {
    instance_buffer: wgpu::Buffer,

    cells_width: usize,
    cells_height: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    cell_id: u32,
}

impl Instances {
    pub fn new(
        render_state: &luminol_egui_wgpu::RenderState,
        cells_data: &luminol_data::Table2,
    ) -> Self {
        let instances = Self::calculate_instances(cells_data);
        let instance_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("cells instance buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            instance_buffer,

            cells_width: cells_data.xsize(),
            cells_height: cells_data.ysize(),
        }
    }

    pub fn set_cell(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        cell_id: i16,
        position: (usize, usize),
    ) {
        let offset = position.0 + (position.1 * self.cells_width);
        let offset = offset * std::mem::size_of::<Instance>();
        render_state.queue.write_buffer(
            &self.instance_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&Instance {
                cell_id: cell_id as u32,
            }),
        )
    }

    fn calculate_instances(cells_data: &luminol_data::Table2) -> Vec<Instance> {
        cells_data
            .iter()
            .copied()
            .map(|cell_id| Instance {
                cell_id: cell_id as u32,
            })
            .collect_vec()
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        let count = (self.cells_width * self.cells_height) as u32;

        let start = 0 as wgpu::BufferAddress;
        let end = (count as usize * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        render_pass.set_vertex_buffer(0, self.instance_buffer.slice(start..end));

        render_pass.draw(0..6, 0..count);
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ARRAY: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![0 => Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARRAY,
        }
    }
}
