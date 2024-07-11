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
#![cfg_attr(target_arch = "wasm32", allow(clippy::arc_with_non_send_sync))]

pub mod binding_helpers;
pub use binding_helpers::{BindGroupBuilder, BindGroupLayoutBuilder};

pub mod loaders;
pub use loaders::texture::Texture;

// Building blocks that make up more complex parts (i.e. the map view, or events)
pub mod primitives;
pub use primitives::{
    collision::Collision, grid::Grid, sprite::Sprite, tiles::Atlas, tiles::Tiles,
};

pub mod data;
pub use data::*;

pub mod event;
pub mod map;
pub mod plane;
pub mod tilepicker;

pub use event::Event;
pub use map::Map;
pub use plane::Plane;
pub use tilepicker::Tilepicker;

pub struct GraphicsState {
    pub texture_loader: loaders::texture::Loader,
    pub atlas_loader: loaders::atlas::Loader,
    pub render_state: luminol_egui_wgpu::RenderState,

    pub nearest_sampler: wgpu::Sampler,

    pipelines: primitives::Pipelines,
    bind_group_layouts: primitives::BindGroupLayouts,

    texture_error_tx: crossbeam::channel::Sender<color_eyre::Report>,
    texture_error_rx: crossbeam::channel::Receiver<color_eyre::Report>,
}

impl GraphicsState {
    pub fn new(render_state: luminol_egui_wgpu::RenderState) -> Self {
        let bind_group_layouts = primitives::BindGroupLayouts::new(&render_state);
        let pipelines = primitives::Pipelines::new(&render_state, &bind_group_layouts);

        let texture_loader = loaders::texture::Loader::new(render_state.clone());
        let atlas_cache = loaders::atlas::Loader::default();

        let nearest_sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                label: Some("luminol nearest texture sampler"),
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                ..Default::default()
            });

        let (texture_error_tx, texture_error_rx) = crossbeam::channel::unbounded();

        Self {
            texture_loader,
            atlas_loader: atlas_cache,
            render_state,

            nearest_sampler,

            pipelines,
            bind_group_layouts,

            texture_error_tx,
            texture_error_rx,
        }
    }

    pub fn send_texture_error(&self, error: color_eyre::Report) {
        self.texture_error_tx
            .try_send(error)
            .expect("failed to send texture error");
    }

    pub fn texture_errors(&self) -> impl Iterator<Item = color_eyre::Report> + '_ {
        self.texture_error_rx.try_iter()
    }
}

pub trait Renderable {
    type Prepared: Drawable;
    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared;
}

pub trait Drawable {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>);
}

pub struct Painter<T> {
    prepared: fragile::Fragile<T>,
}

impl<T> Painter<T> {
    pub fn new(prepared: T) -> Self {
        Self {
            prepared: fragile::Fragile::new(prepared),
        }
    }
}

impl<T: Drawable> luminol_egui_wgpu::CallbackTrait for Painter<T> {
    fn paint<'a>(
        &'a self,
        _: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        self.prepared.get().draw(render_pass);
    }
}
