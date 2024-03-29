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

use super::instance::Instances;
use super::Vertex;

pub fn create_render_pipeline(
    render_state: &luminol_egui_wgpu::RenderState,
    bind_group_layouts: &crate::BindGroupLayouts,
) -> wgpu::RenderPipeline {
    let push_constants_supported = crate::push_constants_supported(render_state);

    let mut composer = naga_oil::compose::Composer::default().with_capabilities(
        push_constants_supported
            .then_some(naga::valid::Capabilities::PUSH_CONSTANT)
            .unwrap_or_default(),
    );

    let result = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("collision.wgsl"),
        file_path: "collision.wgsl",
        shader_type: naga_oil::compose::ShaderType::Wgsl,
        shader_defs: std::collections::HashMap::from([(
            "USE_PUSH_CONSTANTS".to_string(),
            naga_oil::compose::ShaderDefValue::Bool(push_constants_supported),
        )]),
        additional_imports: &[],
    });
    let module = match result {
        Ok(module) => module,
        Err(e) => {
            let error = e.emit_to_string(&composer);
            panic!("{error}");
        }
    };

    let shader_module = render_state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tilemap Collision Shader Module"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        });

    let push_constant_ranges: &[_] = if push_constants_supported {
        &[
            // Viewport
            wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX,
                range: 0..64,
            },
        ]
    } else {
        &[]
    };
    let label = if push_constants_supported {
        "Tilemap Collision Render Pipeline Layout (push constants)"
    } else {
        "Tilemap Collision Render Pipeline Layout (uniforms)"
    };

    let collision_bgl: &wgpu::BindGroupLayout = &bind_group_layouts.collision;
    let bind_group_layout_slice = std::slice::from_ref(&collision_bgl);
    let bind_group_layouts: &[&wgpu::BindGroupLayout] = if push_constants_supported {
        &[]
    } else {
        bind_group_layout_slice
    };

    let pipeline_layout =
        render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts,
                push_constant_ranges,
            });

    render_state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Collision Render Pipeline"),
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
        })
}
