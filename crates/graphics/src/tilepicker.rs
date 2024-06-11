// Copyright (C) 2024 Lily Lyons
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

use std::{sync::Arc, time::Duration};

use fragile::Fragile;
use itertools::Itertools;

use crate::{Atlas, Collision, GraphicsState, Grid, Tiles, Viewport};

pub struct Tilepicker {
    pub coll_enabled: bool,
    pub grid_enabled: bool,

    resources: Arc<Resources>,
    viewport: Arc<Viewport>,
    ani_time: Option<f64>,
}

struct Resources {
    tiles: Tiles,
    collision: Collision,
    grid: Grid,
}

struct Callback {
    resources: Fragile<Arc<Resources>>,
    graphics_state: Fragile<Arc<GraphicsState>>,

    coll_enabled: bool,
    grid_enabled: bool,
}

impl luminol_egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        let resources = self.resources.get();
        let graphics_state = self.graphics_state.get();

        resources
            .tiles
            .draw(graphics_state, &[true], None, render_pass);

        if self.coll_enabled {
            resources.collision.draw(graphics_state, render_pass);
        }

        if self.grid_enabled {
            resources.grid.draw(graphics_state, &info, render_pass);
        }
    }
}

impl Tilepicker {
    pub fn new(
        graphics_state: &GraphicsState,
        tileset: &luminol_data::rpg::Tileset,
        filesystem: &impl luminol_filesystem::FileSystem,
    ) -> color_eyre::Result<Self> {
        let atlas = graphics_state
            .atlas_loader
            .load_atlas(graphics_state, filesystem, tileset)?;

        let tilepicker_data = (47..(384 + 47))
            .step_by(48)
            .chain(384..(atlas.tileset_height as i16 / 32 * 8 + 384))
            .collect_vec();
        let tilepicker_data = luminol_data::Table3::new_data(
            8,
            1 + (atlas.tileset_height / 32) as usize,
            1,
            tilepicker_data,
        );

        let viewport = Arc::new(Viewport::new(
            graphics_state,
            256.,
            atlas.tileset_height as f32 + 32.,
        ));

        let tiles = Tiles::new(graphics_state, viewport.clone(), atlas, &tilepicker_data);

        let grid = Grid::new(
            graphics_state,
            viewport.clone(),
            tilepicker_data.xsize() as u32,
            tilepicker_data.ysize() as u32,
        );

        let mut passages =
            luminol_data::Table2::new(tilepicker_data.xsize(), tilepicker_data.ysize());
        for x in 0..8 {
            passages[(x, 0)] = {
                let tile_id = tilepicker_data[(x, 0, 0)].try_into().unwrap_or_default();
                if tile_id >= tileset.passages.len() {
                    0
                } else {
                    tileset.passages[tile_id]
                }
            };
        }
        let length =
            (passages.len().saturating_sub(8)).min(tileset.passages.len().saturating_sub(384));
        passages.as_mut_slice()[8..8 + length]
            .copy_from_slice(&tileset.passages.as_slice()[384..384 + length]);
        let collision = Collision::new(graphics_state, viewport.clone(), &passages);

        Ok(Self {
            resources: Arc::new(Resources {
                tiles,
                collision,
                grid,
            }),
            viewport,
            coll_enabled: false,
            grid_enabled: false,
            ani_time: None,
        })
    }

    pub fn paint(
        &mut self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
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
                resources: Fragile::new(self.resources.clone()),
                graphics_state: Fragile::new(graphics_state),

                coll_enabled: self.coll_enabled,
                grid_enabled: self.grid_enabled,
            },
        ));
    }

    pub fn set_proj(&self, render_state: &luminol_egui_wgpu::RenderState, proj: glam::Mat4) {
        self.viewport.set_proj(render_state, proj);
    }

    pub fn atlas(&self) -> &Atlas {
        &self.resources.tiles.atlas
    }
}
