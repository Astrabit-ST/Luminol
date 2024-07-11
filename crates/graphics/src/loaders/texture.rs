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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use dashmap::DashMap;

use std::sync::Arc;

use wgpu::util::DeviceExt;

pub struct Loader {
    loaded_textures: DashMap<camino::Utf8PathBuf, Arc<Texture>>,

    placeholder_texture: Arc<Texture>,
    blank_autotile_texture: Arc<Texture>,
    placeholder_image: image::RgbaImage,

    render_state: luminol_egui_wgpu::RenderState,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub texture_id: egui::TextureId,

    render_state: luminol_egui_wgpu::RenderState,
}

impl Drop for Texture {
    fn drop(&mut self) {
        let mut renderer = self.render_state.renderer.write();
        renderer.free_texture(&self.texture_id);
    }
}

fn load_wgpu_texture_from_path(
    filesystem: &impl luminol_filesystem::FileSystem,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> color_eyre::Result<wgpu::Texture> {
    let file = filesystem.read(path)?;
    let texture_data = image::load_from_memory(&file)?.to_rgba8();

    if device.limits().max_texture_dimension_2d < texture_data.width().max(texture_data.height()) {
        return Err(color_eyre::eyre::eyre!(
            "Texture is too large: {}x{}",
            texture_data.width(),
            texture_data.height()
        ));
    }

    Ok(load_wgpu_texture_from_image(
        &texture_data,
        device,
        queue,
        Some(path),
    ))
}

fn load_wgpu_texture_from_image(
    image: &image::RgbaImage,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: Option<&str>,
) -> wgpu::Texture {
    device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            label,
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
        wgpu::util::TextureDataOrder::LayerMajor,
        image,
    )
}

fn register_native_texture(
    render_state: luminol_egui_wgpu::RenderState,
    texture: wgpu::Texture,
    label: Option<&str>,
) -> Arc<Texture> {
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label,
        ..Default::default()
    });
    let texture_id = render_state.renderer.write().register_native_texture(
        &render_state.device,
        &view,
        wgpu::FilterMode::Nearest,
    );
    Arc::new(Texture {
        texture,
        view,
        texture_id,
        render_state,
    })
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

impl Loader {
    pub fn new(render_state: luminol_egui_wgpu::RenderState) -> Self {
        let placeholder_image =
            image::load_from_memory(luminol_macros::include_asset!("assets/placeholder.png"))
                .expect("assets/placeholder.png is not a valid image")
                .to_rgba8();

        let placeholder_texture = load_wgpu_texture_from_image(
            &placeholder_image,
            &render_state.device,
            &render_state.queue,
            Some("assets/placeholder.png"),
        );
        let placeholder_texture = register_native_texture(
            render_state.clone(),
            placeholder_texture,
            Some("placeholder texture"),
        );

        let blank_autotile_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("blank autotile texture"),
                size: wgpu::Extent3d {
                    width: crate::primitives::tiles::AUTOTILE_FRAME_COLS
                        * crate::primitives::tiles::TILE_SIZE,
                    height: crate::primitives::tiles::AUTOTILE_ROWS
                        * crate::primitives::tiles::TILE_SIZE,
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
            });
        let blank_autotile_texture = register_native_texture(
            render_state.clone(),
            blank_autotile_texture,
            Some("blank autotile texture"),
        );

        Self {
            loaded_textures: DashMap::with_capacity(64),

            placeholder_texture,
            blank_autotile_texture,
            placeholder_image,

            render_state,
        }
    }

    pub fn load_now_dir(
        &self,
        filesystem: &impl luminol_filesystem::FileSystem,
        directory: impl AsRef<camino::Utf8Path>,
        file: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<Arc<Texture>> {
        let path = directory.as_ref().join(file.as_ref());
        self.load_now(filesystem, path)
    }

    pub fn load_now(
        &self,
        filesystem: &impl luminol_filesystem::FileSystem,
        path: impl AsRef<camino::Utf8Path>,
    ) -> color_eyre::Result<Arc<Texture>> {
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

        let texture =
            register_native_texture(self.render_state.clone(), texture, Some(path.as_str()));
        self.loaded_textures.insert(path, texture.clone());
        texture
    }

    pub fn get(&self, path: impl AsRef<camino::Utf8Path>) -> Option<Arc<Texture>> {
        self.loaded_textures.get(path.as_ref()).as_deref().cloned()
    }

    pub fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Option<Arc<Texture>> {
        self.loaded_textures
            .remove(path.as_ref())
            .map(|(_, value)| value)
    }

    pub fn clear(&self) {
        self.loaded_textures.clear();
    }

    pub fn placeholder_texture(&self) -> Arc<Texture> {
        self.placeholder_texture.clone()
    }

    pub fn blank_autotile_texture(&self) -> Arc<Texture> {
        self.blank_autotile_texture.clone()
    }

    pub fn placeholder_image(&self) -> &image::RgbaImage {
        &self.placeholder_image
    }
}

// can't use Arc because of orphan rule, must use &instead (this does allow for for &Texture to be used in Image::new tho)
impl From<&Texture> for egui::load::SizedTexture {
    fn from(val: &Texture) -> Self {
        egui::load::SizedTexture {
            id: val.texture_id,
            size: val.size_vec2(),
        }
    }
}
