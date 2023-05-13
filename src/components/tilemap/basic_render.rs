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
use super::vertex::Vertex;
use crate::prelude::*;

use eframe::wgpu::util::DeviceExt;
use once_cell::sync::Lazy;

#[derive(Debug)]
pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,

    pub vertices: wgpu::Buffer,
}

impl Shader {
    fn new() -> Self {
        let render_state = &state!().render_state;

        let shader_module = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("basic.wgsl"));

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Tilemap Render Pipeline Layout"),
                    bind_group_layouts: &[image_cache::Cache::bind_group_layout()],
                    push_constant_ranges: &[],
                });
        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Tilemap Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            ..render_state.target_format.into()
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        // polygon_mode: wgpu::PolygonMode::Line,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let vertices = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("basic tilemap renderer vertices this is stupid why do i need this"),
                contents: bytemuck::cast_slice(&[
                    Vertex {
                        position: [-1., 1., 0.],
                        tex_coords: [0., 0.],
                    },
                    Vertex {
                        position: [1., 1., 0.0],
                        tex_coords: [1., 0.],
                    },
                    Vertex {
                        position: [1., -1., 0.0],
                        tex_coords: [1., 1.],
                    },
                    //
                    Vertex {
                        position: [1., -1., 0.0],
                        tex_coords: [1., 1.],
                    },
                    Vertex {
                        position: [-1., -1., 0.0],
                        tex_coords: [0., 1.],
                    },
                    Vertex {
                        position: [-1., 1., 0.0],
                        tex_coords: [0., 0.],
                    },
                ]),
                usage: wgpu::BufferUsages::VERTEX,
            });

        Self { pipeline, vertices }
    }

    pub fn draw(render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&SHADER.pipeline);
        render_pass.set_vertex_buffer(0, SHADER.vertices.slice(..));
        render_pass.draw(0..6, 0..1);
    }
}

static SHADER: Lazy<Shader> = Lazy::new(Shader::new);
