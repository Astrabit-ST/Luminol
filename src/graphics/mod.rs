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
mod quad;
mod sprite;
mod tiles;
mod vertex;
mod viewport;

pub use atlas::Atlas;
pub use quad::Quad;
pub use sprite::Sprite;
pub use tiles::Tiles;
pub use vertex::Vertex;
pub use viewport::Viewport;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Hash)]
pub enum BlendMode {
    Normal = 0,
    Add = 1,
    Subtract = 2,
}

impl TryFrom<i32> for BlendMode {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => BlendMode::Normal,
            1 => BlendMode::Add,
            2 => BlendMode::Subtract,
            mode => return Err(format!("unexpected blend mode {mode}")),
        })
    }
}