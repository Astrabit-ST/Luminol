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
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub struct Instances {
    vertex_buffer: wgpu::Buffer,
    map_size: u32,
}

impl Instances {
    pub fn new(
        render_state: &luminol_egui_wgpu::RenderState,
        map_width: u32,
        map_height: u32,
    ) -> Self {
        let vertices = Self::calculate_vertices(render_state);
        let vertex_buffer =
            render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap grid vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });

        Self {
            vertex_buffer,
            map_size: map_width * map_height,
        }
    }

    fn calculate_vertices(render_state: &luminol_egui_wgpu::RenderState) -> [Vertex; 6] {
        // OpenGL and WebGL use the last vertex in each triangle as the provoking vertex, and
        // Direct3D, Metal, Vulkan and WebGPU use the first vertex in each triangle
        if render_state.adapter.get_info().backend == wgpu::Backend::Gl {
            [
                Vertex {
                    position: glam::vec2(1., 0.),
                },
                Vertex {
                    position: glam::vec2(0., 1.),
                },
                Vertex {
                    position: glam::vec2(0., 0.), // Provoking vertex
                },
                Vertex {
                    position: glam::vec2(0., 1.),
                },
                Vertex {
                    position: glam::vec2(1., 0.),
                },
                Vertex {
                    position: glam::vec2(1., 1.), // Provoking vertex
                },
            ]
        } else {
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
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        render_pass.draw(0..6, 0..self.map_size);
    }
}
