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
use image::{GenericImage, GenericImageView};

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
    vertex_buffer: Arc<wgpu::Buffer>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2], // NEW!
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

// A-----C
// | \  |
// |  \ |
// B----D
const VERTICES: &[Vertex] = &[
    // A
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    // C
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
    // D
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    // D
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    // B
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    // A
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
];

struct Textures {
    atlas: Arc<WgpuTexture>,
    event_texs: HashMap<String, Arc<WgpuTexture>>,
    fog_tex: Option<Arc<WgpuTexture>>,
    pano_tex: Option<Arc<WgpuTexture>>,
    tileset_height: u32,
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
        let textures = Arc::new(Self::load_data(id)?);
        let vertex_buffer =
            state!()
                .render_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("tilemap vertex buffer"),
                    contents: bytemuck::cast_slice(VERTICES),
                    usage: wgpu::BufferUsages::VERTEX,
                });
        let vertex_buffer = Arc::new(vertex_buffer);

        Ok(Self {
            pan: egui::Vec2::ZERO,
            scale: 100.,
            visible_display: false,
            move_preview: false,

            textures,
            vertex_buffer,
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

        let atlas = self.textures.atlas.clone();
        let vertex_buffer = self.vertex_buffer.clone();
        ui.painter().add(egui::PaintCallback {
            rect: canvas_rect,
            callback: Arc::new(
                egui_wgpu::CallbackFn::new()
                    .prepare(move |device, queue, _encoder, paint_callback_resources| {
                        //
                        paint_callback_resources.insert(atlas.clone());
                        paint_callback_resources.insert(vertex_buffer.clone());
                        vec![]
                    })
                    .paint(move |_info, render_pass, paint_callback_resources| {
                        //
                        let atlas: &Arc<WgpuTexture> = paint_callback_resources
                            .get()
                            .expect("failed to get tileset atlas");
                        let vertex_buffer: &Arc<wgpu::Buffer> = paint_callback_resources
                            .get()
                            .expect("failed to get vertex buffer");

                        render_pass.set_pipeline(&TILEMAP_SHADER);
                        render_pass.set_bind_group(0, &atlas.bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..6, 0..1);
                    }),
            ),
        });

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }

    pub fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16) {
        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(TILESET_WIDTH as f32, self.textures.tileset_height as f32),
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
    fn load_data(id: i32) -> Result<Textures, String> {
        let state = state!();
        // Load the map.

        let map = state.data_cache.load_map(id)?;
        // Get tilesets.
        let tilesets = state.data_cache.tilesets();

        // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
        let tileset = &tilesets[map.tileset_id as usize - 1];

        let (atlas, tileset_height) = Self::build_atlas(tileset)?;
        let atlas = Arc::new(atlas);

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
            tileset_height,
        })
    }

    fn build_atlas(tileset: &rpg::Tileset) -> Result<(WgpuTexture, u32), String> {
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

        let mut auotile_width = 0;
        for at in autotiles.iter().flatten() {
            auotile_width = auotile_width.max(at.texture.width());
        }

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
            width = auotile_width.max(tileset_img.width()); // in case we have less autotiles frames than the tileset is wide
            height = TOTAL_AUTOTILE_HEIGHT + tileset_img.height(); // we're sure that the tileset can fit into the atlas just fine
        } else {
            // I have no idea how this math works.
            // Like at all lmao
            let rows_under = tileset_img
                .height()
                .div_ceil(UNDER_HEIGHT)
                .min(auotile_width.div_ceil(UNDER_HEIGHT));
            let rows_side = (tileset_img.height() - rows_under * UNDER_HEIGHT) / MAX_SIZE;

            width = (rows_under + rows_side) * TILESET_WIDTH;
            height = MAX_SIZE;
        }

        let atlas = render_state
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

        let mut atlas_copy = atlas.as_image_copy();

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
            let rows_under = tileset_img
                .height()
                .div_ceil(UNDER_HEIGHT)
                .min(auotile_width.div_ceil(UNDER_HEIGHT));
            let rows_side = (tileset_img.height() - rows_under * UNDER_HEIGHT) / MAX_SIZE;

            for i in 0..rows_under {
                atlas_copy.origin.x = i * TILESET_WIDTH;
                atlas_copy.origin.y = TOTAL_AUTOTILE_HEIGHT;
                let sub = tileset_img.view(0, UNDER_HEIGHT * i, TILESET_WIDTH, UNDER_HEIGHT);

                let inner_width = sub.inner().width();
                let stride = inner_width * 4;
                let offset = ((UNDER_HEIGHT * i) * inner_width) * 4;

                render_state.queue.write_texture(
                    atlas_copy,
                    sub.inner(),
                    wgpu::ImageDataLayout {
                        offset: offset as wgpu::BufferAddress,
                        bytes_per_row: NonZeroU32::new(stride),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: TILESET_WIDTH,
                        height: UNDER_HEIGHT,
                        depth_or_array_layers: 1,
                    },
                );
            }
            for i in 0..=rows_side {
                atlas_copy.origin.x = TILESET_WIDTH * (rows_under + i);
                atlas_copy.origin.y = 0;
                let sub = tileset_img.view(
                    0,
                    (UNDER_HEIGHT * rows_under) + MAX_SIZE * i,
                    TILESET_WIDTH,
                    MAX_SIZE,
                );

                let inner_width = sub.inner().width();
                let stride = inner_width * 4;
                let offset = ((UNDER_HEIGHT * i) * inner_width) * 4;

                render_state.queue.write_texture(
                    atlas_copy,
                    sub.inner(),
                    wgpu::ImageDataLayout {
                        offset: offset as wgpu::BufferAddress,
                        bytes_per_row: NonZeroU32::new(stride),
                        rows_per_image: None,
                    },
                    wgpu::Extent3d {
                        width: TILESET_WIDTH,
                        height: MAX_SIZE,
                        depth_or_array_layers: 1,
                    },
                );
            }
        }

        let bind_group = image_cache::Cache::create_texture_bind_group(&atlas);

        Ok((WgpuTexture::new(atlas, bind_group), tileset_img.height()))
    }
}

fn write_texture_region<P>(image: image::SubImage<&image::ImageBuffer<P, Vec<P::Subpixel>>>)
where
    P: image::Pixel,
    P::Subpixel: bytemuck::Pod,
{
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
