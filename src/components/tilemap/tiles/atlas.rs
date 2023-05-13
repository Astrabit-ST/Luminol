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

use super::{AUTOTILE_HEIGHT, MAX_SIZE, TILESET_WIDTH, TOTAL_AUTOTILE_HEIGHT, UNDER_HEIGHT};
use crate::prelude::*;

use image::GenericImageView;
use std::{num::NonZeroU32, sync::Arc};

#[derive(Debug)]
pub struct Atlas {
    pub atlas_texture: Arc<image_cache::WgpuTexture>,
    pub autotile_width: u32,
    pub tileset_height: u32,
    pub columns_under: u32,
    pub autotile_frames: [u32; super::AUTOTILE_AMOUNT as usize],
}

impl Atlas {
    pub fn new(tileset: &rpg::Tileset) -> Result<Atlas, String> {
        let tileset_img = state!()
            .image_cache
            .load_image("Graphics/Tilesets", &tileset.tileset_name)?;
        let tileset_img = tileset_img.to_rgba8();

        let autotiles: Vec<_> = tileset
            .autotile_names
            .iter()
            .map(|s| {
                if s.is_empty() {
                    Ok(None)
                } else {
                    state!()
                        .image_cache
                        .load_wgpu_image("Graphics/Autotiles", s)
                        .map(Some)
                }
            })
            .try_collect()?;

        let autotile_frames = std::array::from_fn(|i| {
            autotiles[i]
                .as_deref()
                .map(image_cache::WgpuTexture::width)
                .unwrap_or(0)
                / 96
        });

        let mut autotile_width = 0;
        for at in autotiles.iter().flatten() {
            autotile_width = autotile_width.max(at.texture.width());
        }
        println!("{autotile_width}");

        let render_state = &state!().render_state;
        let mut encoder =
            render_state
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("atlas creation"),
                });

        let width;
        let height;
        if TOTAL_AUTOTILE_HEIGHT + tileset_img.height() < MAX_SIZE {
            width = autotile_width.max(tileset_img.width()); // in case we have less autotiles frames than the tileset is wide
            height = TOTAL_AUTOTILE_HEIGHT + tileset_img.height(); // we're sure that the tileset can fit into the atlas just fine
        } else {
            // I have no idea how this math works.
            // Like at all lmao
            let rows_under = u32::min(
                tileset_img.height().div_ceil(UNDER_HEIGHT),
                autotile_width.div_ceil(TILESET_WIDTH),
            );
            let rows_side = (tileset_img
                .height()
                .saturating_sub(rows_under * UNDER_HEIGHT))
            .div_ceil(MAX_SIZE);
            println!("rows_under: {rows_under}");
            println!("rows_side: {rows_side}");

            width = ((rows_under + rows_side) * TILESET_WIDTH).max(autotile_width);
            height = MAX_SIZE;
        }

        let atlas_texture = render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("tileset_atlas"),
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
            });

        let mut atlas_copy = atlas_texture.as_image_copy();

        for (index, tile) in tileset.autotile_names.iter().enumerate() {
            if tile.is_empty() {
                continue;
            }
            let autotile_tex = state!()
                .image_cache
                .load_wgpu_image("Graphics/Autotiles", tile)?;
            let autotile_copy = autotile_tex.texture.as_image_copy();
            atlas_copy.origin.y = AUTOTILE_HEIGHT * index as u32;

            encoder.copy_texture_to_texture(autotile_copy, atlas_copy, autotile_tex.texture.size())
        }

        render_state.queue.submit(std::iter::once(encoder.finish()));
        atlas_copy.origin.y = TOTAL_AUTOTILE_HEIGHT;

        if TOTAL_AUTOTILE_HEIGHT + tileset_img.height() < MAX_SIZE {
            render_state.queue.write_texture(
                atlas_copy,
                &tileset_img,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * tileset_img.width()),
                    rows_per_image: NonZeroU32::new(tileset_img.height()),
                },
                wgpu::Extent3d {
                    width: tileset_img.width(),
                    height: tileset_img.height(),
                    depth_or_array_layers: 1,
                },
            )
        } else {
            // I have no idea how this math works.
            // Like at all lmao
            let rows_under = u32::min(
                tileset_img.height().div_ceil(UNDER_HEIGHT),
                autotile_width.div_ceil(TILESET_WIDTH),
            );
            let rows_side = (tileset_img
                .height()
                .saturating_sub(rows_under * UNDER_HEIGHT))
            .div_ceil(MAX_SIZE);

            for i in 0..rows_under {
                // out.image(tileset, TILESET_WIDTH * i, AUTOTILE_HEIGHT * AUTOTILE_AMOUNT, TILESET_WIDTH, underHeight, 0, underHeight * i, TILESET_WIDTH, underHeight);
                //     image(img,     dx,                dy,                                dWidth,        dHeight,    sx, sy,              sWidth,        sHeight)
                let y = UNDER_HEIGHT * i;
                let height = if y + UNDER_HEIGHT > tileset_img.height() {
                    tileset_img.height() - y
                } else {
                    UNDER_HEIGHT
                };
                write_texture_region(
                    &atlas_texture,
                    tileset_img.view(0, y, TILESET_WIDTH, height),
                    (TILESET_WIDTH * i, TOTAL_AUTOTILE_HEIGHT),
                )
            }
            for i in 0..rows_side {
                // out.image(tileset, TILESET_WIDTH * (rowsUnder + i), 0, TILESET_WIDTH, MAX_SIZE, 0, (underHeight * rowsUnder) + MAX_SIZE * i, TILESET_WIDTH, MAX_SIZE);
                //     image(img,     dx,                             dy, dWidth,        dHeight, sx,                                       sy, sWidth,         sHeight)
                let y = (UNDER_HEIGHT * rows_under) + MAX_SIZE * i;
                let height = if y + MAX_SIZE > tileset_img.height() {
                    tileset_img.height() - y
                } else {
                    MAX_SIZE
                };
                write_texture_region(
                    &atlas_texture,
                    tileset_img.view(0, y, TILESET_WIDTH, height),
                    (TILESET_WIDTH * (rows_under + i), 0),
                )
            }
        }

        let bind_group = image_cache::Cache::create_texture_bind_group(&atlas_texture);
        let atlas_texture = Arc::new(image_cache::WgpuTexture::new(atlas_texture, bind_group));

        let columns_under = u32::min(
            tileset_img.height().div_ceil(UNDER_HEIGHT),
            autotile_width.div_ceil(TILESET_WIDTH),
        );

        Ok(Atlas {
            atlas_texture,
            autotile_width,
            columns_under,
            tileset_height: tileset_img.height(),
            autotile_frames,
        })
    }

    pub fn bind<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.atlas_texture.bind(render_pass);
    }
}

fn write_texture_region<P>(
    texture: &wgpu::Texture,
    image: image::SubImage<&image::ImageBuffer<P, Vec<P::Subpixel>>>,
    (dest_x, dest_y): (u32, u32),
) where
    P: image::Pixel,
    P::Subpixel: bytemuck::Pod,
{
    let (x, y, width, height) = image.bounds();
    let bytes = bytemuck::cast_slice(image.inner().as_raw());

    let inner_width = image.inner().width();
    // let inner_width = subimage.width();
    let stride = inner_width * std::mem::size_of::<P>() as u32;
    let offset = (y * inner_width + x) * std::mem::size_of::<P>() as u32;

    state!().render_state.queue.write_texture(
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
            bytes_per_row: NonZeroU32::new(stride),
            rows_per_image: None,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}
