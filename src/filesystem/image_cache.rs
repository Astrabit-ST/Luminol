// Copyright (C) 2022 Lily Lyons
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

use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use egui_extras::RetainedImage;

use super::Filesystem;

#[derive(Default)]
pub struct ImageCache {
    images: RefCell<HashMap<String, RetainedImage>>,
}

impl ImageCache {
    pub fn load_image(&self, path: String, filesystem: &Filesystem) -> RefMut<'_, RetainedImage> {
        RefMut::map(self.images.borrow_mut(), |images| {
            images.entry(path.clone()).or_insert_with(|| {
                egui_extras::RetainedImage::from_image_bytes(
                    "",
                    &filesystem
                        .read_bytes(&format!("{}.png", path))
                        .unwrap_or_else(|_| {
                            filesystem
                                .read_bytes(&format!("{}.jpg", path))
                                .unwrap_or_else(|_| {
                                    filesystem
                                        .read_bytes(&format!("{}.jpg", path))
                                        .expect("Failed to read image from path")
                                })
                        }),
                )
                .expect("Failed to load image")
            })
        })
    }
}
