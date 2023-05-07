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
#![allow(unsafe_code)]

use std::num::NonZeroU32;
use std::sync::Arc;

use eframe::wgpu::util::DeviceExt;
use image::GenericImageView;

use crate::image_cache::WgpuTexture;
use crate::prelude::*;

pub struct Tilemap {
    /// The tilemap pan.
    pub pan: egui::Vec2,
    /// The scale of the tilemap.
    pub scale: f32,
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    textures: Arc<Textures>,
    tile_vertices: Arc<TileVertices>,
}

struct Textures {
    atlas: Atlas,
    event_texs: HashMap<String, Arc<WgpuTexture>>,
    fog_tex: Option<Arc<WgpuTexture>>,
    pano_tex: Option<Arc<WgpuTexture>>,
}

struct Atlas {
    atlas_texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
    autotile_width: u32,
    tileset_height: u32,
}

struct TileVertices {
    buffer: wgpu::Buffer,
    vertices: u32,
    instances: u32,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];
    const fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

static_assertions::assert_impl_all!(Textures: Send, Sync);

static TILEMAP_SHADER: once_cell::sync::Lazy<wgpu::RenderPipeline> =
    once_cell::sync::Lazy::new(create_tilemap_shader);

const MAX_SIZE: u32 = 2048; // Max texture size in one dimension
const TILE_SIZE: u32 = 32; // Tiles are 32x32
const TILESET_WIDTH: u32 = TILE_SIZE * 8; // Tilesets are 8 tiles across

const AUTOTILE_HEIGHT: u32 = TILE_SIZE * 4; // Autotiles are 4 tiles high
const AUTOTILE_AMOUNT: u32 = 7; // There are 7 autotiles per tileset
const TOTAL_AUTOTILE_HEIGHT: u32 = AUTOTILE_HEIGHT * AUTOTILE_AMOUNT;
const UNDER_HEIGHT: u32 = MAX_SIZE - TOTAL_AUTOTILE_HEIGHT;

