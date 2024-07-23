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

use super::instance::Instances;
use crate::primitives::BindGroupLayouts;

pub fn create_render_pipeline(
    composer: &mut naga_oil::compose::Composer,
    render_state: &luminol_egui_wgpu::RenderState,
    bind_group_layouts: &BindGroupLayouts,
) -> Result<wgpu::RenderPipeline, naga_oil::compose::ComposerError> {
    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/translation.wgsl"),
        file_path: "translation.wgsl",
        ..Default::default()
    })?;

    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/gamma.wgsl"),
        file_path: "gamma.wgsl",
        ..Default::default()
    })?;

    composer.add_composable_module(naga_oil::compose::ComposableModuleDescriptor {
        source: include_str!("../shaders/hue.wgsl"),
        file_path: "hue.wgsl",
        ..Default::default()
    })?;

    let module = composer.make_naga_module(naga_oil::compose::NagaModuleDescriptor {
        source: include_str!("../shaders/tilemap.wgsl"),
        file_path: "tilemap.wgsl",
        shader_type: naga_oil::compose::ShaderType::Wgsl,
        shader_defs: std::collections::HashMap::from([
            (
                "AUTOTILE_ID_AMOUNT".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::AUTOTILE_ID_AMOUNT),
            ),
            (
                "TOTAL_AUTOTILE_ID_AMOUNT".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::TOTAL_AUTOTILE_ID_AMOUNT),
            ),
            (
                "ROWS_UNDER_AUTOTILES_TIMES_COLUMNS".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(
                    super::atlas::ROWS_UNDER_AUTOTILES_TIMES_COLUMNS,
                ),
            ),
            (
                "AUTOTILE_FRAME_COLS".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::AUTOTILE_FRAME_COLS),
            ),
            (
                "TILE_SIZE".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::TILE_SIZE),
            ),
            (
                "TILESET_COLUMNS".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::TILESET_COLUMNS),
            ),
            (
                "ROWS_UNDER_AUTOTILES".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::ROWS_UNDER_AUTOTILES),
            ),
            (
                "TOTAL_AUTOTILE_ROWS".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::TOTAL_AUTOTILE_ROWS),
            ),
            (
                "MAX_SIZE".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::MAX_SIZE),
            ),
            (
                "AUTOTILE_FRAME_WIDTH".to_string(),
                naga_oil::compose::ShaderDefValue::UInt(super::atlas::AUTOTILE_FRAME_WIDTH),
            ),
        ]),
        additional_imports: &[],
    })?;

    let shader_module = render_state
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Tilemap Shader Module"),
            source: wgpu::ShaderSource::Naga(std::borrow::Cow::Owned(module)),
        });

    let pipeline_layout =
        render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tilemap Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layouts.tiles],
                push_constant_ranges: &[],
            });

    Ok(render_state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Instances::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    ..render_state.target_format.into()
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        }))
}
