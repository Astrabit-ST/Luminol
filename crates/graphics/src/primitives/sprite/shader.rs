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

use std::collections::HashMap;

use naga_oil::compose::ComposerError;

use crate::{primitives::BindGroupLayouts, Vertex};

fn create_shader(
    composer: &mut naga_oil::compose::Composer,
    render_state: &luminol_egui_wgpu::RenderState,
    bind_group_layouts: &BindGroupLayouts,
    target: wgpu::BlendState,
) -> Result<wgpu::RenderPipeline, ComposerError> {
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/translation.wgsl"),
        file_path: "translation.wgsl",
        ..Default::default()
    })?;

    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/hue.wgsl"),
        file_path: "hue.wgsl",
        ..Default::default()
    })?;

    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/gamma.wgsl"),
        file_path: "gamma.wgsl",
        ..Default::default()
    })?;

    let module = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("../shaders/sprite.wgsl"),
        file_path: "sprite.wgsl",
        shader_type: naga_oil::compose::ShaderType::Wgsl,
        shader_defs: HashMap::new(),
        additional_imports: &[],
    })?;

    let shader_module = render_state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Sprite Shader Module"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        });

    let pipeline_layout =
        render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Sprite Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layouts.sprite],
                push_constant_ranges: &[],
            });

    Ok(render_state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Sprite Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
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
        }))
}

const BLEND_ADD: wgpu::BlendState = wgpu::BlendState {
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
};
const BLEND_SUBTRACT: wgpu::BlendState = wgpu::BlendState {
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
};

pub fn create_sprite_shaders(
    composer: &mut naga_oil::compose::Composer,
    render_state: &luminol_egui_wgpu::RenderState,
    bind_group_layouts: &BindGroupLayouts,
) -> Result<HashMap<luminol_data::BlendMode, wgpu::RenderPipeline>, ComposerError> {
    [
        (
            luminol_data::BlendMode::Normal,
            wgpu::BlendState::ALPHA_BLENDING,
        ),
        (luminol_data::BlendMode::Add, BLEND_ADD),
        (luminol_data::BlendMode::Subtract, BLEND_SUBTRACT),
    ]
    .into_iter()
    .map(|(mode, target)| {
        let shader = create_shader(composer, render_state, bind_group_layouts, target)?;
        Ok((mode, shader))
    })
    .collect()
}
