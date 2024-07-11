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

#[derive(Debug, Clone, Copy)]
pub struct Instances {
    map_size: u32,
}

impl Instances {
    pub fn new(map_width: u32, map_height: u32) -> Self {
        Self {
            map_size: map_width * map_height,
        }
    }

    pub fn draw(self, render_pass: &mut wgpu::RenderPass<'_>) {
        render_pass.draw(0..6, 0..self.map_size);
    }
}
