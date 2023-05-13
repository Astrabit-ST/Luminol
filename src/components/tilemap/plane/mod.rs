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
use super::quad::Quad;
use super::sprite::{BlendMode, Sprite};

use crate::prelude::*;

#[derive(Debug)]
pub struct Plane {
    sprite: Sprite,
}

impl Plane {
    pub fn new(
        texture: Arc<image_cache::WgpuTexture>,
        hue: i32,
        zoom: i32,
        blend_mode: BlendMode,
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
            0.0,
        );

        let sprite = Sprite::new(&[quad], texture, blend_mode, hue, opacity);

        Self { sprite }
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.sprite.draw(render_pass);
    }
}
