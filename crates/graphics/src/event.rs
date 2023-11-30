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

use std::sync::Arc;

use crate::{quad::Quad, sprite::Sprite, tiles::Atlas, viewport::Viewport, GraphicsState};

pub struct Event {
    sprite: Arc<Sprite>,
    viewport: Arc<Viewport>,
    pub sprite_size: egui::Vec2,
}

struct Callback {
    sprite: Arc<Sprite>,
    graphics_state: Arc<GraphicsState>,
}

//? SAFETY:
//? wgpu resources are not Send + Sync on wasm, but egui_wgpu::CallbackTrait requires Send + Sync (because egui::Context is Send + Sync)
//? as long as this callback does not leave the thread it was created on on wasm (which it shouldn't be) these are ok.
#[allow(unsafe_code)]
unsafe impl Send for Callback {}
#[allow(unsafe_code)]
unsafe impl Sync for Callback {}

impl egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        self.sprite.draw(&self.graphics_state, render_pass);
    }
}

impl Event {
    // code smell, fix
    pub fn new(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        event: &luminol_data::rpg::Event,
        atlas: &Atlas,
    ) -> anyhow::Result<Option<Self>> {
        let Some(page) = event.pages.first() else {
            anyhow::bail!("event does not have first page");
        };

        let texture = if let Some(ref filename) = page.graphic.character_name {
            graphics_state.texture_loader.load_now_dir(
                filesystem,
                "Graphics/Characters",
                filename,
            )?
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

    pub fn set_proj(&self, render_state: &egui_wgpu::RenderState, proj: glam::Mat4) {
        self.viewport.set_proj(render_state, proj);
    }

    pub fn paint(
        &self,
        graphics_state: Arc<GraphicsState>,
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        painter.add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Callback {
                sprite: self.sprite.clone(),
                graphics_state,
            },
        ));
    }
}
