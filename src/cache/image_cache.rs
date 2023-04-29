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

use std::ops::Deref;

use crate::prelude::*;
use glow::HasContext;

#[derive(Default)]
pub struct Cache {
    egui_imgs: dashmap::DashMap<String, RetainedImage>,
    glow_imgs: dashmap::DashMap<String, glow::Texture>,
}

impl Cache {
    pub fn load_egui_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<impl Deref<Target = RetainedImage> + '_, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();

        self.egui_imgs
            .entry(format!("{directory}/{filename}"))
            .or_try_insert_with(|| {
                let Some(f) = state!().filesystem.dir_children(directory)?.find(|entry| {
                    entry.as_ref().is_ok_and(|entry| entry.file_name() == filename)
                }) else {
                    return  Err("image not found".to_string());
                };

                egui_extras::RetainedImage::from_image_bytes(
                    format!("{directory}/{filename}"),
                    std::fs::read(f.expect("invalid dir entry despite checking").path())
                        .map_err(|e| e.to_string())?
                        .as_slice(),
                )
            })
    }

    pub fn load_glow_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<impl core::ops::Deref<Target = glow::Texture> + '_, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();

        self.glow_imgs
            .entry(format!("{directory}/{filename}"))
            .or_try_insert_with(|| {
                let Some(f) = state!().filesystem.dir_children(directory)?.find(|entry| {
                    entry.as_ref().is_ok_and(|entry| entry.file_name() == filename)
                }) else {
                    return  Err("image not found".to_string());
                };

                let image = image::open(f.expect("invalid dir entry despite checking").path())
                    .map_err(|e| e.to_string())?;
                let image = image.to_rgba8();
                // Check that the image will fit into the texture
                // If we dont perform this check, we may get a segfault (dont ask me how i know this)
                assert_eq!(image.len() as u32, image.width() * image.height() * 4);

                #[allow(unsafe_code)]
                unsafe {
                    let texture = state!().gl.create_texture()?;
                    state!().gl.bind_texture(glow::TEXTURE_2D, Some(texture));

                    state!().gl.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA as _,
                        image.width() as _,
                        image.height() as _,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        Some(image.as_raw()),
                    );
                    state!().gl.generate_mipmap(glow::TEXTURE_2D);

                    Ok(texture)
                }
            })
    }
}
