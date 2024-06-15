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

use color_eyre::eyre::Context;
use image::EncodableLayout;
use itertools::Itertools;
use wgpu::util::DeviceExt;

use std::sync::Arc;

use fragile::Fragile;

use crate::{quad::Quad, sprite::Sprite, tiles::Atlas, viewport::Viewport, GraphicsState};

pub struct Event {
    sprite: Arc<Sprite>,
    viewport: Arc<Viewport>,
    pub sprite_size: egui::Vec2,
}

// wgpu types are not Send + Sync on webassembly, so we use fragile to make sure we never access any wgpu resources across thread boundaries
pub struct Callback {
    sprite: Fragile<Arc<Sprite>>,
    graphics_state: Fragile<Arc<GraphicsState>>,
}

impl Callback {
    pub fn paint<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        let sprite = self.sprite.get();
        let graphics_state = self.graphics_state.get();

        sprite.draw(graphics_state, render_pass);
    }
}

impl luminol_egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a luminol_egui_wgpu::CallbackResources,
    ) {
        self.paint(render_pass)
    }
}

impl Event {
    // code smell, fix
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        event: &luminol_data::rpg::Event,
        atlas: &Atlas,
    ) -> color_eyre::Result<Option<Self>> {
        let Some(page) = event.pages.first() else {
            color_eyre::eyre::bail!("event does not have first page");
        };

        let texture = if let Some(ref filename) = page.graphic.character_name {
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Characters", filename)
                .wrap_err_with(|| format!("Error loading event character graphic {filename:?}"));
            match texture {
                Ok(t) => t,
                Err(e) => {
                    graphics_state.send_texture_error(e);

                    let placeholder_char_texture = graphics_state
                        .texture_loader
                        .get("placeholder_char_texture")
                        .unwrap_or_else(|| {
                            let placeholder_img = graphics_state.placeholder_img();

                            graphics_state.texture_loader.register_texture(
                                "placeholder_char_texture",
                                graphics_state.render_state.device.create_texture_with_data(
                                    &graphics_state.render_state.queue,
                                    &wgpu::TextureDescriptor {
                                        label: Some("placeholder_char_texture"),
                                        size: wgpu::Extent3d {
                                            width: 128,
                                            height: 128,
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
                                    &itertools::iproduct!(0..128, 0..128, 0..4)
                                        .map(|(y, x, c)| {
                                            // Tile the placeholder image
                                            placeholder_img.as_bytes()[(c
                                                + (x % placeholder_img.width()) * 4
                                                + (y % placeholder_img.height())
                                                    * 4
                                                    * placeholder_img.width())
                                                as usize]
                                        })
                                        .collect_vec(),
                                ),
                            )
                        });

                    placeholder_char_texture
                }
            }
        } else if page.graphic.tile_id.is_some() {
            atlas.atlas_texture.clone()
        } else {
            return Ok(None);
        };

        let (quads, viewport, sprite_size) = if let Some(id) = page.graphic.tile_id {
            // Why does this have to be + 1?
            let quad = atlas.calc_quad((id + 1) as i16);

            let viewport = Arc::new(Viewport::new(graphics_state, 32., 32.));

            (quad, viewport, egui::vec2(32., 32.))
        } else {
            let cw = texture.width() as f32 / 4.;
            let ch = texture.height() as f32 / 4.;
            let pos = egui::Rect::from_min_size(
                egui::pos2(
                    0., //(event.x as f32 * 32.) + (16. - (cw / 2.)),
                    0., //(event.y as f32 * 32.) + (32. - ch),
                ),
                egui::vec2(cw, ch),
            );

            // Reduced by 0.01 px on all sides to reduce texture bleeding
            let tex_coords = egui::Rect::from_min_size(
                egui::pos2(
                    page.graphic.pattern as f32 * cw + 0.01,
                    (page.graphic.direction as f32 - 2.) / 2. * ch + 0.01,
                ),
                egui::vec2(cw - 0.02, ch - 0.02),
            );
            let quad = Quad::new(pos, tex_coords, 0.0);

            let viewport = Arc::new(Viewport::new(graphics_state, cw, ch));

            (quad, viewport, egui::vec2(cw, ch))
        };

        let sprite = Arc::new(Sprite::new(
            graphics_state,
            viewport.clone(),
            quads,
            texture,
            page.graphic.blend_type,
            page.graphic.character_hue,
            page.graphic.opacity,
        ));

        Ok(Some(Self {
            sprite,
            viewport,
            sprite_size,
        }))
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }

    pub fn set_proj(&self, render_state: &luminol_egui_wgpu::RenderState, proj: glam::Mat4) {
        self.viewport.set_proj(render_state, proj);
    }

    pub fn callback(&self, graphics_state: Arc<GraphicsState>) -> Callback {
        Callback {
            sprite: Fragile::new(self.sprite.clone()),
            graphics_state: Fragile::new(graphics_state),
        }
    }

    pub fn paint(
        &self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        painter.add(luminol_egui_wgpu::Callback::new_paint_callback(
            rect,
            self.callback(graphics_state),
        ));
    }
}
