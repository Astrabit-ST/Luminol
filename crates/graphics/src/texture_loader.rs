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

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use wgpu::util::DeviceExt;

pub struct TextureLoader {
    loaded_textures: DashMap<String, Arc<Texture>>,
    load_errors: DashMap<String, anyhow::Error>,
    unloaded_textures: DashSet<String>,

    loaded_bytes: AtomicUsize,

    render_state: egui_wgpu::RenderState,
}

pub struct Texture {
    wgpu: wgpu::Texture,
    egui: egui::TextureId,
}

pub const LOADER_ID: &str = egui::load::generate_loader_id!(TextureLoader);

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

impl TextureLoader {
    pub fn new(render_state: egui_wgpu::RenderState) -> Self {
        Self {
            loaded_textures: DashMap::with_capacity(64),
            load_errors: DashMap::new(),
            unloaded_textures: DashSet::with_capacity(64),

            loaded_bytes: AtomicUsize::new(0),

            render_state,
        }
    }

    pub fn load_unloaded_textures(&self, filesystem: &impl luminol_filesystem::FileSystem) {
        // dashmap has no drain method so this is the best we can do
        let mut renderer = self.render_state.renderer.write();
        for path in self.unloaded_textures.iter() {
            let wgpu_texture = match load_wgpu_texture_from_path(
                filesystem,
                &self.render_state.device,
                &self.render_state.queue,
                path.as_str(),
            ) {
                Ok(t) => t,
                Err(error) => {
                    self.load_errors.insert(path.to_string(), error);

                    continue;
                }
            };

            self.loaded_bytes.fetch_add(
                texture_size_bytes(&wgpu_texture) as usize,
                Ordering::Relaxed,
            );

            let texture_id = renderer.register_native_texture(
                &self.render_state.device,
                &wgpu_texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some(&path),
                    ..Default::default()
                }),
                wgpu::FilterMode::Nearest,
            );

            self.loaded_textures.insert(
                path.to_string(),
                Arc::new(Texture {
                    wgpu: wgpu_texture,
                    egui: texture_id,
                }),
            );
        }
        self.unloaded_textures.clear();
    }

    pub fn load_now(
        &self,
        filesystem: &impl luminol_filesystem::FileSystem,
        path: impl AsRef<camino::Utf8Path>,
    ) -> anyhow::Result<Arc<Texture>> {
        let path = path.as_ref().as_str();

        let wgpu_texture = load_wgpu_texture_from_path(
            filesystem,
            &self.render_state.device,
            &self.render_state.queue,
            path,
        )?;

        Ok(self.register_texture(path.to_string(), wgpu_texture))
    }

    pub fn register_texture(&self, uri: String, wgpu_texture: wgpu::Texture) -> Arc<Texture> {
        self.loaded_bytes.fetch_add(
            texture_size_bytes(&wgpu_texture) as usize,
            Ordering::Relaxed,
        );

        // todo maybe use custom sampler descriptor?
        // would allow for better texture names in debuggers
        let texture_id = self.render_state.renderer.write().register_native_texture(
            &self.render_state.device,
            &wgpu_texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some(&uri),
                ..Default::default()
            }),
            wgpu::FilterMode::Nearest,
        );

        let texture = Arc::new(Texture {
            wgpu: wgpu_texture,
            egui: texture_id,
        });
        self.loaded_textures.insert(uri, texture.clone());
        texture
    }

    pub fn get(&self, path: impl AsRef<camino::Utf8Path>) -> Option<Arc<Texture>> {
        self.loaded_textures
            .get(path.as_ref().as_str())
            .as_deref()
            .cloned()
    }
}

impl egui::load::TextureLoader for TextureLoader {
    fn id(&self) -> &str {
        LOADER_ID
    }

    fn load(
        &self,
        _: &egui::Context,
        uri: &str,
        _: egui::TextureOptions,
        _: egui::SizeHint,
    ) -> TextureLoadResult {
        if let Some(texture) = self.loaded_textures.get(uri).as_deref() {
            let extents = texture.wgpu.size();
            return Ok(TexturePoll::Ready {
                texture: SizedTexture::new(
                    texture.egui,
                    egui::vec2(extents.width as f32, extents.height as f32),
                ),
            });
        }

        if let Some(error) = self.load_errors.get(uri) {
            return Err(LoadError::Loading(error.to_string()));
        }

        self.unloaded_textures.insert(uri.to_string());

        Ok(TexturePoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        if let Some((_, texture)) = self.loaded_textures.remove(uri) {
            self.loaded_bytes.fetch_sub(
                texture_size_bytes(&texture.wgpu) as usize,
                Ordering::Relaxed,
            );
        }
    }

    fn forget_all(&self) {
        self.loaded_textures.clear();
        self.loaded_bytes.store(0, Ordering::Relaxed);
    }

    fn byte_size(&self) -> usize {
        self.loaded_bytes.load(Ordering::Relaxed)
    }
}
