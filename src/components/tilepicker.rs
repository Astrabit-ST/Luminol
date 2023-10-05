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

use slab::Slab;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct Tilepicker {
    pub selected_tiles_left: i16,
    pub selected_tiles_top: i16,
    pub selected_tiles_right: i16,
    pub selected_tiles_bottom: i16,

    drag_origin: Option<egui::Pos2>,

    resources: Arc<Resources>,
    ani_time: Option<f64>,
}

#[derive(Debug)]
struct Resources {
    tiles: primitives::Tiles,
    viewport: primitives::Viewport,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SelectedTile {
    Autotile(i16),
    Tile(i16),
}

impl SelectedTile {
    pub fn from_id(id: i16) -> Self {
        if id < 384 {
            SelectedTile::Autotile(id / 48)
        } else {
            SelectedTile::Tile(id)
        }
    }

    pub fn to_id(&self) -> i16 {
        match *self {
            Self::Autotile(tile) => tile * 48,
            Self::Tile(tile) => tile,
        }
    }
}

impl Default for SelectedTile {
    fn default() -> Self {
        SelectedTile::Autotile(0)
    }
}

type ResourcesSlab = Slab<Arc<Resources>>;

impl Tilepicker {
    pub fn new(tileset: &rpg::Tileset) -> Result<Tilepicker, String> {
        let atlas = state!().atlas_cache.load_atlas(tileset)?;

        let tilepicker_data = (47..(384 + 47))
            .step_by(48)
            .chain(384..(atlas.tileset_height as i16 / 32 * 8 + 384))
            .collect_vec();
        let tilepicker_data = Table3::new_data(
            8,
            1 + (atlas.tileset_height / 32) as usize,
            1,
            tilepicker_data,
        );

        let viewport = primitives::Viewport::new(
            glam::Mat4::orthographic_rh(
                0.0,
                256.,
                atlas.tileset_height as f32 + 32.,
                0.0,
                -1.0,
                1.0,
            ),
            crate::USE_PUSH_CONSTANTS,
        );

        let tiles = primitives::Tiles::new(atlas, &tilepicker_data, crate::USE_PUSH_CONSTANTS);

        Ok(Self {
            resources: Arc::new(Resources { tiles, viewport }),
            ani_time: None,
            selected_tiles_left: 0,
            selected_tiles_top: 0,
            selected_tiles_right: 0,
            selected_tiles_bottom: 0,
            drag_origin: None,
        })
    }

    pub fn get_tile_from_offset(&self, x: i16, y: i16) -> SelectedTile {
        let width = self.selected_tiles_right - self.selected_tiles_left + 1;
        let height = self.selected_tiles_bottom - self.selected_tiles_top + 1;
        let x = self.selected_tiles_left + x.rem_euclid(width);
        let y = self.selected_tiles_top + y.rem_euclid(height);
        match y {
            ..=0 => SelectedTile::Autotile(x),
            _ => SelectedTile::Tile(x + (y - 1) * 8 + 384),
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let time = ui.ctx().input(|i| i.time);
        if let Some(ani_time) = self.ani_time {
            if time - ani_time >= 16. / 60. {
                self.ani_time = Some(time);
                self.resources.tiles.autotiles.inc_ani_index();
            }
        } else {
            self.ani_time = Some(time);
        }

        ui.ctx().request_repaint_after(Duration::from_millis(16));

        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(256., self.resources.tiles.atlas.tileset_height as f32 + 32.),
            egui::Sense::click_and_drag(),
        );

        let resources = self.resources.clone();
        let prepare_id = Arc::new(OnceCell::new());
        let paint_id = prepare_id.clone();

        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |_, _, _encoder, paint_callback_resources| {
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
                            tiles, viewport, ..
                        } = resources.as_ref();

                        viewport.bind(render_pass);
                        tiles.draw(viewport, &[true], None, render_pass);
                    }),
            ),
        });

        let rect = egui::Rect::from_x_y_ranges(
            (self.selected_tiles_left * 32) as f32..=((self.selected_tiles_right + 1) * 32) as f32,
            (self.selected_tiles_top * 32) as f32..=((self.selected_tiles_bottom + 1) * 32) as f32,
        )
        .translate(canvas_rect.min.to_vec2());
        ui.painter()
            .rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

        let Some(pos) = response.interact_pointer_pos() else {
            return response;
        };
        let pos = ((pos - canvas_rect.min) / 32.).to_pos2();

        if response.dragged_by(egui::PointerButton::Primary) {
            let drag_origin = if let Some(drag_origin) = self.drag_origin {
                drag_origin
            } else {
                self.drag_origin = Some(pos);
                pos
            };
            let rect = egui::Rect::from_two_pos(drag_origin, pos);
            self.selected_tiles_left = (rect.left() as i16).max(0);
            self.selected_tiles_right = (rect.right() as i16).min(7);
            self.selected_tiles_top = (rect.top() as i16).max(0);
            self.selected_tiles_bottom =
                (rect.bottom() as i16).min(self.resources.tiles.atlas.tileset_height as i16 / 32);
        } else {
            self.drag_origin = None;
        }

        response
    }
}
