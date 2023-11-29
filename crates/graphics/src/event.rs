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

#[derive(Debug)]
pub struct Event {
    resources: Arc<Resources>,
    pub sprite_size: egui::Vec2,
}

#[derive(Debug)]
struct Resources {
    sprite: crate::sprite::Sprite,
    viewport: crate::viewport::Viewport,
}

struct Callback {
    resources: Arc<Resources>,
    graphics_state: Arc<crate::GraphicsState>,
}

// FIXME
unsafe impl Send for Callback {}
unsafe impl Sync for Callback {}

impl egui_wgpu::CallbackTrait for Callback {
    fn paint<'a>(
        &'a self,
        _info: egui::PaintCallbackInfo,
        render_pass: &mut wgpu::RenderPass<'a>,
        _callback_resources: &'a egui_wgpu::CallbackResources,
    ) {
        self.resources.viewport.bind(1, render_pass);
        self.resources
            .sprite
            .draw(&self.graphics_state, &self.resources.viewport, render_pass);
    }
}

impl Event {
    // code smell, fix
    pub fn new(
        graphics_state: &crate::GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        event: &luminol_data::rpg::Event,
        atlas: &crate::tiles::Atlas,
        use_push_constants: bool,
    ) -> anyhow::Result<Option<Self>> {
        let Some(page) = event.pages.first() else {
            anyhow::bail!("event does not have first page");
        };

        let texture = if let Some(ref filename) = page.graphic.character_name {
            graphics_state.image_cache.load_wgpu_image(
                graphics_state,
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

            let viewport = crate::viewport::Viewport::new(
                graphics_state,
                glam::Mat4::orthographic_rh(0.0, 32., 32., 0., -1., 1.),
                use_push_constants,
            );

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
            let quad = crate::quad::Quad::new(pos, tex_coords, 0.0);

            let viewport = crate::viewport::Viewport::new(
                graphics_state,
                glam::Mat4::orthographic_rh(0.0, cw, ch, 0., -1., 1.),
                use_push_constants,
            );

            (quad, viewport, egui::vec2(cw, ch))
        };

        let sprite = crate::sprite::Sprite::new(
            graphics_state,
            quads,
            texture,
            page.graphic.blend_type,
            page.graphic.character_hue,
            page.graphic.opacity,
            use_push_constants,
        );

        Ok(Some(Self {
            resources: Arc::new(Resources { sprite, viewport }),
            sprite_size,
        }))
    }

    pub fn sprite(&self) -> &crate::sprite::Sprite {
        &self.resources.sprite
    }

    pub fn set_proj(&self, render_state: &egui_wgpu::RenderState, proj: glam::Mat4) {
        self.resources.viewport.set_proj(render_state, proj);
    }

    pub fn paint(
        &self,
        graphics_state: Arc<crate::GraphicsState>,
        painter: &egui::Painter,
        rect: egui::Rect,
    ) {
        painter.add(egui_wgpu::Callback::new_paint_callback(
            rect,
            Callback {
                resources: self.resources.clone(),
                graphics_state,
            },
        ));
    }
}
