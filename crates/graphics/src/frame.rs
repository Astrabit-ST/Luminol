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

use crate::primitives::cells::Atlas;
use crate::{Drawable, GraphicsState, Renderable, Sprite, Viewport};
use luminol_data::OptionVec;

pub const FRAME_WIDTH: usize = 640;
pub const FRAME_HEIGHT: usize = 320;

pub struct Frame {
    pub atlas: Atlas,
    pub sprites: OptionVec<Sprite>,
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
            glam::vec2(FRAME_WIDTH as f32 * 32., FRAME_HEIGHT as f32 * 32.),
        );

        let frame = &animation.frames[frame_index];
        let sprites = frame.cell_data.as_slice()
            [0..frame.cell_data.xsize().min(frame.cell_max as usize)]
            .iter()
            .copied()
            .enumerate()
            .map(|(i, cell_id)| {
                (
                    i,
                    Sprite::basic_hue_quad(
                        graphics_state,
                        0,
                        atlas.calc_quad(cell_id),
                        &atlas.atlas_texture,
                        &viewport,
                    ),
                )
            })
            .collect();

        Self {
            atlas,
            sprites,
            viewport,
        }
    }

    /// Updates the sprite for one cell based on the given animation frame.
    pub fn update_cell(
        &mut self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
        cell_index: usize,
    ) {
        let cell_id = frame.cell_data[(cell_index, 0)];
        self.sprites.insert(
            cell_index,
            Sprite::basic_hue_quad(
                graphics_state,
                0,
                self.atlas.calc_quad(cell_id),
                &self.atlas.atlas_texture,
                &self.viewport,
            ),
        )
    }

    /// Updates the sprite for every cell based on the given animation frame.
    pub fn update_all_cells(
        &mut self,
        graphics_state: &GraphicsState,
        frame: &luminol_data::rpg::animation::Frame,
    ) {
        self.sprites.clear();
        self.sprites.extend(
            frame.cell_data.as_slice()[0..frame.cell_data.xsize().min(frame.cell_max as usize)]
                .iter()
                .copied()
                .enumerate()
                .map(|(i, cell_id)| {
                    (
                        i,
                        Sprite::basic_hue_quad(
                            graphics_state,
                            0,
                            self.atlas.calc_quad(cell_id),
                            &self.atlas.atlas_texture,
                            &self.viewport,
                        ),
                    )
                }),
        );
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
                .map(|(_, sprite)| sprite.prepare(graphics_state))
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
