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

use eframe::wgpu::util::DeviceExt;
use once_cell::sync::Lazy;

use crate::prelude::*;

#[derive(Default)]
pub struct Cache {
    // FIXME: This may not handle reloading textures properly.
    egui_imgs: dashmap::DashMap<String, Arc<RetainedImage>>,
    glow_imgs: dashmap::DashMap<String, Arc<WgpuTexture>>,
}

#[derive(Debug)]
pub struct WgpuTexture {
    pub texture: wgpu::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl WgpuTexture {
    pub fn new(texture: wgpu::Texture, bind_group: wgpu::BindGroup) -> Self {
        Self {
            texture,
            bind_group,
        }
    }

    pub fn size(&self) -> wgpu::Extent3d {
        self.texture.size()
    }

    pub fn size_vec2(&self) -> egui::Vec2 {
        egui::vec2(self.texture.width() as _, self.texture.height() as _)
    }

    pub fn width(&self) -> u32 {
        self.texture.width()
    }

    pub fn height(&self) -> u32 {
        self.texture.height()
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.set_bind_group(0, &self.bind_group, &[]);
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
                entry.path().file_stem().and_then(std::ffi::OsStr::to_str).map(str::to_lowercase) == Some(filename.to_lowercase())
                // entry.path().file_stem() == Some(std::ffi::OsStr::new(filename))
            }) else {
                return Err(format!("{filename} not found in {directory}"));
            };

        let image = image::open(f.path()).map_err(|e| e.to_string())?;
        Ok(image)
    }

    pub fn create_texture_bind_group(texture: &wgpu::Texture) -> wgpu::BindGroup {
        let render_state = &state!().render_state;
        // We *really* don't care about the fields here.
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // We want our texture to use Nearest filtering and repeat.
        // The only time our texture should be repeating is for fogs and panoramas.
        let sampler = render_state
            .device
            .create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });

        // Create the bind group
        // Again, I have no idea why its setup this way
        render_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: Self::bind_group_layout(),
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            })
    }

    pub fn load_wgpu_image(
        &self,
        directory: impl AsRef<str>,
        filename: impl AsRef<str>,
    ) -> Result<Arc<WgpuTexture>, String> {
        let directory = directory.as_ref();
        let filename = filename.as_ref();

        let entry = self
            .glow_imgs
            .entry(format!("{directory}/{filename}"))
            .or_try_insert_with(|| -> Result<_, String> {
                // We force the image to be rgba8 to avoid any weird texture errors.
                // If the image was not rgba8 (say it was rgb8) we would get weird texture errors
                let image = self.load_image(directory, filename)?.into_rgba8();
                // Check that the image will fit into the texture
                // If we dont perform this check, we may get a segfault (dont ask me how i know this)
                assert_eq!(image.len() as u32, image.width() * image.height() * 4);
                let render_state = &state!().render_state;
                // Create the texture and upload the data at the same time.
                // This is just a utility function to avoid boilerplate
                let texture = render_state.device.create_texture_with_data(
                    &render_state.queue,
                    &wgpu::TextureDescriptor {
                        label: Some(&format!("{directory}/{filename}")),
                        size: wgpu::Extent3d {
                            width: image.width(),
                            height: image.height(),
                            depth_or_array_layers: 1,
                        },
                        dimension: wgpu::TextureDimension::D2,
                        mip_level_count: 1,
                        sample_count: 1,
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        usage: wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::COPY_DST
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    },
                    &image,
                );
                let bind_group = Self::create_texture_bind_group(&texture);

                let texture = WgpuTexture {
                    texture,
                    bind_group,
                };

                Ok(Arc::new(texture))
            })?;
        Ok(Arc::clone(&entry))
    }

    pub fn clear(&self) {
        self.egui_imgs.clear();
        self.glow_imgs.clear();
    }

    pub fn bind_group_layout() -> &'static wgpu::BindGroupLayout {
        &LAYOUT
    }
}

static LAYOUT: Lazy<wgpu::BindGroupLayout> = Lazy::new(|| {
    let render_state = &state!().render_state;

    render_state
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            // I just copy pasted this stuff from the wgpu guide.
            // No clue why I need it.
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
});
