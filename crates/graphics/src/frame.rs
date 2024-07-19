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

use crate::primitives::cells::{Atlas, CELL_SIZE};
use crate::{Drawable, GraphicsState, Renderable, Sprite, Transform, Viewport};
use luminol_data::{BlendMode, OptionVec};

pub const FRAME_WIDTH: usize = 640;
pub const FRAME_HEIGHT: usize = 320;

const CELL_OFFSET: glam::Vec2 = glam::Vec2::splat(-(CELL_SIZE as f32) / 2.);

pub struct Frame {
    pub atlas: Atlas,
    pub cells: OptionVec<Cell>,
    pub onion_skin_cells: OptionVec<Cell>,
    pub viewport: Viewport,

    pub enable_onion_skin: bool,
}

pub struct Cell {
    pub sprite: Sprite,
    pub rect: egui::Rect,
}

impl Frame {
    pub fn new(
        graphics_state: &GraphicsState,
        atlas: Atlas,
        animation: &luminol_data::rpg::Animation,
        frame_index: usize,
    ) -> Self {
        let viewport = Viewport::new(
            graphics_state,
            glam::vec2(FRAME_WIDTH as f32, FRAME_HEIGHT as f32),
        );

        let mut frame = Self {
            atlas,
            cells: Default::default(),
            onion_skin_cells: Default::default(),
            viewport,
            enable_onion_skin: false,
        };
        frame.rebuild_all_cells(graphics_state, animation, frame_index);
        frame
    }

    pub fn rebuild_all_cells(
        &mut self,
        graphics_state: &GraphicsState,
        animation: &luminol_data::rpg::Animation,
        frame_index: usize,
    ) {
        let mut cells = std::mem::take(&mut self.cells);
        cells.clear();
        cells.extend(
            (0..cells
                .len()
                .max(animation.frames[frame_index].cell_data.xsize()))
                .filter_map(|i| {
                    self.cell_from_cell_data(
                        graphics_state,
                        &animation.frames[frame_index],
                        animation.animation_hue,
                        i,
                        1.,
                    )
                    .map(|cell| (i, cell))
                }),
        );
        self.cells = cells;

        let mut cells = std::mem::take(&mut self.onion_skin_cells);
        cells.clear();
        cells.extend(
            (0..cells.len().max(
                animation.frames[frame_index.saturating_sub(1)]
                    .cell_data
                    .xsize(),
            ))
                .filter_map(|i| {
                    self.cell_from_cell_data(
                        graphics_state,
                        &animation.frames[frame_index.saturating_sub(1)],
                        animation.animation_hue,
                        i,
                        0.5,
                    )
                    .map(|cell| (i, cell))
                }),
        );
        self.cells = cells;
    }

    pub fn update_cell(
        &mut self,
        graphics_state: &GraphicsState,
        animation: &luminol_data::rpg::Animation,
        frame_index: usize,
        cell_index: usize,
    ) {
        let cells = std::mem::take(&mut self.cells);
        self.cells = self.update_cell_inner(
            cells,
            graphics_state,
            &animation.frames[frame_index],
            animation.animation_hue,
            cell_index,
            1.,
        );

        let cells = std::mem::take(&mut self.onion_skin_cells);
        self.onion_skin_cells = self.update_cell_inner(
            cells,
            graphics_state,
            &animation.frames[frame_index.saturating_sub(1)],
            animation.animation_hue,
            cell_index,
            0.5,
        );
    }

    pub fn update_all_cells(
        &mut self,
        graphics_state: &GraphicsState,
        animation: &luminol_data::rpg::Animation,
        frame_index: usize,
    ) {
        for cell_index in 0..self
            .cells
            .len()
            .max(animation.frames[frame_index].cell_data.xsize())
        {
            let cells = std::mem::take(&mut self.cells);
            self.cells = self.update_cell_inner(
                cells,
                graphics_state,
                &animation.frames[frame_index],
                animation.animation_hue,
                cell_index,
                1.,
            );
        }

        for cell_index in 0..self.onion_skin_cells.len().max(
            animation.frames[frame_index.saturating_sub(1)]
                .cell_data
                .xsize(),
        ) {
            let cells = std::mem::take(&mut self.onion_skin_cells);
            self.onion_skin_cells = self.update_cell_inner(
                cells,
                graphics_state,
                &animation.frames[frame_index.saturating_sub(1)],
                animation.animation_hue,
                cell_index,
                0.5,
            );
        }
    }

