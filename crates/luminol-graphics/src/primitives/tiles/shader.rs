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
use once_cell::sync::Lazy;

use super::autotiles::Autotiles;
use super::instance::Instances;
use super::opacity::Opacity;
use const_format::str_replace;
use crate::primitives::Vertex;

#[derive(Debug)]
pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
}

impl Shader {
    fn new(use_push_constants: bool) -> Self {
        let render_state = &state!().render_state;

        let shader_module = if use_push_constants {
            render_state
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("tilemap.wgsl (push constants)"),
                    source: wgpu::ShaderSource::Wgsl(
                        str_replace!(
                            str_replace!(
                                concat!(
                                    include_str!("tilemap_header_push_constants.wgsl"),
                                    include_str!("tilemap.wgsl"),
                                ),
                                "FRAGMENT_OPACITY",
                                "HOST.opacity"
                            ),
                            "HOST.",
                            "push_constants."
                        )
                        .into(),
                    ),
                })
        } else {
            render_state
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("tilemap.wgsl (uniforms)"),
                    source: wgpu::ShaderSource::Wgsl(
                        str_replace!(
                            str_replace!(
                                concat!(
                                    include_str!("tilemap_header_uniforms.wgsl"),
                                    include_str!("tilemap.wgsl"),
                                ),
                                "FRAGMENT_OPACITY",
                                "HOST.opacity[input.layer / 4u][input.layer % 4u]"
                            ),
                            "HOST.",
                            ""
                        )
                        .into(),
                    ),
                })
        };

        let pipeline_layout = if use_push_constants {
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Tilemap Render Pipeline Layout (push constants)"),
                    bind_group_layouts: &[crate::image_cache::Cache::bind_group_layout()],
                    push_constant_ranges: &[
                        // Viewport + Autotiles
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: 0..(64 + 36),
                        },
                        // Fragment
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::FRAGMENT,
                            range: (64 + 36)..(64 + 36 + 4),
                        },
                    ],
                })
        } else {
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Tilemap Render Pipeline Layout (uniforms)"),
                    bind_group_layouts: &[
                        image_cache::Cache::bind_group_layout(),
                        Viewport::layout(),
                        Autotiles::layout(),
                        Opacity::layout(),
                    ],
                    push_constant_ranges: &[],
                })
        };

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Tilemap Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader_module,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc(), Instances::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader_module,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                            ..render_state.target_format.into()
                        })],
                    }),
                    primitive: wgpu::PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        Self { pipeline }
    }

    pub fn bind(use_push_constants: bool, render_pass: &mut wgpu::RenderPass<'_>) {
        if use_push_constants {
            render_pass.set_pipeline(&TILEMAP_SHADER_PUSH_CONSTANTS.pipeline)
        } else {
            render_pass.set_pipeline(&TILEMAP_SHADER_UNIFORMS.pipeline)
        }
    }
}

static TILEMAP_SHADER_PUSH_CONSTANTS: Lazy<Shader> = Lazy::new(|| Shader::new(true));
static TILEMAP_SHADER_UNIFORMS: Lazy<Shader> = Lazy::new(|| Shader::new(false));
