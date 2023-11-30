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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use dashmap::{DashMap, DashSet};

use egui::load::{LoadError, SizedTexture, TextureLoadResult, TexturePoll};

use std::sync::Arc;

use wgpu::util::DeviceExt;

pub struct TextureLoader {
    // todo: add a load state enum for loading textures (waiting on file -> file read -> image loaded -> texture loaded)
    loaded_textures: DashMap<camino::Utf8PathBuf, Arc<Texture>>,
    load_errors: DashMap<camino::Utf8PathBuf, anyhow::Error>,
    unloaded_textures: DashSet<camino::Utf8PathBuf>,

    render_state: egui_wgpu::RenderState,
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub texture_id: egui::TextureId,
}

pub const TEXTURE_LOADER_ID: &str = egui::load::generate_loader_id!(TextureLoader);

pub const PROTOCOL: &str = "project://";

// NOTE blindly assumes texture components are 1 byte
fn texture_size_bytes(texture: &wgpu::Texture) -> u32 {
    texture.width()
        * texture.height()
        * texture.depth_or_array_layers()
        * texture.format().components() as u32
}

fn load_wgpu_texture_from_path(
    filesystem: &impl luminol_filesystem::FileSystem,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> anyhow::Result<wgpu::Texture> {
    let file = filesystem.read(path)?;
    let texture_data = image::load_from_memory(&file)?.to_rgba8();

    Ok(device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label: Some(path),
            size: wgpu::Extent3d {
                width: texture_data.width(),
                height: texture_data.height(),
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        },
        &texture_data,
    ))
}

fn supported_uri_to_path(uri: &str) -> Option<&camino::Utf8Path> {
    uri.strip_prefix(PROTOCOL)
        .map(camino::Utf8Path::new)
        .filter(|path| path.extension().filter(|&ext| ext != "svg").is_some())
}

impl Texture {
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
}

impl TextureLoader {
    pub fn new(render_state: egui_wgpu::RenderState) -> Self {
        Self {
            loaded_textures: DashMap::with_capacity(64),
            load_errors: DashMap::new(),
            unloaded_textures: DashSet::with_capacity(64),

            render_state,
        }
    }

    pub fn load_unloaded_textures(
        &self,
        ctx: &egui::Context,
        filesystem: &impl luminol_filesystem::FileSystem,
    ) {
        // dashmap has no drain method so this is the best we can do
        let mut renderer = self.render_state.renderer.write();
        for path in self.unloaded_textures.iter() {
            let texture = match load_wgpu_texture_from_path(
                filesystem,
                &self.render_state.device,
                &self.render_state.queue,
                path.as_str(),
            ) {
                Ok(t) => t,
                Err(error) => {
                    self.load_errors.insert(path.clone(), error);

                    continue;
                }
            };
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(path.as_str()),
                ..Default::default()
            });

            let texture_id = renderer.register_native_texture(
                &self.render_state.device,
                &view,
                wgpu::FilterMode::Nearest,
            );

            self.loaded_textures.insert(
                path.clone(),
                Arc::new(Texture {
                    texture,
                    view,
                    texture_id,
                }),
            );
        }

        if !self.unloaded_textures.is_empty() {
            ctx.request_repaint(); // if we've loaded textures
        }

        self.unloaded_textures.clear();
    }

    pub fn load_now_dir(
        &self,
        filesystem: &impl luminol_filesystem::FileSystem,
        directory: impl AsRef<camino::Utf8Path>,
        file: impl AsRef<camino::Utf8Path>,
    ) -> anyhow::Result<Arc<Texture>> {
        let path = directory.as_ref().join(file.as_ref());
        self.load_now(filesystem, path)
    }

    pub fn load_now(
        &self,
        filesystem: &impl luminol_filesystem::FileSystem,
        path: impl AsRef<camino::Utf8Path>,
    ) -> anyhow::Result<Arc<Texture>> {
        let path = path.as_ref().as_str();

        let texture = load_wgpu_texture_from_path(
            filesystem,
            &self.render_state.device,
            &self.render_state.queue,
            path,
        )?;

        Ok(self.register_texture(path.to_string(), texture))
    }

    pub fn register_texture(
        &self,
        path: impl Into<camino::Utf8PathBuf>,
        texture: wgpu::Texture,
    ) -> Arc<Texture> {
        let path = path.into();

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(path.as_str()),
            ..Default::default()
        });

        // todo maybe use custom sampler descriptor?
        // would allow for better texture names in debuggers
        let texture_id = self.render_state.renderer.write().register_native_texture(
            &self.render_state.device,
            &view,
            wgpu::FilterMode::Nearest,
        );

        let texture = Arc::new(Texture {
            texture,
            view,
            texture_id,
        });
        self.loaded_textures.insert(path, texture.clone());
        texture
    }

    pub fn get(&self, path: impl AsRef<camino::Utf8Path>) -> Option<Arc<Texture>> {
        self.loaded_textures.get(path.as_ref()).as_deref().cloned()
    }
}

impl egui::load::TextureLoader for TextureLoader {
    fn id(&self) -> &str {
        TEXTURE_LOADER_ID
    }

    fn load(
        &self,
        _: &egui::Context,
        uri: &str,
        _: egui::TextureOptions,
        _: egui::SizeHint,
    ) -> TextureLoadResult {
        // check if the uri is supported (starts with project:// and does not end with ".svg")
        let Some(path) = supported_uri_to_path(uri) else {
            return Err(LoadError::NotSupported);
        };

        if let Some(texture) = self.loaded_textures.get(path).as_deref() {
            return Ok(TexturePoll::Ready {
                texture: SizedTexture::new(texture.texture_id, texture.size_vec2()),
            });
        }

        // if during loading we errored, check if it's because the image crate doesn't support loading this file format
        if let Some(error) = self.load_errors.get(path) {
            match error.downcast_ref::<image::ImageError>() {
                Some(image::ImageError::Decoding(error))
                    if matches!(error.format_hint(), image::error::ImageFormatHint::Unknown) =>
                {
                    return Err(LoadError::NotSupported)
                }
                Some(image::ImageError::Unsupported(_)) => return Err(LoadError::NotSupported),
                _ => return Err(LoadError::Loading(error.to_string())),
            }
        }

        self.unloaded_textures.insert(path.to_path_buf());

        Ok(TexturePoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        let Some(path) = supported_uri_to_path(uri) else {
            return;
        };

        self.loaded_textures.remove(path);
    }

    fn forget_all(&self) {
        self.loaded_textures.clear();
        self.load_errors.clear();
    }

    fn byte_size(&self) -> usize {
        self.loaded_textures
            .iter()
            .map(|texture| texture_size_bytes(&texture.texture) as usize)
            .sum()
    }
}
