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

use std::sync::Arc;

use crate::{
    BindGroupBuilder, BindGroupLayoutBuilder, Drawable, GraphicsState, Renderable, Transform,
    Viewport,
};

pub use atlas::*;

use autotiles::Autotiles;
use display::Display;
use instance::Instances;

mod atlas;
mod autotile_ids;
pub(crate) mod autotiles;
pub(crate) mod display;
mod instance;
pub(crate) mod shader;

pub struct Tiles {
    pub autotiles: Autotiles,
    pub display: Display,
    pub transform: Transform,
    pub enabled_layers: Vec<bool>,
    pub selected_layer: Option<usize>,
    pub auto_opacity: bool,

    instances: Arc<Instances>,
    bind_group: Arc<wgpu::BindGroup>,
}

impl Tiles {
    pub fn new(
        graphics_state: &GraphicsState,
        tiles: &luminol_data::Table3,
        // in order of use in bind group
        atlas: &Atlas,
        viewport: &Viewport,
        transform: Transform,
    ) -> Self {
        let autotiles = Autotiles::new(graphics_state, atlas);
        let instances = Instances::new(&graphics_state.render_state, tiles);
        let display = Display::new(
            graphics_state,
            tiles.xsize() as u32,
            tiles.ysize() as u32,
            tiles.zsize(),
        );

        let mut bind_group_builder = BindGroupBuilder::new();
        bind_group_builder
            .append_texture_view(&atlas.atlas_texture.view)
            .append_sampler(&graphics_state.nearest_sampler)
            .append_buffer(viewport.as_buffer())
            .append_buffer(transform.as_buffer())
            .append_buffer(autotiles.as_buffer())
            .append_buffer_with_size(display.as_buffer(), display.aligned_layer_size() as u64);

        let bind_group = bind_group_builder.build(
            &graphics_state.render_state.device,
            Some("tilemap bind group"),
            &graphics_state.bind_group_layouts.tiles,
        );

        Self {
            autotiles,
            display,
            transform,
            enabled_layers: vec![true; tiles.zsize()],
            selected_layer: None,
            auto_opacity: true,

            instances: Arc::new(instances),
            bind_group: Arc::new(bind_group),
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
}

pub struct Prepared {
    bind_group: Arc<wgpu::BindGroup>,
    instances: Arc<Instances>,
    graphics_state: Arc<GraphicsState>,

    layer_offsets: Vec<u32>,
    enabled_layers: Vec<bool>,
}

impl Renderable for Tiles {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &Arc<GraphicsState>) -> Self::Prepared {
        let bind_group = Arc::clone(&self.bind_group);
        let graphics_state = Arc::clone(graphics_state);
        let instances = Arc::clone(&self.instances);

        if self.auto_opacity {
            for layer in 0..self.enabled_layers.len() {
                let opacity = if self.selected_layer.is_some_and(|s| s != layer) {
                    0.5
                } else {
                    1.0
                };
                self.display
                    .set_opacity(&graphics_state.render_state, opacity, layer);
            }
        }

        Prepared {
            bind_group,
            instances,
            graphics_state,

            layer_offsets: self.display.layer_offsets(),
            enabled_layers: self.enabled_layers.clone(),
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap tiles renderer");
        render_pass.set_pipeline(&self.graphics_state.pipelines.tiles);

        for layer in self
            .enabled_layers
            .iter()
            .enumerate()
            .filter_map(|(layer, enabled)| enabled.then_some(layer))
        {
            render_pass.set_bind_group(0, &self.bind_group, &[self.layer_offsets[layer]]);

            self.instances.draw(render_pass, layer);
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

    Viewport::add_to_bind_group_layout(&mut builder);
    Transform::add_to_bind_group_layout(&mut builder);
    autotiles::add_to_bind_group_layout(&mut builder);
    display::add_to_bind_group_layout(&mut builder);

    builder.build(&render_state.device, Some("tilemap bind group layout"))
}
