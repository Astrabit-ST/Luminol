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
use crate::prelude::*;

#[derive(Debug)]
pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
    pub uniform_layout: wgpu::BindGroupLayout,
}

impl Shader {
    fn new() -> Self {
        let render_state = &state!().render_state;

        let shader_module = render_state
            .device
            .create_shader_module(wgpu::include_wgsl!("tilemap.wgsl"));

        let texture_layout = image_cache::Cache::create_texture_bind_group_layout();
        let uniform_layout =
            render_state
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                    label: Some("tilemap bind group layout"),
                });

        let pipeline_layout =
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Tilemap Render Pipeline Layout"),
                    bind_group_layouts: &[&texture_layout, &uniform_layout],
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

        Self {
            pipeline,
            uniform_layout,
        }
    }

    pub fn bind(render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.set_pipeline(&TILEMAP_SHADER.pipeline)
    }

    pub fn uniform_layout() -> &'static wgpu::BindGroupLayout {
        &TILEMAP_SHADER.uniform_layout
    }
}

static TILEMAP_SHADER: once_cell::sync::Lazy<Shader> = once_cell::sync::Lazy::new(Shader::new);
