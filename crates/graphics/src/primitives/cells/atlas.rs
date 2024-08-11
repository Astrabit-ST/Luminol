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

use color_eyre::eyre::WrapErr;
use image::EncodableLayout;
use itertools::Itertools;
use wgpu::util::DeviceExt;

use crate::{GraphicsState, Quad, Texture};

pub const MAX_SIZE: u32 = 8192; // Max texture size in one dimension
pub const CELL_SIZE: u32 = 192; // Animation cells are 192x192
pub const ANIMATION_COLUMNS: u32 = 5; // Animation sheets are 5 cells wide
pub const ANIMATION_WIDTH: u32 = CELL_SIZE * ANIMATION_COLUMNS;
pub const MAX_ROWS: u32 = MAX_SIZE / CELL_SIZE; // Max rows of cells that can fit before wrapping
pub const MAX_HEIGHT: u32 = MAX_ROWS * CELL_SIZE;
pub const MAX_CELLS: u32 = MAX_ROWS * ANIMATION_COLUMNS;

use image::GenericImageView;
use std::sync::Arc;

#[derive(Clone)]
pub struct Atlas {
    atlas_texture: Arc<Texture>,
    animation_height: u32,
}

impl Atlas {
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        animation_name: Option<&camino::Utf8Path>,
    ) -> Atlas {
        let animation_img = animation_name.as_ref().and_then(|animation_name| {
            let result = filesystem
                .read(camino::Utf8Path::new("Graphics/Animations").join(animation_name))
                .and_then(|file| image::load_from_memory(&file).map_err(|e| e.into()))
                .wrap_err_with(|| format!("Error loading atlas animation {animation_name:?}"));
            // we don't actually need to unwrap this to a placeholder image because we fill in the atlas texture with the placeholder image.
            match result {
                Ok(img) => Some(img.into_rgba8()),
                Err(e) => {
                    graphics_state.send_texture_error(e);
                    None
                }
            }
        });

        let animation_height = animation_img
            .as_ref()
            .map(|i| i.height() / CELL_SIZE * CELL_SIZE)
            .unwrap_or(CELL_SIZE);

        let wrap_columns = animation_height.div_ceil(MAX_SIZE);
        let width = wrap_columns * ANIMATION_WIDTH;
        let height = animation_height.min(MAX_SIZE);

        let placeholder_img = graphics_state.texture_loader.placeholder_image();
        let atlas_texture = graphics_state.render_state.device.create_texture_with_data(
            &graphics_state.render_state.queue,
            &wgpu::TextureDescriptor {
                label: Some("cells_atlas"),
                size: wgpu::Extent3d {
                    width,
                    height,
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
            // we can avoid this collect_vec() by mapping a buffer and then copying that to a texture. it'd also allow us to copy everything easier too. do we want to do this?
            &itertools::iproduct!(0..height, 0..width, 0..4)
                .map(|(y, x, c)| {
                    // Tile the placeholder image to fill the atlas
                    placeholder_img.as_bytes()[(c
                        + (x % placeholder_img.width()) * 4
                        + (y % placeholder_img.height()) * 4 * placeholder_img.width())
                        as usize]
                })
                .collect_vec(),
        );

        if let Some(animation_img) = animation_img {
            for i in 0..wrap_columns {
                let region_width = ANIMATION_WIDTH.min(animation_img.width());
                let region_height = if i == wrap_columns - 1 {
                    animation_height - MAX_HEIGHT * i
                } else {
                    MAX_HEIGHT
                };
                write_texture_region(
                    &graphics_state.render_state,
                    &atlas_texture,
                    animation_img.view(0, MAX_HEIGHT * i, region_width, region_height),
                    (ANIMATION_WIDTH * i, 0),
                );
            }
        }

        let atlas_texture = graphics_state.texture_loader.register_texture(
            format!(
                "animation_atlases/{}",
                animation_name.unwrap_or(&camino::Utf8PathBuf::default())
            ),
            atlas_texture,
        );

        Atlas {
            atlas_texture,
            animation_height,
        }
    }

    pub fn calc_quad(&self, cell: i16) -> Quad {
        let cell_u32 = if cell < 0 { 0 } else { cell as u32 };

        let atlas_cell_position = egui::pos2(
            ((cell_u32 % ANIMATION_COLUMNS + (cell_u32 / MAX_CELLS) * ANIMATION_COLUMNS)
                * CELL_SIZE) as f32,
            (cell_u32 / ANIMATION_COLUMNS % MAX_ROWS * CELL_SIZE) as f32,
        );

        Quad::new(
            egui::Rect::from_min_size(
                egui::pos2(0., 0.),
                egui::vec2(CELL_SIZE as f32, CELL_SIZE as f32),
            ),
            // Reduced by 0.01 px on all sides to decrease texture bleeding
            egui::Rect::from_min_size(
                atlas_cell_position + egui::vec2(0.01, 0.01),
                egui::vec2(CELL_SIZE as f32 - 0.02, CELL_SIZE as f32 - 0.02),
            ),
        )
    }

    /// Returns this atlas's texture
    #[inline]
    pub fn texture(&self) -> &Arc<Texture> {
        &self.atlas_texture
    }

    /// Returns the height of the original animation texture in pixels
    #[inline]
    pub fn animation_height(&self) -> u32 {
        self.animation_height
    }

    /// Calculates the total number of cells in the atlas based on the size of the texture
    #[inline]
    pub fn num_patterns(&self) -> u32 {
        self.animation_height / CELL_SIZE * ANIMATION_COLUMNS
    }
}

fn write_texture_region<P>(
    render_state: &luminol_egui_wgpu::RenderState,
    texture: &wgpu::Texture,
    image: image::SubImage<&image::ImageBuffer<P, Vec<P::Subpixel>>>,
    (dest_x, dest_y): (u32, u32),
) where
    P: image::Pixel,
    P::Subpixel: bytemuck::Pod,
{
    let (x, y) = image.offsets();
    let (width, height) = image.dimensions();
    let bytes = bytemuck::cast_slice(image.inner().as_raw());

    let inner_width = image.inner().width();
    // let inner_width = subimage.width();
    let stride = inner_width * std::mem::size_of::<P>() as u32;
    let offset = (y * inner_width + x) * std::mem::size_of::<P>() as u32;

    render_state.queue.write_texture(
        wgpu::ImageCopyTexture {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: dest_x,
                y: dest_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        bytes,
        wgpu::ImageDataLayout {
            offset: offset as wgpu::BufferAddress,
            bytes_per_row: Some(stride),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}
