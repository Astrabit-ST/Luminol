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

use super::Vertex;
use itertools::Itertools;
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Instances {
    instance_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,

    map_width: usize,
    map_height: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    position: [f32; 2],
}

impl Instances {
    pub fn new(
        render_state: &luminol_egui_wgpu::RenderState,
        map_width: usize,
        map_height: usize,
    ) -> Self {
        let instances = Self::calculate_instances(map_width, map_height);
        let instance_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap grid instance buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let vertices = Self::calculate_vertices();
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap grid vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            instance_buffer,
            vertex_buffer,

            map_width,
            map_height,
        }
    }

    fn calculate_instances(map_width: usize, map_height: usize) -> Vec<Instance> {
        (0..map_height)
            .cartesian_product(0..map_width)
            .map(|(map_y, map_x)| Instance {
                position: [map_x as f32, map_y as f32],
            })
            .collect_vec()
    }

    fn calculate_vertices() -> [Vertex; 6] {
        [
            Vertex {
                position: glam::vec2(0., 0.), // Provoking vertex
            },
            Vertex {
                position: glam::vec2(1., 0.),
            },
            Vertex {
                position: glam::vec2(0., 1.),
            },
            Vertex {
                position: glam::vec2(1., 1.), // Provoking vertex
            },
            Vertex {
                position: glam::vec2(0., 1.),
            },
            Vertex {
                position: glam::vec2(1., 0.),
            },
        ]
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        // Calculate the start and end index of the buffer, as well as the amount of instances.
        let start_index = 0;
        let end_index = self.map_width * self.map_height;
        let count = (end_index - start_index) as u32;

        // Convert the indexes into actual offsets.
        let start = (start_index * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;
        let end = (end_index * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(start..end));

        render_pass.draw(0..6, 0..count);
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ARRAY: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![1 => Float32x2];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARRAY,
        }
    }
}
