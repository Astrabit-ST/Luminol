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

use color_eyre::eyre::Context;
use itertools::Itertools;

use crate::{
    Atlas, Collision, Drawable, Event, GraphicsState, Grid, Plane, Renderable, Tiles, Transform,
    Viewport,
};

pub struct Map {
    pub tiles: Tiles,
    pub panorama: Option<Plane>,
    pub fog: Option<Plane>,
    pub collision: Collision,
    pub grid: Grid,
    pub events: luminol_data::OptionVec<Event>,
    pub atlas: Atlas,

    pub viewport: Viewport,
    ani_time: Option<f64>,

    pub fog_enabled: bool,
    pub pano_enabled: bool,
    pub coll_enabled: bool,
    pub grid_enabled: bool,
    pub event_enabled: bool,
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

        let viewport = Viewport::new(
            graphics_state,
            glam::vec2(map.width as f32 * 32., map.height as f32 * 32.),
        );

        let tiles = Tiles::new(
            graphics_state,
            &map.data,
            &atlas,
            &viewport,
            Transform::unit(graphics_state),
        );
        let grid = Grid::new(
            graphics_state,
            &viewport,
            Transform::unit(graphics_state),
            map.data.xsize() as u32,
            map.data.ysize() as u32,
        );
        let collision = Collision::new(
            graphics_state,
            &viewport,
            Transform::unit(graphics_state),
            passages,
        );

        let panorama = if let Some(ref panorama_name) = tileset.panorama_name {
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Panoramas", panorama_name)
                .wrap_err_with(|| format!("Error loading map panorama {panorama_name:?}"))
                .unwrap_or_else(|e| {
                    graphics_state.send_texture_error(e);

                    graphics_state.texture_loader.placeholder_texture()
                });

            Some(Plane::new(
                graphics_state,
                &viewport,
                &texture,
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

                    graphics_state.texture_loader.placeholder_texture()
                });

            Some(Plane::new(
                graphics_state,
                &viewport,
                &texture,
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

        let events = map
            .events
            .iter()
            .map(|(id, event)| {
                Event::new_map(graphics_state, filesystem, &viewport, event, &atlas)
                    .map(|opt_e| opt_e.map(|e| (id, e)))
            })
            .flatten_ok()
            .try_collect()?;

        Ok(Self {
            tiles,
            panorama,
            fog,
            collision,
            grid,
            events,
            viewport,
            atlas,

            ani_time: None,

            fog_enabled: true,
            pano_enabled: true,
            coll_enabled: false,
            grid_enabled: true,
            event_enabled: true,
        })
    }

    pub fn set_tile(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        tile_id: i16,
        position: (usize, usize, usize),
    ) {
        self.tiles.set_tile(render_state, tile_id, position);
    }

    pub fn set_passage(
        &self,
        render_state: &luminol_egui_wgpu::RenderState,
        passage: i16,
        position: (usize, usize),
    ) {
        self.collision.set_passage(render_state, passage, position);
    }

    pub fn update_animation(&mut self, render_state: &luminol_egui_wgpu::RenderState, time: f64) {
        if let Some(ani_time) = self.ani_time {
            if time - ani_time >= 16. / 60. {
                self.ani_time = Some(time);
                self.tiles.autotiles.inc_ani_index(render_state);
            }
        } else {
            self.ani_time = Some(time);
        }
    }
}

pub struct Prepared {
    tiles: <Tiles as Renderable>::Prepared,
    panorama: Option<<Plane as Renderable>::Prepared>,
    fog: Option<<Plane as Renderable>::Prepared>,
    collision: Option<<Collision as Renderable>::Prepared>,
    grid: Option<<Grid as Renderable>::Prepared>,
    events: Vec<<Event as Renderable>::Prepared>,
}

impl Renderable for Map {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared {
        let tiles = self.tiles.prepare(graphics_state);
        let panorama = self
            .panorama
            .as_mut()
            .filter(|_| self.pano_enabled)
            .map(|pano| pano.prepare(graphics_state));
        let fog = self
            .fog
            .as_mut()
            .filter(|_| self.fog_enabled)
            .map(|fog| fog.prepare(graphics_state));
        let collision = self
            .coll_enabled
            .then(|| self.collision.prepare(graphics_state));
        let grid = self.grid_enabled.then(|| self.grid.prepare(graphics_state));
        let events = if self.event_enabled {
            self.events
                .iter_mut()
                .map(|(_, event)| event.prepare(graphics_state))
                .collect()
        } else {
            vec![]
        };

        Prepared {
            tiles,
            panorama,
            fog,
            collision,
            grid,
            events,
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        if let Some(ref pano) = self.panorama {
            pano.draw(render_pass);
        }

        self.tiles.draw(render_pass);

        for event in &self.events {
            event.draw(render_pass);
        }

        if let Some(ref fog) = self.fog {
            fog.draw(render_pass);
        }

        if let Some(ref collision) = self.collision {
            collision.draw(render_pass);
        }

        if let Some(ref grid) = self.grid {
            grid.draw(render_pass);
        }
    }
}
