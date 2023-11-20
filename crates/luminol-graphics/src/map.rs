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

use crate::Plane;
use std::sync::Arc;

use std::time::Duration;

#[derive(Debug)]
pub struct Map {
    resources: std::sync::Arc<Resources>,
    ani_time: Option<f64>,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub enabled_layers: Vec<bool>,
}

#[derive(Debug)]
struct Resources {
    tiles: crate::tiles::Tiles,
    viewport: crate::viewport::Viewport,
    panorama: Option<Plane>,
    fog: Option<Plane>,
}

struct Callback {
    resources: Arc<Resources>,
    graphics_state: Arc<crate::GraphicsState>,

    fog_enabled: bool,
    pano_enabled: bool,
    enabled_layers: Vec<bool>,
    selected_layer: Option<usize>,
}

impl egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        self.resources.viewport.bind(render_pass);

        if self.pano_enabled {
            if let Some(panorama) = &self.resources.panorama {
                panorama.draw(&self.graphics_state, &self.resources.viewport, render_pass);
            }
        }

        self.resources.tiles.draw(
            &self.graphics_state,
            &self.resources.viewport,
            &self.enabled_layers,
            self.selected_layer,
            render_pass,
        );
        if self.fog_enabled {
            if let Some(fog) = &self.resources.fog {
                fog.draw(&self.graphics_state, &self.resources.viewport, render_pass);
            }
        }
    }
}

impl Map {
    pub fn new(
        graphics_state: &crate::GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        map: &luminol_data::rpg::Map,
        tileset: &luminol_data::rpg::Tileset,
        use_push_constants: bool,
    ) -> anyhow::Result<Self> {
        let atlas = graphics_state
            .atlas_cache
            .load_atlas(graphics_state, filesystem, tileset)?;

        let tiles = crate::tiles::Tiles::new(graphics_state, atlas, &map.data, use_push_constants);

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            Some(Plane::new(
                graphics_state,
                graphics_state.image_cache.load_wgpu_image(
                    graphics_state,
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
                use_push_constants,
            ))
        } else {
            None
        };
        let fog = if let Some(ref fog_name) = tileset.fog_name {
            Some(Plane::new(
                graphics_state,
                graphics_state.image_cache.load_wgpu_image(
                    graphics_state,
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
                use_push_constants,
            ))
        } else {
            None
        };
        let viewport = crate::viewport::Viewport::new(
            graphics_state,
            glam::Mat4::orthographic_rh(
                0.0,
                map.width as f32 * 32.,
                map.height as f32 * 32.,
                0.0,
                -1.0,
                1.0,
            ),
            use_push_constants,
        );

        Ok(Self {
            resources: std::sync::Arc::new(Resources {
                tiles,
                viewport,
                panorama,
                fog,
            }),

            ani_time: None,

            fog_enabled: true,
            pano_enabled: true,
            enabled_layers: vec![true; map.data.zsize()],
        })
    }

    pub fn set_tile(
        &self,
        render_state: &egui_wgpu::RenderState,
        tile_id: i16,
        position: (usize, usize, usize),
    ) {
        self.resources
            .tiles
            .set_tile(render_state, tile_id, position);
    }

    pub fn set_proj(&self, render_state: &egui_wgpu::RenderState, proj: glam::Mat4) {
        self.resources.viewport.set_proj(render_state, proj);
    }

    pub fn paint(
        &mut self,
        graphics_state: Arc<crate::GraphicsState>,
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

        painter.add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Callback {
                resources: self.resources.clone(),
                graphics_state,

                fog_enabled: self.fog_enabled,
                pano_enabled: self.pano_enabled,
                enabled_layers: self.enabled_layers.clone(),
                selected_layer,
            },
        ));
    }
}
