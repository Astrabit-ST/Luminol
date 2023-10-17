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

type ResourcesSlab = slab::Slab<std::sync::Arc<Resources>>;

impl Map {
    pub fn new(
        graphics_state: &crate::GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        map: &luminol_data::rpg::Map,
        tileset: &luminol_data::rpg::Tileset,
        use_push_constants: bool,
    ) -> Result<Self, String> {
        let atlas = graphics_state.atlas_cache.load_atlas(
            graphics_state,
            filesystem,
            &graphics_state.image_cache,
            tileset,
        )?;

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
        graphics_state: &'static crate::GraphicsState,
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
            .request_repaint_after(Duration::from_millis(16));

        let resources = self.resources.clone();
        let resource_id = std::sync::Arc::new(once_cell::sync::OnceCell::new());

        let prepare_id = resource_id;
        let paint_id = prepare_id.clone();

        let fog_enabled = self.fog_enabled;
        let pano_enabled = self.pano_enabled;
        let enabled_layers = self.enabled_layers.clone();

        let paint_callback = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, _queue, _encoder, paint_callback_resources| {
                let res_hash: &mut ResourcesSlab = paint_callback_resources
                    .entry()
                    .or_insert_with(Default::default);
                let id = res_hash.insert(resources.clone());
                prepare_id.set(id).expect("resources id already set?");

                vec![]
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let res_hash: &ResourcesSlab = paint_callback_resources.get().unwrap();
                let id = paint_id.get().copied().expect("resources id is unset");
                let resources = &res_hash[id];
                let Resources {
                    tiles,
                    viewport,
                    panorama,
                    fog,
                    ..
                } = resources.as_ref();

                viewport.bind(render_pass);

                if pano_enabled {
                    if let Some(panorama) = panorama {
                        panorama.draw(graphics_state, viewport, render_pass);
                    }
                }

                tiles.draw(
                    graphics_state,
                    viewport,
                    &enabled_layers,
                    selected_layer,
                    render_pass,
                );
                if fog_enabled {
                    if let Some(fog) = fog {
                        fog.draw(graphics_state, viewport, render_pass);
                    }
                }
            });

        painter.add(egui::PaintCallback {
            rect,
            callback: std::sync::Arc::new(paint_callback),
        });
    }
}
