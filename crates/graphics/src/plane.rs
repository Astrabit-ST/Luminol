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

use crate::{GraphicsState, Quad, Renderable, Sprite, Texture, Transform, Viewport};

pub struct Plane {
    pub sprite: Sprite,
}

impl Plane {
    // FIXME lots of arguments
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        graphics_state: &GraphicsState,
        viewport: &Viewport,
        texture: &Texture,
        hue: i32,
        zoom: i32,
        blend_mode: luminol_data::BlendMode,
        opacity: i32,
        map_width: usize,
        map_height: usize,
    ) -> Self {
        let zoom = zoom as f32 / 100.;
        let map_width = map_width as f32 * 32.;
        let map_height = map_height as f32 * 32.;

        let tex_coords = egui::Rect::from_min_size(
            egui::pos2(0.0, 0.0),
            egui::vec2(map_width / zoom, map_height / zoom),
        );

        let quad = Quad::new(
            egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(map_width, map_height)),
            tex_coords,
        );

        let sprite = Sprite::new(
            graphics_state,
            quad,
            hue,
            opacity,
            blend_mode,
            texture,
            viewport,
            Transform::unit(graphics_state),
        );

        Self { sprite }
    }
}

impl Renderable for Plane {
    type Prepared = <Sprite as Renderable>::Prepared;

    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared {
        self.sprite.prepare(graphics_state)
    }
}
