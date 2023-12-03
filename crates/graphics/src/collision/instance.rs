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
struct Instance {
    position: [f32; 3],
    passage: u32,
}

impl Instances {
    pub fn new(
        render_state: &luminol_egui_wgpu::RenderState,
        passages: &luminol_data::Table2,
    ) -> Self {
        let instances = Self::calculate_instances(passages);
        let instance_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap collision instance buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        let vertices = Self::calculate_vertices();
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap collision vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            instance_buffer,
            vertex_buffer,

            map_width: passages.xsize(),
            map_height: passages.ysize(),
        }
    }

    pub fn set_passage(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        passage: i16,
        position: (usize, usize),
    ) {
        let offset = position.0 + (position.1 * self.map_width);
        let offset = offset * std::mem::size_of::<Instance>();
        render_state.queue.write_buffer(
            &self.instance_buffer,
            offset as wgpu::BufferAddress,
            bytemuck::bytes_of(&Instance {
                position: [position.0 as f32, position.1 as f32, 0.0],
                passage: passage as u32,
            }),
        )
    }

    fn calculate_instances(passages: &luminol_data::Table2) -> Vec<Instance> {
        passages
            .iter()
            .copied()
            .enumerate()
            .map(|(index, passage)| {
                // We reset the x every xsize elements.
                let map_x = index % passages.xsize();
                // We reset the y every ysize elements, but only increment it every xsize elements.
                let map_y = (index / passages.xsize()) % passages.ysize();

                Instance {
                    position: [
                        map_x as f32,
                        map_y as f32,
                        0., // We don't do a depth buffer. z doesn't matter
                    ],
                    passage: passage as u32,
                }
            })
            .collect_vec()
    }

    fn calculate_vertices() -> [Vertex; 12] {
        let rect = egui::Rect::from_min_size(egui::pos2(0., 0.), egui::vec2(32., 32.));
        let center = glam::vec3(rect.center().x, rect.center().y, 0.);
        let top_left = glam::vec3(rect.left_top().x, rect.left_top().y, 0.);
        let top_right = glam::vec3(rect.right_top().x, rect.right_top().y, 0.);
        let bottom_left = glam::vec3(rect.left_bottom().x, rect.left_bottom().y, 0.);
        let bottom_right = glam::vec3(rect.right_bottom().x, rect.right_bottom().y, 0.);

        [
            Vertex {
                position: center,
                direction: 1,
            },
            Vertex {
                position: bottom_left,
                direction: 1,
            },
            Vertex {
                position: bottom_right,
                direction: 1,
            },
            Vertex {
                position: center,
                direction: 2,
            },
            Vertex {
                position: top_left,
                direction: 2,
            },
            Vertex {
                position: bottom_left,
                direction: 2,
            },
            Vertex {
                position: center,
                direction: 4,
            },
            Vertex {
                position: bottom_right,
                direction: 4,
            },
            Vertex {
                position: top_right,
                direction: 4,
            },
            Vertex {
                position: center,
                direction: 8,
            },
            Vertex {
                position: top_right,
                direction: 8,
            },
            Vertex {
                position: top_left,
                direction: 8,
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

        render_pass.draw(0..12, 0..count);
    }

    pub const fn desc() -> wgpu::VertexBufferLayout<'static> {
        const ARRAY: &[wgpu::VertexAttribute] =
            &wgpu::vertex_attr_array![2 => Float32x3, 3 => Uint32];
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: ARRAY,
        }
    }
}
