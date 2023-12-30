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

use std::time::Duration;

use crate::{collision::Collision, tiles::Tiles, viewport::Viewport, GraphicsState, Plane};

pub struct Map {
    resources: Arc<Resources>,
    viewport: Arc<Viewport>,
    ani_time: Option<f64>,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub coll_enabled: bool,
    pub enabled_layers: Vec<bool>,
}

struct Resources {
    tiles: Tiles,
    panorama: Option<Plane>,
    fog: Option<Plane>,
    collision: Collision,
}

struct Callback {
    resources: Arc<Resources>,
    graphics_state: Arc<GraphicsState>,

    pano_enabled: bool,
    enabled_layers: Vec<bool>,
    selected_layer: Option<usize>,
}

struct OverlayCallback {
    resources: Arc<Resources>,
    graphics_state: Arc<GraphicsState>,

    fog_enabled: bool,
    coll_enabled: bool,
}

//? SAFETY:
//? wgpu resources are not Send + Sync on wasm, but egui_wgpu::CallbackTrait requires Send + Sync (because egui::Context is Send + Sync)
//? as long as this callback does not leave the thread it was created on on wasm (which it shouldn't be) these are ok.
#[allow(unsafe_code)]
unsafe impl Send for Callback {}
#[allow(unsafe_code)]
unsafe impl Sync for Callback {}
#[allow(unsafe_code)]
unsafe impl Send for OverlayCallback {}
#[allow(unsafe_code)]
unsafe impl Sync for OverlayCallback {}

impl luminol_egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        if self.pano_enabled {
            if let Some(panorama) = &self.resources.panorama {
                panorama.draw(&self.graphics_state, render_pass);
            }
        }

        self.resources.tiles.draw(
            &self.graphics_state,
            &self.enabled_layers,
            self.selected_layer,
            render_pass,
        );
    }
}

impl luminol_egui_wgpu::CallbackTrait for OverlayCallback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        if self.fog_enabled {
            if let Some(fog) = &self.resources.fog {
                fog.draw(&self.graphics_state, render_pass);
            }
        }

        if self.coll_enabled {
            self.resources
                .collision
                .draw(&self.graphics_state, render_pass);
        }
    }
}

impl Map {
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        map: &luminol_data::rpg::Map,
        tileset: &luminol_data::rpg::Tileset,
        passages: &luminol_data::Table2,
    ) -> color_eyre::eyre::Result<Self> {
        let atlas = graphics_state
            .atlas_loader
            .load_atlas(graphics_state, filesystem, tileset)?;

        let viewport = Arc::new(Viewport::new(
            graphics_state,
            map.width as f32 * 32.,
            map.height as f32 * 32.,
        ));

        let tiles = Tiles::new(graphics_state, viewport.clone(), atlas, &map.data);
        let collision = Collision::new(graphics_state, viewport.clone(), passages);

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            Some(Plane::new(
                graphics_state,
                viewport.clone(),
                graphics_state.texture_loader.load_now_dir(
                    filesystem,
                    "Graphics/Panoramas",
                    panorama_name,
                )?,
                tileset.panorama_hue,
                100,
                luminol_data::BlendMode::Normal,
                255,
                map.width,
                map.height,
            ))
        } else {
            None
        };
        let fog = if let Some(ref fog_name) = tileset.fog_name {
            Some(Plane::new(
                graphics_state,
                viewport.clone(),
                graphics_state.texture_loader.load_now_dir(
                    filesystem,
                    "Graphics/Fogs",
                    fog_name,
                )?,
                tileset.fog_hue,
                tileset.fog_zoom,
                tileset.fog_blend_type,
                tileset.fog_opacity,
                map.width,
                map.height,
            ))
        } else {
            None
        };

        Ok(Self {
            resources: std::sync::Arc::new(Resources {
                tiles,
                panorama,
                fog,
                collision,
            }),
            viewport,

            ani_time: None,

            fog_enabled: true,
            pano_enabled: true,
            coll_enabled: false,
            enabled_layers: vec![true; map.data.zsize()],
        })
    }

    pub fn set_tile(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        tile_id: i16,
        position: (usize, usize, usize),
    ) {
        self.resources
            .tiles
            .set_tile(render_state, tile_id, position);
    }

    pub fn set_passage(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        passage: i16,
        position: (usize, usize),
    ) {
        self.resources
            .collision
            .set_passage(render_state, passage, position);
    }

    pub fn set_proj(&self, render_state: &luminol_egui_wgpu::RenderState, proj: glam::Mat4) {
        self.viewport.set_proj(render_state, proj);
    }

    pub fn paint(
        &mut self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
        selected_layer: Option<usize>,
        rect: egui::Rect,
    ) {
        let time = painter.ctx().input(|i| i.time);
        if let Some(ani_time) = self.ani_time {
            if time - ani_time >= 16. / 60. {
                self.ani_time = Some(time);
                self.resources
                    .tiles
                    .autotiles
                    .inc_ani_index(&graphics_state.render_state);
            }
        } else {
            self.ani_time = Some(time);
        }

        painter
            .ctx()
            .request_repaint_after(Duration::from_secs_f64(16. / 60.));

        painter.add(luminol_egui_wgpu::Callback::new_paint_callback(
            rect,
            Callback {
                resources: self.resources.clone(),
                graphics_state,

                pano_enabled: self.pano_enabled,
                enabled_layers: self.enabled_layers.clone(),
                selected_layer,
            },
        ));
    }

    pub fn paint_overlay(
        &mut self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        painter.add(luminol_egui_wgpu::Callback::new_paint_callback(
            rect,
            OverlayCallback {
                resources: self.resources.clone(),
                graphics_state,

                fog_enabled: self.fog_enabled,
                coll_enabled: self.coll_enabled,
            },
        ));
    }
}
