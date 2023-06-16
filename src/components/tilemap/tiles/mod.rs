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
mod autotile_ids;

mod autotiles;
mod instance;
mod shader;

use crate::prelude::*;

pub const MAX_SIZE: u32 = 8192; // Max texture size in one dimension
pub const TILE_SIZE: u32 = 32; // Tiles are 32x32
pub const TILESET_COLUMNS: u32 = 8; // Tilesets are 8 tiles across
pub const TILESET_WIDTH: u32 = TILE_SIZE * TILESET_COLUMNS; // self explanatory

pub const AUTOTILE_ID_AMOUNT: u32 = 48; // there are 48 tile ids per autotile
pub const AUTOTILE_FRAME_COLS: u32 = TILESET_COLUMNS; // this is how many "columns" of autotiles there are per frame
pub const AUTOTILE_AMOUNT: u32 = 7; // There are 7 autotiles per tileset

pub const AUTOTILE_ROWS: u32 = AUTOTILE_ID_AMOUNT / AUTOTILE_FRAME_COLS; // split up the 48 tiles across each tileset row
pub const AUTOTILE_ROW_HEIGHT: u32 = AUTOTILE_ROWS * TILE_SIZE; // This is how high one row of autotiles is
pub const TOTAL_AUTOTILE_HEIGHT: u32 = AUTOTILE_ROW_HEIGHT * AUTOTILE_AMOUNT; // self explanatory
pub const HEIGHT_UNDER_AUTOTILES: u32 = MAX_SIZE - TOTAL_AUTOTILE_HEIGHT; // this is the height under autotiles

pub const AUTOTILE_FRAME_WIDTH: u32 = AUTOTILE_FRAME_COLS * TILE_SIZE; // This is per frame!

use super::quad::Quad;
use super::vertex::Vertex;

pub use atlas::Atlas;
use autotiles::Autotiles;
use instance::Instances;
use shader::Shader;

#[derive(Debug)]
pub struct Tiles {
    pub autotiles: Autotiles,
    pub atlas: Atlas,
    pub instances: Instances,
}

impl Tiles {
    pub fn new(tileset: &rpg::Tileset, map: &rpg::Map) -> Result<Self, String> {
        let atlas = Atlas::new(tileset)?;
        let autotiles = Autotiles::new(&atlas);
        let instances = Instances::new(map, atlas.atlas_texture.size());

        Ok(Self {
            autotiles,
            atlas,
            instances,
        })
    }

    pub fn draw<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu::RenderPass<'rpass>,
        enabled_layers: &[bool],
    ) {
        render_pass.push_debug_group("tilemap tiles renderer");
        Shader::bind(render_pass);
        self.autotiles.bind(render_pass);
        self.atlas.bind(render_pass);
        self.instances.draw(render_pass);
        render_pass.pop_debug_group();
    }
}
