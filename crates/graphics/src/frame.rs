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

use crate::primitives::cells::{Atlas, CELL_SIZE};
use crate::{Drawable, GraphicsState, Renderable, Sprite, Transform, Viewport};
use luminol_data::{BlendMode, OptionVec};

pub const FRAME_WIDTH: usize = 640;
pub const FRAME_HEIGHT: usize = 320;

const CELL_OFFSET: glam::Vec2 = glam::Vec2::splat(-(CELL_SIZE as f32) / 2.);

pub struct Frame {
    pub atlas: Atlas,
    pub sprites: OptionVec<(Sprite, egui::Rect)>,
    pub viewport: Viewport,
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
            sprites: Default::default(),
            viewport,
        };
        frame.update_all_cells(graphics_state, &animation.frames[frame_index]);
        frame
    }

    /// Updates the sprite for one cell based on the given animation frame.
    pub fn update_cell(
        &mut self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
        cell_index: usize,
    ) {
        if let Some(sprite) = self.sprite_from_cell_data(graphics_state, frame, cell_index) {
            self.sprites.insert(cell_index, sprite);
        } else {
            let _ = self.sprites.try_remove(cell_index);
        }
    }

    /// Updates the sprite for every cell based on the given animation frame.
    pub fn update_all_cells(
        &mut self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
    ) {
        let mut sprites = std::mem::take(&mut self.sprites);
        sprites.clear();
        sprites.extend((0..frame.cell_data.xsize()).filter_map(|i| {
            self.sprite_from_cell_data(graphics_state, frame, i)
                .map(|s| (i, s))
        }));
        self.sprites = sprites;
    }

    pub fn sprite_from_cell_data(
        &self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
        cell_index: usize,
    ) -> Option<(Sprite, egui::Rect)> {
        (cell_index < frame.cell_data.xsize() && frame.cell_data[(cell_index, 0)] >= 0).then(|| {
            let id = frame.cell_data[(cell_index, 0)];
            let offset_x = frame.cell_data[(cell_index, 1)] as f32;
            let offset_y = frame.cell_data[(cell_index, 2)] as f32;
            let scale = frame.cell_data[(cell_index, 3)] as f32 / 100.;
            let rotation = -(frame.cell_data[(cell_index, 4)] as f32).to_radians();
            let flip = glam::vec2(
                if frame.cell_data[(cell_index, 5)] == 1 {
                    -1.
                } else {
                    1.
                },
                1.,
            );
            let opacity = frame.cell_data[(cell_index, 6)] as i32;
            let blend_mode = match frame.cell_data[(cell_index, 7)] {
                1 => BlendMode::Add,
                2 => BlendMode::Subtract,
                _ => BlendMode::Normal,
            };
            let glam::Vec2 { x: cos, y: sin } = glam::Vec2::from_angle(rotation);
            (
                Sprite::new_with_rotation(
                    graphics_state,
                    self.atlas.calc_quad(id),
                    0,
                    opacity,
                    blend_mode,
                    &self.atlas.atlas_texture,
                    &self.viewport,
                    Transform::new(
                        graphics_state,
                        glam::vec2(offset_x, offset_y)
                            + glam::Mat2::from_cols_array(&[cos, sin, -sin, cos])
                                * (scale * flip * CELL_OFFSET),
                        scale * flip,
                    ),
                    rotation,
                ),
                egui::Rect::from_center_size(
                    egui::pos2(offset_x, offset_y),
                    egui::Vec2::splat(CELL_SIZE as f32 * (cos.abs() + sin.abs()) * scale),
                ),
            )
        })
    }
}

pub struct Prepared {
    sprites: Vec<<Sprite as Renderable>::Prepared>,
}

impl Renderable for Frame {
    type Prepared = Prepared;

    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared {
        Self::Prepared {
            sprites: self
                .sprites
                .iter_mut()
                .map(|(_, (sprite, _))| sprite.prepare(graphics_state))
                .collect(),
        }
    }
}

impl Drawable for Prepared {
    fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        for sprite in &self.sprites {
            sprite.draw(render_pass);
        }
    }
}
