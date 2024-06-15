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

use color_eyre::eyre::Context;
use image::EncodableLayout;
use itertools::Itertools;
use wgpu::util::DeviceExt;

use std::sync::Arc;

use std::time::Duration;

use fragile::Fragile;

use crate::{
    collision::Collision, grid::Grid, tiles::Tiles, viewport::Viewport, GraphicsState, Plane,
};

pub struct Map {
    resources: Arc<Resources>,
    viewport: Arc<Viewport>,
    ani_time: Option<f64>,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub coll_enabled: bool,
    pub grid_enabled: bool,
    pub enabled_layers: Vec<bool>,
}

struct Resources {
    tiles: Tiles,
    panorama: Option<Plane>,
    fog: Option<Plane>,
    collision: Collision,
    grid: Grid,
}

// wgpu types are not Send + Sync on webassembly, so we use fragile to make sure we never access any wgpu resources across thread boundaries
pub struct Callback {
    resources: Fragile<Arc<Resources>>,
    graphics_state: Fragile<Arc<GraphicsState>>,

    pano_enabled: bool,
    enabled_layers: Vec<bool>,
    selected_layer: Option<usize>,
}

pub struct OverlayCallback {
    resources: Fragile<Arc<Resources>>,
    graphics_state: Fragile<Arc<GraphicsState>>,

    fog_enabled: bool,
    coll_enabled: bool,
    grid_enabled: bool,
}

impl Callback {
    pub fn paint<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let resources = self.resources.get();
        let graphics_state = self.graphics_state.get();

        if self.pano_enabled {
            if let Some(panorama) = &resources.panorama {
                panorama.draw(graphics_state, render_pass);
            }
        }

        resources.tiles.draw(
            graphics_state,
            &self.enabled_layers,
            self.selected_layer,
            render_pass,
        );
    }
}

impl OverlayCallback {
    pub fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let resources = self.resources.get();
        let graphics_state = self.graphics_state.get();

        if self.fog_enabled {
            if let Some(fog) = &resources.fog {
                fog.draw(graphics_state, render_pass);
            }
        }

        if self.coll_enabled {
            resources.collision.draw(graphics_state, render_pass);
        }

        if self.grid_enabled {
            resources.grid.draw(graphics_state, &info, render_pass);
        }
    }
}

impl luminol_egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        self.paint(render_pass);
    }
}

impl luminol_egui_wgpu::CallbackTrait for OverlayCallback {
    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        self.paint(info, render_pass);
    }
}

