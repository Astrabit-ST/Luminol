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

use std::sync::Arc;

use crate::{
    viewport::{self, Viewport},
    BindGroupBuilder, BindGroupLayoutBuilder, GraphicsState,
};

pub use atlas::Atlas;

use autotiles::Autotiles;
use instance::Instances;
use opacity::Opacity;

mod atlas;
mod autotile_ids;
pub(crate) mod autotiles;
mod instance;
pub(crate) mod opacity;
pub(crate) mod shader;

pub struct Tiles {
    pub autotiles: Autotiles,
    pub atlas: Atlas,
    pub instances: Instances,
    pub opacity: Opacity,
    pub viewport: Arc<Viewport>,

    pub bind_group: wgpu::BindGroup,
}

impl Tiles {
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: Arc<Viewport>,
        atlas: Atlas,
        tiles: &luminol_data::Table3,
    ) -> Self {
        let autotiles = Autotiles::new(graphics_state, &atlas);
        let instances = Instances::new(
            &graphics_state.render_state,
            tiles,
            atlas.atlas_texture.size(),
        );
        let opacity = Opacity::new(graphics_state);

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder
            .append_texture_view(&atlas.atlas_texture.view)
            .append_sampler(&graphics_state.nearest_sampler);
        if !graphics_state.push_constants_supported() {
            bind_group_builder
                .append_buffer(viewport.as_buffer().unwrap())
                .append_buffer(autotiles.as_buffer().unwrap())
                .append_buffer(opacity.as_buffer().unwrap());
        }
        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("tilemap bind group"),
            &graphics_state.bind_group_layouts.tiles,
        );

        Self {
            autotiles,
            atlas,
            instances,
            opacity,

            bind_group,
            viewport,
        }
    }

    pub fn set_tile(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        tile_id: i16,
        position: (usize, usize, usize),
    ) {
        self.instances.set_tile(render_state, tile_id, position)
    }

    pub fn draw<'rpass>(
        &'rpass self,
        graphics_state: &'rpass GraphicsState,
        enabled_layers: &[bool],
        selected_layer: Option<usize>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct VertexPushConstant {
            viewport: [u8; 64],
            autotiles: [u8; 48],
        }

        render_pass.push_debug_group("tilemap tiles renderer");
        render_pass.set_pipeline(&graphics_state.pipelines.tiles);
        render_pass.set_bind_group(0, &self.bind_group, &[]);

        if graphics_state.push_constants_supported() {
            render_pass.set_push_constants(
                wgpu::ShaderStages::VERTEX,
                0,
                bytemuck::bytes_of(&VertexPushConstant {
                    viewport: self.viewport.as_bytes(),
                    autotiles: self.autotiles.as_bytes(),
                }),
            );
        }

        for (layer, enabled) in enabled_layers.iter().copied().enumerate() {
            let opacity = if selected_layer.is_some_and(|s| s != layer) {
                0.5
            } else {
                1.0
            };
            if enabled {
                self.opacity
                    .set_opacity(&graphics_state.render_state, layer, opacity);
                if graphics_state.push_constants_supported() {
                    render_pass.set_push_constants(
                        wgpu::ShaderStages::FRAGMENT,
                        64 + 48,
                        bytemuck::bytes_of::<f32>(&opacity),
                    );
                }

                self.instances.draw(render_pass, layer);
            }
        }
        render_pass.pop_debug_group();
    }
}

pub fn create_bind_group_layout(
    render_state: &luminol_egui_wgpu::RenderState,
) -> wgpu::BindGroupLayout {
    let mut builder = BindGroupLayoutBuilder::new();
    builder
        .append(
            wgpu::ShaderStages::VERTEX_FRAGMENT,
            wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            None,
        )
        .append(
            wgpu::ShaderStages::FRAGMENT,
            wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
            None,
        );

    if !crate::push_constants_supported(render_state) {
        viewport::add_to_bind_group_layout(&mut builder);
        autotiles::add_to_bind_group_layout(&mut builder);
        opacity::add_to_bind_group_layout(&mut builder);
    }

    builder.build(&render_state.device, Some("tilemap bind group layout"))
}
