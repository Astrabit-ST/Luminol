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

use super::graphic::Graphic;
use super::{BlendMode, Vertex};
use const_format::str_replace;
use primitives::Viewport;

pub struct Shader {
    pub pipeline: wgpu::RenderPipeline,
}

impl Shader {
    pub fn new(target: wgpu::BlendState, use_push_constants: bool) -> Self {
        let render_state = &state!().render_state;

        let shader_module = if use_push_constants {
            render_state
                .device
                .create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("sprite.wgsl (push constants)"),
                    source: wgpu::ShaderSource::Wgsl(
                        str_replace!(
                            concat!(
                                include_str!("sprite_header_push_constants.wgsl"),
                                include_str!("sprite.wgsl"),
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
                    label: Some("sprite.wgsl (uniforms)"),
                    source: wgpu::ShaderSource::Wgsl(
                        str_replace!(
                            concat!(
                                include_str!("sprite_header_uniforms.wgsl"),
                                include_str!("sprite.wgsl"),
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
                    label: Some("Tilemap Sprite Pipeline Layout (push constants)"),
                    bind_group_layouts: &[image_cache::Cache::bind_group_layout()],
                    push_constant_ranges: &[
                        // Viewport
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::VERTEX,
                            range: 0..64,
                        },
                        wgpu::PushConstantRange {
                            stages: wgpu::ShaderStages::FRAGMENT,
                            range: 64..(64 + 4 + 4 + 4),
                        },
                    ],
                })
        } else {
            render_state
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Tilemap Sprite Pipeline Layout (uniforms)"),
                    bind_group_layouts: &[
                        image_cache::Cache::bind_group_layout(),
                        Viewport::layout(),
                        Graphic::layout(),
                    ],
                    push_constant_ranges: &[],
                })
        };

        let pipeline =
            render_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Tilemap Sprite Render Pipeline"),
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
                            blend: Some(target),
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

        Shader { pipeline }
    }

    pub fn bind(mode: BlendMode, use_push_constants: bool, render_pass: &mut wgpu::RenderPass<'_>) {
        if use_push_constants {
            render_pass.set_pipeline(&EVENT_SHADERS_PUSH_CONSTANTS[&mode].pipeline)
        } else {
            render_pass.set_pipeline(&EVENT_SHADERS_UNIFORMS[&mode].pipeline)
        }
    }
}

static EVENT_SHADERS_PUSH_CONSTANTS: Lazy<HashMap<BlendMode, Shader>> = Lazy::new(|| {
    [
        (BlendMode::Normal, wgpu::BlendState::ALPHA_BLENDING),
        (
            BlendMode::Add,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
        ),
        (
            BlendMode::Subtract,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
            },
        ),
    ]
    .into_iter()
    .map(|(mode, target)| (mode, Shader::new(target, true)))
    .collect()
});

static EVENT_SHADERS_UNIFORMS: Lazy<HashMap<BlendMode, Shader>> = Lazy::new(|| {
    [
        (BlendMode::Normal, wgpu::BlendState::ALPHA_BLENDING),
        (
            BlendMode::Add,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            },
        ),
        (
            BlendMode::Subtract,
            wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::ReverseSubtract,
                },
            },
        ),
    ]
    .into_iter()
    .map(|(mode, target)| (mode, Shader::new(target, false)))
    .collect()
});
