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

mod quad;
mod sprite;
mod tiles;
mod vertex;
mod viewport;

mod event;
mod map;
mod plane;

pub mod atlas_cache;
pub mod image_cache;

pub use event::Event;
pub use map::Map;
pub use plane::Plane;

pub struct GraphicsState {
    image_cache: image_cache::Cache,
    atlas_cache: atlas_cache::Cache,
    render_state: egui_wgpu::RenderState,

    pipelines: Pipelines,
    bind_group_layouts: BindGroupLayouts,
}

pub struct BindGroupLayouts {
    image_cache_texture: wgpu::BindGroupLayout,
}

pub struct Pipelines {
    sprites: std::collections::HashMap<luminol_data::BlendMode, wgpu::RenderPipeline>,
    tiles: wgpu::RenderPipeline,
}
