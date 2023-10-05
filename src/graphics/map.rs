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

pub use crate::prelude::*;

use std::time::Duration;

#[derive(Debug)]
pub struct Map {
    resources: Arc<Resources>,
    ani_time: Option<f64>,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub enabled_layers: Vec<bool>,
}

#[derive(Debug)]
struct Resources {
    tiles: primitives::Tiles,
    viewport: primitives::Viewport,
    panorama: Option<Plane>,
    fog: Option<Plane>,
}

type ResourcesSlab = slab::Slab<Arc<Resources>>;

impl Map {
    pub fn new(
        map: &rpg::Map,
        tileset: &rpg::Tileset,
        use_push_constants: bool,
    ) -> Result<Self, String> {
        let atlas = state!().atlas_cache.load_atlas(tileset)?;

        let tiles = primitives::Tiles::new(atlas, &map.data, use_push_constants);

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            Some(Plane::new(
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Panoramas", panorama_name)?,
                tileset.panorama_hue,
                100,
                BlendMode::Normal,
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
                state!()
                    .image_cache
                    .load_wgpu_image("Graphics/Fogs", fog_name)?,
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
        let viewport = primitives::Viewport::new(
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
            resources: Arc::new(Resources {
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

    pub fn set_tile(&self, tile_id: i16, position: (usize, usize, usize)) {
        self.resources.tiles.set_tile(tile_id, position);
    }

    pub fn paint(
        &mut self,
        painter: &egui::Painter,
        selected_layer: Option<usize>,
        rect: egui::Rect,
    ) {
        let time = painter.ctx().input(|i| i.time);
        if let Some(ani_time) = self.ani_time {
            if time - ani_time >= 16. / 60. {
                self.ani_time = Some(time);
                self.resources.tiles.autotiles.inc_ani_index();
            }
        } else {
            self.ani_time = Some(time);
        }

        painter
            .ctx()
            .request_repaint_after(Duration::from_millis(16));

        let resources = self.resources.clone();
        let resource_id = Arc::new(OnceCell::new());

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
                        panorama.draw(viewport, render_pass);
                    }
                }

                tiles.draw(viewport, &enabled_layers, selected_layer, render_pass);
                if fog_enabled {
                    if let Some(fog) = fog {
                        fog.draw(viewport, render_pass);
                    }
                }
            });

        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(paint_callback),
        });
    }
}
