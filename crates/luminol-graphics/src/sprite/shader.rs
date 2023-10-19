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

fn create_shader(
    render_state: &egui_wgpu::RenderState,
    bind_group_layouts: &crate::BindGroupLayouts,
    target: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let shader_module = render_state
        .device
        .create_shader_module(wgpu::include_wgsl!("sprite.wgsl"));

    let pipeline_layout =
        render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tilemap Sprite Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layouts.image_cache_texture],
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
            });
    render_state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Sprite Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[crate::vertex::Vertex::desc()],
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
        })
}

pub fn create_sprite_shaders(
    render_state: &egui_wgpu::RenderState,
    bind_group_layouts: &crate::BindGroupLayouts,
) -> std::collections::HashMap<luminol_data::BlendMode, wgpu::RenderPipeline> {
    [
        (
            luminol_data::BlendMode::Normal,
            wgpu::BlendState::ALPHA_BLENDING,
        ),
        (
            luminol_data::BlendMode::Add,
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
            luminol_data::BlendMode::Subtract,
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
    .map(|(mode, target)| {
        (
            mode,
            create_shader(render_state, bind_group_layouts, target),
        )
    })
    .collect()
}