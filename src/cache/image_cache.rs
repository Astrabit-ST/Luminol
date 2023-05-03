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

use crate::prelude::*;
use glow::HasContext;

#[derive(Default)]
pub struct Cache {
    // FIXME: This may not handle reloading textures properly.
    egui_imgs: dashmap::DashMap<String, Arc<RetainedImage>>,
    glow_imgs: dashmap::DashMap<String, Arc<GlTexture>>,
}

#[derive(Debug)]
pub struct GlTexture {
    raw: glow::Texture,
    width: u32,
    height: u32,
}

impl GlTexture {
    /// # Safety
    /// Do not free the returned texture using glow::Context::delete_texture.
    #[allow(unsafe_code)]
    pub unsafe fn raw(&self) -> glow::Texture {
        self.raw
    }

    /// # Safety
    /// Texture must be valid. Dimensions must be correct.
    #[allow(unsafe_code)]
    pub unsafe fn new(raw: glow::Texture, width: u32, height: u32) -> Self {
        Self { raw, width, height }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn size_vec2(&self) -> egui::Vec2 {
        egui::vec2(self.width as _, self.height as _)
    }
}

impl Drop for GlTexture {
    fn drop(&mut self) {
        // Delete the texture on drop.
        // This assumes that the texture is valid.
        #[allow(unsafe_code)]
        unsafe {
            state!().gl.delete_texture(self.raw)
        }
    }
}

impl Cache {
    pub fn load_egui_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<Arc<RetainedImage>, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();

        let entry = self
            .egui_imgs
            .entry(format!("{directory}/{filename}"))
            .or_try_insert_with(|| -> Result<_, String> {
                let image = self.load_image(directory, filename)?.into_rgba8();
                let image = egui_extras::RetainedImage::from_color_image(
                    format!("{directory}/{filename}"),
                    egui::ColorImage::from_rgba_unmultiplied(
                        [image.width() as usize, image.height() as usize],
                        &image,
                    ),
                )
                .with_options(egui::TextureOptions::NEAREST);
                Ok(Arc::new(image))
            })?;
        Ok(Arc::clone(&entry))
    }

    pub fn load_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<image::DynamicImage, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();
        let Some(f) = state!().filesystem.dir_children(directory)?.map(Result::unwrap).find(|entry| {
                entry.path().file_stem() == Some(std::ffi::OsStr::new(filename))
            }) else {
                return Err(format!("{filename} not found in {directory}"));
            };

        let image = image::open(f.path()).map_err(|e| e.to_string())?;
        Ok(image)
    }

    /// # Safety
    /// Do not free the returned texture using glow::Context::delete_texture.
    /// All other safety rules when working with OpenGl apply.
    #[allow(unsafe_code)]
    pub unsafe fn load_glow_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<Arc<GlTexture>, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();

        let entry = self
            .glow_imgs
            .entry(format!("{directory}/{filename}"))
            .or_try_insert_with(|| -> Result<_, String> {
                // We force the image to be rgba8 to avoid any weird texture errors.
                // If the image was not rgba8 (say it was rgb8) we would also get a segfault as opengl is expecting a series of bytes with the len of width * height * 4.
                let image = self.load_image(directory, filename)?.into_rgba8();
                // Check that the image will fit into the texture
                // If we dont perform this check, we may get a segfault (dont ask me how i know this)
                assert_eq!(image.len() as u32, image.width() * image.height() * 4);
                let gl = &state!().gl;

                let raw = gl.create_texture()?;
                gl.bind_texture(glow::TEXTURE_2D, Some(raw));

                gl.tex_image_2d(
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
                gl.generate_mipmap(glow::TEXTURE_2D);
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::NEAREST as i32,
                );
                gl.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MAG_FILTER,
                    glow::NEAREST as i32,
                );

                Ok(Arc::new(GlTexture {
                    raw,
                    width: image.width(),
                    height: image.height(),
                }))
            })?;
        Ok(Arc::clone(&entry))
    }

    pub fn clear(&self) {
        self.egui_imgs.clear();
        self.glow_imgs.clear();
    }
}
