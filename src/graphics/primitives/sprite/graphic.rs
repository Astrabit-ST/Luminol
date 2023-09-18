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
pub struct Graphic {
    data: AtomicCell<Data>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Data {
    hue: f32,
    opacity: f32,
    opacity_multiplier: f32,
}

impl Graphic {
    pub fn new(hue: i32, opacity: i32) -> Self {
        let hue = (hue % 360) as f32 / 360.0;
        let opacity = opacity as f32 / 255.;
        let data = Data {
            hue,
            opacity,
            opacity_multiplier: 1.,
        };

        Self {
            data: AtomicCell::new(data),
        }
    }

    pub fn hue(&self) -> i32 {
        (self.data.load().hue * 360.) as i32
    }

    pub fn set_hue(&self, hue: i32) {
        let hue = (hue % 360) as f32 / 360.0;
        let data = self.data.load();

        self.data.store(Data { hue, ..data });
    }

    pub fn opacity(&self) -> i32 {
        (self.data.load().opacity * 255.) as i32
    }

    pub fn set_opacity(&self, opacity: i32) {
        let opacity = opacity as f32 / 255.0;
        let data = self.data.load();

        self.data.store(Data { opacity, ..data });
    }

    pub fn opacity_multiplier(&self) -> f32 {
        self.data.load().opacity_multiplier
    }

    pub fn set_opacity_multiplier(&self, opacity_multiplier: f32) {
        let data = self.data.load();

        self.data.store(Data {
            opacity_multiplier,
            ..data
        });
    }

    pub fn as_bytes(&self) -> [u8; std::mem::size_of::<Data>()] {
        bytemuck::cast(self.data.load())
    }
}
