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

pub mod quad;
pub mod sprite;
pub mod tiles;
pub mod vertex;
pub mod viewport;

pub mod event;
pub mod map;
pub mod plane;

pub mod atlas_cache;
pub mod image_cache;

pub use event::Event;
pub use map::Map;
pub use plane::Plane;

pub struct GraphicsState {
    pub image_cache: image_cache::Cache,
    pub atlas_cache: atlas_cache::Cache,
    pub render_state: egui_wgpu::RenderState,

    pipelines: Pipelines,
    bind_group_layouts: BindGroupLayouts,
}

pub struct BindGroupLayouts {
    image_cache_texture: wgpu::BindGroupLayout,
    viewport: wgpu::BindGroupLayout,
    sprite_graphic: wgpu::BindGroupLayout,
    atlas_autotiles: wgpu::BindGroupLayout,
    tile_layer_opacity: wgpu::BindGroupLayout,
}

pub struct Pipelines {
    sprites: std::collections::HashMap<luminol_data::BlendMode, wgpu::RenderPipeline>,
    tiles: wgpu::RenderPipeline,
}

impl GraphicsState {
    pub fn new(render_state: egui_wgpu::RenderState) -> Self {
        let bind_group_layouts = BindGroupLayouts {
            image_cache_texture: image_cache::create_bind_group_layout(&render_state),
            viewport: viewport::create_bind_group_layout(&render_state),
            sprite_graphic: sprite::graphic::create_bind_group_layout(&render_state),
            atlas_autotiles: tiles::autotiles::create_bind_group_layout(&render_state),
            tile_layer_opacity: tiles::opacity::create_bind_group_layout(&render_state),
        };

        let pipelines = Pipelines {
            sprites: sprite::shader::create_sprite_shaders(&render_state, &bind_group_layouts),
            tiles: tiles::shader::create_render_pipeline(&render_state, &bind_group_layouts),
        };

        let image_cache = image_cache::Cache::default();
        let atlas_cache = atlas_cache::Cache::default();

        Self {
            image_cache,
            atlas_cache,
            render_state,
            pipelines,
            bind_group_layouts,
        }
    }
}