impl Tilemap {
    pub fn new(id: i32) -> Result<Tilemap, String> {
        // Load the map.
        let map = state!().data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state!().data_cache.tilesets();
        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let textures = Arc::new(Self::load_data(&map, tileset)?);

        let vertex_buffer = Self::generate_tile_vertices(&map, &textures.atlas);
        let vertex_buffer = Arc::new(vertex_buffer);

        Ok(Self {
            pan: egui::Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,

            textures,
            tile_vertices: vertex_buffer,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut egui::Pos2,
        toggled_layers: &[bool],
        selected_layer: usize,
        dragging_event: bool,
    ) -> egui::Response {
        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let textures = self.textures.clone();
        let tile_vertices = self.tile_vertices.clone();
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |device, queue, _encoder, paint_callback_resources| {
                        //
                        paint_callback_resources.insert(textures.clone());
                        paint_callback_resources.insert(tile_vertices.clone());
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                        let textures: &Arc<Textures> = paint_callback_resources
                            .get()
                            .expect("failed to get tileset textures");
                        let tile_vertices: &Arc<TileVertices> = paint_callback_resources
                            .get()
                            .expect("failed to get vertex buffer");

                        render_pass.set_pipeline(&TILEMAP_SHADER);
                        render_pass.set_bind_group(0, &textures.atlas.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, tile_vertices.buffer.slice(..));

                        render_pass.draw(0..tile_vertices.vertices, 0..tile_vertices.instances);
                    }),
            ),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(
                TILESET_WIDTH as f32,
                self.textures.atlas.tileset_height as f32,
            ),
            egui::Sense::click(),
        );

        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(|device, queue, _encoder, paint_callback_resources| {
                        //
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                    }),
            ),
        });
    }

    #[allow(unused_variables, unused_assignments)]
    fn load_data(map: &rpg::Map, tileset: &rpg::Tileset) -> Result<Textures, String> {
        let state = state!();

        let atlas = Self::build_atlas(tileset)?;

        let event_texs = map
            .events
            .iter()
            .filter_map(|(_, e)| e.pages.first().map(|p| p.graphic.character_name.clone()))
            .filter(|s| !s.is_empty())
            .dedup()
            .map(|char_name| {
                //
                state
                    .image_cache
                    .load_wgpu_image("Graphics/Characters", &char_name)
                    .map(|texture| (char_name, texture))
            })
            .try_collect()?;

        // These two are pretty simple.
        let fog_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Fogs", &tileset.fog_name)
            .ok();

        let pano_tex = state
            .image_cache
            .load_wgpu_image("Graphics/Panoramas", &tileset.panorama_name)
            .ok();

        // Finally create and return the struct.
        Ok(Textures {
            atlas,
            event_texs,
            fog_tex,
            pano_tex,
        })
    }

    fn generate_tile_vertices(map: &rpg::Map, atlas: &Atlas) -> TileVertices {
        let render_state = &state!().render_state;

        let mut vertices: Vec<Vertex> = vec![];
        let mut instances = 0;

        let tile_width = 32. / atlas.atlas_texture.width() as f32;
        let tile_height = 32. / atlas.atlas_texture.height() as f32;
        for (index, tile_id) in map.data.iter().copied().enumerate() {
            if tile_id < 48 {
                continue;
            }
            //     | that is index
            // [0, 200, 96, 385, ...]

            // We reset the x every xsize elements.
            let x = (index % map.data.xsize()) as f32;
            // We reset the y every ysize elements, but only increment it every xsize elements.
            let y = ((index / map.data.xsize()) % map.data.ysize()) as f32;
            // We change the z every xsize * ysize elements.
            let z = (index / (map.data.xsize() * map.data.ysize())) as f32;

            let x = x - map.width as f32;
            let y = map.height as f32 - y;
            let z = z - map.data.zsize() as f32;

            if tile_id >= 384 {
                let tex_x = 0.;
                let tex_y = 0.;

                // Tiles are made like this:
                // A-----C
                // | \ / |
                // | / \ |
                // B-----D

                // FIRST TRIANGLE
                // A-----C
                // |   /
                // | /
                // B

                // A
                vertices.push(Vertex {
                    position: [x, y, z],
                    tex_coords: [tex_x, tex_y],
                });
                // C
                vertices.push(Vertex {
                    position: [x + 1., y, z],
                    tex_coords: [tex_x + tile_width, tex_y],
                });
                // B
                vertices.push(Vertex {
                    position: [x, y + 1., z],
                    tex_coords: [tex_x, tex_y + tile_height],
                });
                instances += 1;

                // SECOND TRIANGLE
                //       C
                //     / |
                //   /   |
                // B-----D
                // C
                vertices.push(Vertex {
                    position: [x + 1., y, z],
                    tex_coords: [tex_x + tile_width, tex_y],
                });
                // D
                vertices.push(Vertex {
                    position: [x + 1., y + 1., z],
                    tex_coords: [tex_x + tile_width, tex_y + tile_height],
                });
                // B
                vertices.push(Vertex {
                    position: [x, y + 1., z],
                    tex_coords: [tex_x, tex_y + tile_height],
                });
                instances += 1;
            }
        }

        let buffer = render_state
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("map_vertices"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

        TileVertices {
            buffer,
            vertices: vertices.len() as u32,
            instances,
        }
    }

    fn build_atlas(tileset: &rpg::Tileset) -> Result<Atlas, String> {
        let tileset_img = state!()
            .image_cache
            .load_image("Graphics/Tilesets", &tileset.tileset_name)?;
        let tileset_img = tileset_img.to_rgba8();
        // Tileset height brought up to the closest MAX_SIZE
        let effective_tileset_height = tileset_img.height() % MAX_SIZE + tileset_img.height();
        println!("effective_tileset_height: {effective_tileset_height}");

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
            let rows_side = (tileset_img.height() - rows_under * UNDER_HEIGHT).div_ceil(MAX_SIZE);
            println!("rows_under: {rows_under}");
            println!("rows_side: {rows_side}");

            width = (rows_under + rows_side) * TILESET_WIDTH;
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
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::all(),
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
            let rows_side = (tileset_img.height() - rows_under * UNDER_HEIGHT).div_ceil(MAX_SIZE);

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

        Ok(Atlas {
            atlas_texture,
            bind_group,
            autotile_width,
            tileset_height: tileset_img.height(),
        })
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

fn create_tilemap_shader() -> wgpu::RenderPipeline {
    let render_state = &state!().render_state;

    let shader_module = render_state
        .device
        .create_shader_module(wgpu::include_wgsl!("tilemap.wgsl"));

    let texture_layout = image_cache::Cache::create_texture_bind_group_layout();
    let pipeline_layout =
        render_state
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tilemap Render Pipeline Layout"),
                bind_group_layouts: &[&texture_layout],
                push_constant_ranges: &[],
            });
    render_state
        .device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tilemap Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(render_state.target_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        })
}
