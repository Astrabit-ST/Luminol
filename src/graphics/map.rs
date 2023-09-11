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

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct Map {
    resources: Arc<Resources>,
    ani_instant: Instant,

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
    pub fn new(map: &rpg::Map, tileset: &rpg::Tileset) -> Result<Self, String> {
        let atlas = state!().atlas_cache.load_atlas(tileset)?;

        let tiles = primitives::Tiles::new(atlas, &map.data);

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
            ))
        } else {
            None
        };
        let viewport = primitives::Viewport::new(cgmath::ortho(
            0.0,
            map.width as f32 * 32.,
            map.height as f32 * 32.,
            0.0,
            -1.0,
            1.0,
        ));

        Ok(Self {
            resources: Arc::new(Resources {
                tiles,
                viewport,
                panorama,
                fog,
            }),

            ani_instant: Instant::now(),

            fog_enabled: true,
            pano_enabled: true,
            enabled_layers: vec![true; map.data.zsize()],
        })
    }

    pub fn paint(&mut self, painter: &egui::Painter, rect: egui::Rect) {
        if self.ani_instant.elapsed() >= Duration::from_secs_f32((1. / 60.) * 16.) {
            self.ani_instant = Instant::now();
            self.resources.tiles.autotiles.inc_ani_index();
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
                        panorama.draw(render_pass);
                    }
                }

                tiles.draw(render_pass, Some(&enabled_layers));
                if fog_enabled {
                    if let Some(fog) = fog {
                        fog.draw(render_pass);
                    }
                }
            });

        painter.add(egui::PaintCallback {
            rect,
            callback: Arc::new(paint_callback),
        });
    }
}
