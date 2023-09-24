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
pub struct Autotiles {
    data: AtomicCell<Data>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    ani_index: u32,
    autotile_region_width: u32,
    autotile_frames: [u32; 7],
}

impl Autotiles {
    pub fn new(atlas: &super::Atlas) -> Self {
        let autotiles = Data {
            autotile_frames: atlas.autotile_frames,
            autotile_region_width: atlas.autotile_width,
            ani_index: 0,
        };

        Autotiles {
            data: AtomicCell::new(autotiles),
        }
    }

    pub fn inc_ani_index(&self) {
        let data = self.data.load();
        self.data.store(Data {
            ani_index: data.ani_index.wrapping_add(1),
            ..data
        });
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
        bytemuck::cast(self.data.load())
    }
}
