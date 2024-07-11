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

use super::autotile_ids::AUTOTILES;
use crate::{GraphicsState, Quad, Texture};

pub const MAX_SIZE: u32 = 8192; // Max texture size in one dimension
pub const TILE_SIZE: u32 = 32; // Tiles are 32x32
pub const TILESET_COLUMNS: u32 = 8; // Tilesets are 8 tiles across
pub const TILESET_WIDTH: u32 = TILE_SIZE * TILESET_COLUMNS; // self explanatory

pub const AUTOTILE_ID_AMOUNT: u32 = 48; // there are 48 tile ids per autotile
pub const AUTOTILE_FRAME_COLS: u32 = TILESET_COLUMNS; // this is how many "columns" of autotiles there are per frame
pub const AUTOTILE_AMOUNT: u32 = 7; // There are 7 autotiles per tileset
pub const TOTAL_AUTOTILE_ID_AMOUNT: u32 = AUTOTILE_ID_AMOUNT * (AUTOTILE_AMOUNT + 1); // the first 384 tile ids are for autotiles (including empty tiles)

pub const AUTOTILE_ROWS: u32 = AUTOTILE_ID_AMOUNT / AUTOTILE_FRAME_COLS; // split up the 48 tiles across each tileset row
pub const TOTAL_AUTOTILE_ROWS: u32 = AUTOTILE_ROWS * AUTOTILE_AMOUNT; // total number of rows for all autotiles combined
pub const AUTOTILE_ROW_HEIGHT: u32 = AUTOTILE_ROWS * TILE_SIZE; // This is how high one row of autotiles is
pub const TOTAL_AUTOTILE_HEIGHT: u32 = AUTOTILE_ROW_HEIGHT * AUTOTILE_AMOUNT; // self explanatory
pub const HEIGHT_UNDER_AUTOTILES: u32 = MAX_SIZE - TOTAL_AUTOTILE_HEIGHT; // this is the height under autotiles
pub const ROWS_UNDER_AUTOTILES: u32 = MAX_SIZE / TILE_SIZE - TOTAL_AUTOTILE_ROWS; // number of rows under autotiles
pub const ROWS_UNDER_AUTOTILES_TIMES_COLUMNS: u32 = ROWS_UNDER_AUTOTILES * TILESET_COLUMNS;

pub const AUTOTILE_FRAME_WIDTH: u32 = AUTOTILE_FRAME_COLS * TILE_SIZE; // This is per frame!

use image::GenericImageView;
use std::sync::Arc;

#[derive(Clone)]
pub struct Atlas {
    pub atlas_texture: Arc<Texture>,
    pub autotile_width: u32,
    pub tileset_height: u32,
    pub autotile_frames: [u32; AUTOTILE_AMOUNT as usize],
}