    fn cell_from_cell_data(
        &self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
        hue: i32,
        cell_index: usize,
        opacity_multiplier: f32,
    ) -> Option<Cell> {
        (cell_index < frame.cell_data.xsize() && frame.cell_data[(cell_index, 0)] >= 0).then(|| {
            let id = frame.cell_data[(cell_index, 0)];
            let offset_x = frame.cell_data[(cell_index, 1)] as f32;
            let offset_y = frame.cell_data[(cell_index, 2)] as f32;
            let scale = frame.cell_data[(cell_index, 3)] as f32 / 100.;
            let rotation = -(frame.cell_data[(cell_index, 4)] as f32).to_radians();
            let flip = frame.cell_data[(cell_index, 5)] == 1;
            let opacity = frame.cell_data[(cell_index, 6)] as i32;
            let blend_mode = match frame.cell_data[(cell_index, 7)] {
                1 => BlendMode::Add,
                2 => BlendMode::Subtract,
                _ => BlendMode::Normal,
            };

            let flip_vec = glam::vec2(if flip { -1. } else { 1. }, 1.);
            let glam::Vec2 { x: cos, y: sin } = glam::Vec2::from_angle(rotation);

            Cell {
                sprite: Sprite::new_with_rotation(
                    graphics_state,
                    self.atlas.calc_quad(id),
                    hue,
                    opacity,
                    opacity_multiplier,
                    blend_mode,
                    &self.atlas.atlas_texture,
                    &self.viewport,
                    Transform::new(
                        graphics_state,
                        glam::vec2(offset_x, offset_y)
                            + glam::Mat2::from_cols_array(&[cos, sin, -sin, cos])
                                * (scale * flip_vec * CELL_OFFSET),
                        scale * flip_vec,
                    ),
                    if flip { -rotation } else { rotation },
                ),

                rect: egui::Rect::from_center_size(
                    egui::pos2(offset_x, offset_y),
                    egui::Vec2::splat(CELL_SIZE as f32 * (cos.abs() + sin.abs()) * scale),
                ),
            }
        })
    }

    fn update_cell_inner(
        &self,
        mut cells: OptionVec<Cell>,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
        hue: i32,
        cell_index: usize,
        opacity_multiplier: f32,
    ) -> OptionVec<Cell> {
        if cell_index < frame.cell_data.xsize() && frame.cell_data[(cell_index, 0)] >= 0 {
            if let Some(cell) = cells.get_mut(cell_index) {
                let id = frame.cell_data[(cell_index, 0)];
                let offset_x = frame.cell_data[(cell_index, 1)] as f32;
                let offset_y = frame.cell_data[(cell_index, 2)] as f32;
                let scale = frame.cell_data[(cell_index, 3)] as f32 / 100.;
                let rotation = -(frame.cell_data[(cell_index, 4)] as f32).to_radians();
                let flip = frame.cell_data[(cell_index, 5)] == 1;
                let opacity = frame.cell_data[(cell_index, 6)] as i32;
                let blend_mode = match frame.cell_data[(cell_index, 7)] {
                    1 => BlendMode::Add,
                    2 => BlendMode::Subtract,
                    _ => BlendMode::Normal,
                };

                let flip_vec = glam::vec2(if flip { -1. } else { 1. }, 1.);
                let glam::Vec2 { x: cos, y: sin } = glam::Vec2::from_angle(rotation);

                cell.sprite.transform.set(
                    &graphics_state.render_state,
                    glam::vec2(offset_x, offset_y)
                        + glam::Mat2::from_cols_array(&[cos, sin, -sin, cos])
                            * (scale * flip_vec * CELL_OFFSET),
                    scale * flip_vec,
                );

                cell.sprite.graphic.set(
                    &graphics_state.render_state,
                    hue,
                    opacity,
                    opacity_multiplier,
                    if flip { -rotation } else { rotation },
                );

                cell.sprite.set_quad(
                    &graphics_state.render_state,
                    self.atlas.calc_quad(id),
                    self.atlas.atlas_texture.size(),
                );

                cell.sprite.blend_mode = blend_mode;

                cell.rect = egui::Rect::from_center_size(
                    egui::pos2(offset_x, offset_y),
                    egui::Vec2::splat(CELL_SIZE as f32 * (cos.abs() + sin.abs()) * scale),
                );
            } else if let Some(cell) =
                self.cell_from_cell_data(graphics_state, frame, hue, cell_index, opacity_multiplier)
            {
                cells.insert(cell_index, cell);
            }
        } else {
            let _ = cells.try_remove(cell_index);
        }

        cells
    }
}

pub struct Prepared {
    cells: Vec<<Sprite as Renderable>::Prepared>,
    onion_skin_cells: Vec<<Sprite as Renderable>::Prepared>,
}

impl Renderable for Frame {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared {
        Self::Prepared {
            cells: self
                .cells
                .iter_mut()
                .map(|(_, cell)| cell.sprite.prepare(graphics_state))
                .collect(),

            onion_skin_cells: if self.enable_onion_skin {
                self.onion_skin_cells
                    .iter_mut()
                    .map(|(_, cell)| cell.sprite.prepare(graphics_state))
                    .collect()
            } else {
                Default::default()
            },
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        for sprite in &self.onion_skin_cells {
            sprite.draw(render_pass);
        }
        for sprite in &self.cells {
            sprite.draw(render_pass);
        }
    }
}
