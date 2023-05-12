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
mod atlas;
mod autotiles;

mod shader;
mod uniform;
mod vertices;

use crate::prelude::*;

pub const MAX_SIZE: u32 = 8192; // Max texture size in one dimension
pub const TILE_SIZE: u32 = 32; // Tiles are 32x32
pub const TILESET_WIDTH: u32 = TILE_SIZE * 8; // Tilesets are 8 tiles across

pub const AUTOTILE_HEIGHT: u32 = TILE_SIZE * 4; // Autotiles are 4 tiles high
pub const AUTOTILE_AMOUNT: u32 = 7; // There are 7 autotiles per tileset
pub const TOTAL_AUTOTILE_HEIGHT: u32 = AUTOTILE_HEIGHT * AUTOTILE_AMOUNT;
pub const UNDER_HEIGHT: u32 = MAX_SIZE - TOTAL_AUTOTILE_HEIGHT;

use super::quad::Quad;
use super::vertex::Vertex;

pub use atlas::Atlas;
use shader::Shader;
use uniform::Uniform;
use vertices::Vertices;

#[derive(Debug)]
pub struct Tiles {
    pub uniform: Uniform,
    pub atlas: Atlas,
    vertices: Vertices,
}

impl Tiles {
    pub fn new(tileset: &rpg::Tileset, map: &rpg::Map) -> Result<Self, String> {
        let atlas = Atlas::new(tileset)?;
        let uniform = Uniform::new(&atlas);
        let vertices = Vertices::new(map, &atlas);

        Ok(Self {
            uniform,
            atlas,
            vertices,
        })
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap tiles renderer");
        Shader::bind(render_pass);
        self.uniform.bind(render_pass);
        self.atlas.bind(render_pass);
        self.vertices.draw(render_pass);
        render_pass.pop_debug_group();
    }
}