impl Atlas {
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        tileset: &luminol_data::rpg::Tileset,
    ) -> Atlas {
        let tileset_img = tileset.tileset_name.as_ref().and_then(|tileset_name| {
            let result = filesystem
                .read(camino::Utf8Path::new("Graphics/Tilesets").join(tileset_name))
                .and_then(|file| image::load_from_memory(&file).map_err(|e| e.into()))
                .wrap_err_with(|| format!("Error loading atlas tileset {tileset_name:?}"));
            // we don't actually need to unwrap this to a placeholder image because we fill in the atlas texture with the placeholder image.
            match result {
                Ok(img) => Some(img.into_rgba8()),
                Err(e) => {
                    graphics_state.send_texture_error(e);
                    None
                }
            }
        });

        let tileset_height = tileset_img
            .as_ref()
            .map(|i| i.height() / TILE_SIZE * TILE_SIZE)
            .unwrap_or(256);

        let autotiles = tileset
            .autotile_names
            .iter()
            .map(|s| {
                if s.is_empty() {
                    Some(graphics_state.texture_loader.blank_autotile_texture())
                } else {
                    graphics_state
                        .texture_loader
                        .load_now_dir(filesystem, "Graphics/Autotiles", s)
                        .wrap_err_with(|| format!("Error loading atlas autotiles {s:?}"))
                        .map_or_else(
                            |e| {
                                graphics_state.send_texture_error(e);
                                None
                            },
                            Some,
                        )
                }
            })
            .collect_vec();

        let autotile_frames = std::array::from_fn(|i| {
            autotiles[i]
                .as_deref()
                .map(Texture::width)
                // Why unwrap with a width of 96? Even though the autotile doesn't exist, it still has an effective width on the atlas of one frame.
                // Further rendering code breaks down with an autotile width of 0, anyway.
                .unwrap_or(96)
                / 96
        });

        let autotile_width = autotile_frames
            .iter()
            .map(|f| f * AUTOTILE_FRAME_WIDTH)
            .max()
            .unwrap_or(AUTOTILE_FRAME_WIDTH);

        let mut encoder = graphics_state.render_state.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("atlas creation"),
            },
        );

        let width;
        let height;

        let rows_under;
        let rows_side;
        if TOTAL_AUTOTILE_HEIGHT + tileset_height < MAX_SIZE {
            width = autotile_width.max(TILESET_WIDTH); // in case we have less autotiles frames than the tileset is wide
            height = TOTAL_AUTOTILE_HEIGHT + tileset_height; // we're sure that the tileset can fit into the atlas just fine

            rows_under = 1;
            rows_side = 0;
        } else {
            // Find out how many rows are under autotiles
            // Take the smallest of these
            rows_under = u32::min(
                // How many times can the tileset fit under the autotiles?
                tileset_height.div_ceil(HEIGHT_UNDER_AUTOTILES),
                // How many columns of autotiles are there
                autotile_width.div_ceil(TILESET_WIDTH),
            );
            // Find out how many rows would fit on the side by dividing the left over height by MAX_SIZE
            rows_side = tileset_height
                .saturating_sub(rows_under * HEIGHT_UNDER_AUTOTILES)
                .div_ceil(MAX_SIZE);

            width = ((rows_under + rows_side) * TILESET_WIDTH).max(autotile_width);
            height = MAX_SIZE;
        }

        let placeholder_img = graphics_state.texture_loader.placeholder_image();
        let atlas_texture = graphics_state.render_state.device.create_texture_with_data(
            &graphics_state.render_state.queue,
            &wgpu::TextureDescriptor {
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
        let mut atlas_copy = atlas_texture.as_image_copy();

        for (index, autotile_texture) in
            autotiles
                .into_iter()
                .enumerate()
                .flat_map(|(index, autotile_texture)| {
                    autotile_texture.map(|autotile_texture| (index, autotile_texture))
                })
        {
            let mut autotile_copy = autotile_texture.texture.as_image_copy();

            let frame_y = index as u32 * AUTOTILE_ROW_HEIGHT;
            for frame in 0..autotile_frames[index] {
                let frame_x = frame * AUTOTILE_FRAME_WIDTH;
                for (index, autotile) in AUTOTILES.into_iter().enumerate() {
                    // Reset x every 8 tiles
                    let autotile_x = index as u32 % AUTOTILE_FRAME_COLS * TILE_SIZE;
                    // Increase y every 8 tiles
                    let autotile_y = index as u32 / AUTOTILE_FRAME_COLS * TILE_SIZE;

                    for (index, sub_tile) in autotile.into_iter().enumerate() {
                        let sub_tile_x = index as u32 % 2 * 16;
                        let sub_tile_y = index as u32 / 2 * 16;

                        atlas_copy.origin.x = frame_x + autotile_x + sub_tile_x;
                        atlas_copy.origin.y = frame_y + autotile_y + sub_tile_y;

                        let tile_x = sub_tile % 6 * 16;
                        let tile_y = sub_tile / 6 * 16;

                        autotile_copy.origin.x = tile_x + frame * 96;
                        autotile_copy.origin.y = tile_y;

                        encoder.copy_texture_to_texture(
                            autotile_copy,
                            atlas_copy,
                            wgpu::Extent3d {
                                width: 16,
                                height: 16,
                                depth_or_array_layers: 1,
                            },
                        );
                    }
                }
            }
        }

        graphics_state
            .render_state
            .queue
            .submit(std::iter::once(encoder.finish()));

        atlas_copy.origin.x = 0;
        if let Some(tileset_img) = tileset_img {
            if TOTAL_AUTOTILE_HEIGHT + tileset_height < MAX_SIZE {
                write_texture_region(
                    &graphics_state.render_state,
                    &atlas_texture,
                    tileset_img.view(0, 0, TILESET_WIDTH, tileset_height),
                    (0, TOTAL_AUTOTILE_HEIGHT),
                )
            } else {
                for i in 0..rows_under {
                    let y = HEIGHT_UNDER_AUTOTILES * i;
                    let height = if y + HEIGHT_UNDER_AUTOTILES > tileset_height {
                        tileset_height - y
                    } else {
                        HEIGHT_UNDER_AUTOTILES
                    };
                    write_texture_region(
                        &graphics_state.render_state,
                        &atlas_texture,
                        tileset_img.view(0, y, TILESET_WIDTH, height),
                        (TILESET_WIDTH * i, TOTAL_AUTOTILE_HEIGHT),
                    )
                }
                for i in 0..rows_side {
                    let y = (HEIGHT_UNDER_AUTOTILES * rows_under) + MAX_SIZE * i;
                    let height = if y + MAX_SIZE > tileset_height {
                        tileset_height - y
                    } else {
                        MAX_SIZE
                    };
                    write_texture_region(
                        &graphics_state.render_state,
                        &atlas_texture,
                        tileset_img.view(0, y, TILESET_WIDTH, height),
                        (TILESET_WIDTH * (rows_under + i), 0),
                    )
                }
            }
        }

        let atlas_texture = graphics_state
            .texture_loader
            .register_texture(format!("tileset_atlases/{}", tileset.id), atlas_texture);

        Atlas {
            atlas_texture,
            autotile_width,
            tileset_height,
            autotile_frames,
        }
    }

    pub fn calc_quad(&self, tile: i16) -> Quad {
        let tile_u32 = if tile < 0 { 0 } else { tile as u32 };

        let is_autotile = tile_u32 < TOTAL_AUTOTILE_ID_AMOUNT;
        let max_frame_count = self.autotile_width / AUTOTILE_FRAME_WIDTH;
        let max_tiles_under_autotiles = max_frame_count * ROWS_UNDER_AUTOTILES_TIMES_COLUMNS;
        let is_under_autotiles =
            !is_autotile && tile_u32 - TOTAL_AUTOTILE_ID_AMOUNT < max_tiles_under_autotiles;

        let atlas_tile_position = if tile_u32 < AUTOTILE_ID_AMOUNT {
            egui::pos2(0., 0.)
        } else if is_autotile {
            egui::pos2(
                ((tile_u32 - AUTOTILE_ID_AMOUNT) % AUTOTILE_FRAME_COLS * TILE_SIZE) as f32,
                ((tile_u32 - AUTOTILE_ID_AMOUNT) / AUTOTILE_FRAME_COLS * TILE_SIZE) as f32,
            )
        } else if is_under_autotiles {
            egui::pos2(
                ((tile_u32 % TILESET_COLUMNS
                    + (tile_u32 - TOTAL_AUTOTILE_ID_AMOUNT) / ROWS_UNDER_AUTOTILES_TIMES_COLUMNS
                        * TILESET_COLUMNS)
                    * TILE_SIZE) as f32,
                (((tile_u32 - TOTAL_AUTOTILE_ID_AMOUNT) / TILESET_COLUMNS % ROWS_UNDER_AUTOTILES
                    + TOTAL_AUTOTILE_ROWS)
                    * TILE_SIZE) as f32,
            )
        } else {
            egui::pos2(
                ((tile_u32 % TILESET_COLUMNS
                    + ((tile_u32 - TOTAL_AUTOTILE_ID_AMOUNT - max_tiles_under_autotiles)
                        / (MAX_SIZE / TILE_SIZE * TILESET_COLUMNS)
                        + max_frame_count)
                        * TILESET_COLUMNS)
                    * TILE_SIZE) as f32,
                ((tile_u32 - TOTAL_AUTOTILE_ID_AMOUNT - max_tiles_under_autotiles)
                    / TILESET_COLUMNS
                    % (MAX_SIZE / TILE_SIZE)
                    * TILE_SIZE) as f32,
            )
        };

        Quad::new(
            egui::Rect::from_min_size(
                egui::pos2(0., 0.),
                egui::vec2(TILE_SIZE as f32, TILE_SIZE as f32),
            ),
            // Reduced by 0.01 px on all sides to decrease texture bleeding
            egui::Rect::from_min_size(
                atlas_tile_position + egui::vec2(0.01, 0.01),
                egui::vec2(TILE_SIZE as f32 - 0.02, TILE_SIZE as f32 - 0.02),
            ),
        )
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