impl Map {
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        map: &luminol_data::rpg::Map,
        tileset: &luminol_data::rpg::Tileset,
        passages: &luminol_data::Table2,
    ) -> color_eyre::Result<Self> {
        let atlas = graphics_state
            .atlas_loader
            .load_atlas(graphics_state, filesystem, tileset)?;

        let viewport = Arc::new(Viewport::new(
            graphics_state,
            map.width as f32 * 32.,
            map.height as f32 * 32.,
        ));

        let tiles = Tiles::new(graphics_state, viewport.clone(), atlas, &map.data);
        let grid = Grid::new(
            graphics_state,
            viewport.clone(),
            map.data.xsize(),
            map.data.ysize(),
        );
        let collision = Collision::new(graphics_state, viewport.clone(), passages);

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Panoramas", panorama_name)
                .wrap_err_with(|| format!("Error loading map panorama {panorama_name:?}"))
                .unwrap_or_else(|e| {
                    graphics_state.send_texture_error(e);

                    graphics_state
                        .texture_loader
                        .get("placeholder_tile_texture")
                        .unwrap_or_else(|| {
                            let placeholder_img = graphics_state.placeholder_img();

                            graphics_state.texture_loader.register_texture(
                                "placeholder_tile_texture",
                                graphics_state.render_state.device.create_texture_with_data(
                                    &graphics_state.render_state.queue,
                                    &wgpu::TextureDescriptor {
                                        label: Some("placeholder_tile_texture"),
                                        size: wgpu::Extent3d {
                                            width: 32,
                                            height: 32,
                                            depth_or_array_layers: 1,
                                        },
                                        dimension: wgpu::TextureDimension::D2,
                                        mip_level_count: 1,
                                        sample_count: 1,
                                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                        usage: wgpu::TextureUsages::COPY_SRC
                                            | wgpu::TextureUsages::COPY_DST
                                            | wgpu::TextureUsages::TEXTURE_BINDING,
                                        view_formats: &[],
                                    },
                                    wgpu::util::TextureDataOrder::LayerMajor,
                                    &itertools::iproduct!(0..32, 0..32, 0..4)
                                        .map(|(y, x, c)| {
                                            // Tile the placeholder image
                                            placeholder_img.as_bytes()[(c
                                                + (x % placeholder_img.width()) * 4
                                                + (y % placeholder_img.height())
                                                    * 4
                                                    * placeholder_img.width())
                                                as usize]
                                        })
                                        .collect_vec(),
                                ),
                            )
                        })
                });

            Some(Plane::new(
                graphics_state,
                viewport.clone(),
                texture,
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
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Fogs", fog_name)
                .wrap_err_with(|| format!("Error loading map fog {fog_name:?}"))
                .unwrap_or_else(|e| {
                    graphics_state.send_texture_error(e);

                    graphics_state
                        .texture_loader
                        .get("placeholder_tile_texture")
                        .unwrap_or_else(|| {
                            let placeholder_img = graphics_state.placeholder_img();

                            graphics_state.texture_loader.register_texture(
                                "placeholder_tile_texture",
                                graphics_state.render_state.device.create_texture_with_data(
                                    &graphics_state.render_state.queue,
                                    &wgpu::TextureDescriptor {
                                        label: Some("placeholder_tile_texture"),
                                        size: wgpu::Extent3d {
                                            width: 32,
                                            height: 32,
                                            depth_or_array_layers: 1,
                                        },
                                        dimension: wgpu::TextureDimension::D2,
                                        mip_level_count: 1,
                                        sample_count: 1,
                                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                                        usage: wgpu::TextureUsages::COPY_SRC
                                            | wgpu::TextureUsages::COPY_DST
                                            | wgpu::TextureUsages::TEXTURE_BINDING,
                                        view_formats: &[],
                                    },
                                    wgpu::util::TextureDataOrder::LayerMajor,
                                    &itertools::iproduct!(0..32, 0..32, 0..4)
                                        .map(|(y, x, c)| {
                                            // Tile the placeholder image
                                            placeholder_img.as_bytes()[(c
                                                + (x % placeholder_img.width()) * 4
                                                + (y % placeholder_img.height())
                                                    * 4
                                                    * placeholder_img.width())
                                                as usize]
                                        })
                                        .collect_vec(),
                                ),
                            )
                        })
                });

            Some(Plane::new(
                graphics_state,
                viewport.clone(),
                texture,
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
                grid,
            }),
            viewport,

            ani_time: None,

            fog_enabled: true,
            pano_enabled: true,
            coll_enabled: false,
            grid_enabled: true,
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

    pub fn callback(
        &self,
        graphics_state: Arc<GraphicsState>,
        selected_layer: Option<usize>,
    ) -> Callback {
        Callback {
            resources: Fragile::new(self.resources.clone()),
            graphics_state: Fragile::new(graphics_state),
            pano_enabled: self.pano_enabled,
            enabled_layers: self.enabled_layers.clone(),
            selected_layer,
        }
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
            self.callback(graphics_state, selected_layer),
        ));
    }

    pub fn overlay_callback(
        &self,
        graphics_state: Arc<GraphicsState>,
        grid_inner_thickness: f32,
    ) -> OverlayCallback {
        self.resources
            .grid
            .display
            .set_inner_thickness(&graphics_state.render_state, grid_inner_thickness);

        OverlayCallback {
            resources: Fragile::new(self.resources.clone()),
            graphics_state: Fragile::new(graphics_state),
            fog_enabled: self.fog_enabled,
            coll_enabled: self.coll_enabled,
            grid_enabled: self.grid_enabled,
        }
    }

    pub fn paint_overlay(
        &self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
        grid_inner_thickness: f32,
        rect: egui::Rect,
    ) {
        painter.add(luminol_egui_wgpu::Callback::new_paint_callback(
            rect,
            self.overlay_callback(graphics_state, grid_inner_thickness),
        ));
    }
}
