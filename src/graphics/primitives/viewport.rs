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

use crossbeam::atomic::AtomicCell;

#[derive(Debug)]
pub struct Viewport {
    data: AtomicCell<glam::Mat4>,
}

impl Viewport {
    pub fn new(proj: glam::Mat4) -> Self {
        Self {
            data: AtomicCell::new(proj),
        }
    }

    pub fn set_proj(&self, proj: glam::Mat4) {
        self.data.store(proj);
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<glam::Mat4>()] {
        bytemuck::cast(self.data.load())
    }
}
