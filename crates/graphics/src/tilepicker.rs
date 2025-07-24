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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use std::sync::Arc;

use itertools::Itertools;

use crate::{
    Atlas, Collision, Drawable, GraphicsState, Grid, Renderable, Tiles, Transform, Viewport,
};

pub struct Tilepicker {
    pub coll_enabled: bool,
    pub grid_enabled: bool,

    pub tiles: Tiles,
    pub collision: Collision,
    pub grid: Grid,
    pub atlas: Atlas,

    pub viewport: Viewport,
    ani_time: Option<f64>,
}

fn compute_tilepicker_data(tileset_height: u32, exclude_autotiles: bool) -> luminol_data::Table3 {
    let tilepicker_data = if exclude_autotiles {
        (384..(tileset_height as i16 / 32 * 8 + 384)).collect_vec()
    } else {
        (47..(384 + 47))
            .step_by(48)
            .chain(384..(tileset_height as i16 / 32 * 8 + 384))
            .collect_vec()
    };

    luminol_data::Table3::new_data(
        8,
        !exclude_autotiles as usize + (tileset_height / 32) as usize,
        1,
        tilepicker_data,
    )
}

fn compute_passages(
    passages: &luminol_data::Table1,
    tilepicker_data: &luminol_data::Table3,
) -> luminol_data::Table2 {
    let mut computed_passages =
        luminol_data::Table2::new(tilepicker_data.xsize(), tilepicker_data.ysize());

    for x in 0..8 {
        computed_passages[(x, 0)] = {
            let tile_id = tilepicker_data[(x, 0, 0)].try_into().unwrap_or_default();
            if tile_id >= passages.len() {
                0
            } else {
                passages[tile_id]
            }
        };
    }

    let length =
        (computed_passages.len().saturating_sub(8)).min(passages.len().saturating_sub(384));

    computed_passages.as_mut_slice()[8..8 + length]
        .copy_from_slice(&passages.as_slice()[384..384 + length]);

    computed_passages
}

impl Tilepicker {
    pub fn new(
        graphics_state: &GraphicsState,
        tileset_name: Option<&camino::Utf8Path>,
        autotile_names: &[luminol_data::RpgOption<camino::Utf8PathBuf>],
        passages: &luminol_data::Table1,
        filesystem: &impl luminol_filesystem::FileSystem,
        exclude_autotiles: bool,
    ) -> Self {
        let atlas = graphics_state.atlas_loader.load_atlas(
            graphics_state,
            filesystem,
            tileset_name,
            autotile_names,
        );

        let tilepicker_data = compute_tilepicker_data(atlas.tileset_height(), exclude_autotiles);

        let viewport = Viewport::new(
            graphics_state,
            glam::vec2(256., atlas.tileset_height() as f32 + 32.),
        );

        let tiles = Tiles::new(
            graphics_state,
            &tilepicker_data,
            &atlas,
            &viewport,
            Transform::unit(graphics_state),
        );

        let grid = Grid::new(
            graphics_state,
            &viewport,
            Transform::unit(graphics_state),
            tilepicker_data.xsize() as u32,
            tilepicker_data.ysize() as u32,
        );

        let collision = Collision::new(
            graphics_state,
            &viewport,
            Transform::unit(graphics_state),
            &compute_passages(passages, &tilepicker_data),
        );

        Self {
            tiles,
            collision,
            grid,
            atlas,

            viewport,

            coll_enabled: false,
            grid_enabled: true,
            ani_time: None,
        }
    }

    pub fn update_collision(
        &mut self,
        render_state: &egui_wgpu::RenderState,
        passages: &luminol_data::Table1,
        exclude_autotiles: bool,
    ) {
        self.collision.set_passages(
            render_state,
            &compute_passages(
                passages,
                &compute_tilepicker_data(self.atlas.tileset_height(), exclude_autotiles),
            ),
        )
    }

    pub fn update_animation(&mut self, render_state: &egui_wgpu::RenderState, time: f64) {
        if let Some(ani_time) = self.ani_time {
            if time - ani_time >= 16. / 60. - ani_time.rem_euclid(16. / 60.) {
                self.ani_time = Some(time);
                self.tiles.autotiles.inc_ani_index(render_state);
            }
        } else {
            self.ani_time = Some(time);
        }
    }

    pub fn set_position(&mut self, render_state: &egui_wgpu::RenderState, position: glam::Vec2) {
        self.tiles.transform.set_position(render_state, position);
        self.collision
            .transform
            .set_position(render_state, position);
        self.grid.transform.set_position(render_state, position);
    }
}

pub struct Prepared {
    tiles: <Tiles as Renderable>::Prepared,
    collision: <Collision as Renderable>::Prepared,
    grid: <Grid as Renderable>::Prepared,

    coll_enabled: bool,
    grid_enabled: bool,
}

impl Renderable for Tilepicker {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &Arc<GraphicsState>) -> Self::Prepared {
        Prepared {
            tiles: self.tiles.prepare(graphics_state),
            collision: self.collision.prepare(graphics_state),
            grid: self.grid.prepare(graphics_state),

            coll_enabled: self.coll_enabled,
            grid_enabled: self.grid_enabled,
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.tiles.draw(render_pass);

        if self.coll_enabled {
            self.collision.draw(render_pass);
        }

        if self.grid_enabled {
            self.grid.draw(render_pass);
        }
    }
}
