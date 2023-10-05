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
use once_cell::sync::OnceCell;
use std::sync::Arc;

use crate::primitives;

#[derive(Debug)]
pub struct Event {
    resources: Arc<Resources>,
    pub sprite_size: egui::Vec2,
}

#[derive(Debug)]
struct Resources {
    sprite: primitives::Sprite,
    viewport: primitives::Viewport,
}

type ResourcesSlab = slab::Slab<Arc<Resources>>;

impl Event {
    // code smell, fix
    pub fn new(
        event: &luminol_data::rpg::Event,
        atlas: &primitives::Atlas,
        use_push_constants: bool,
    ) -> Result<Option<Self>, String> {
        let Some(page) = event.pages.first() else {
            return Err("event does not have first page".to_string());
        };

        let texture = if let Some(ref filename) = page.graphic.character_name {
            state!()
                .image_cache
                .load_wgpu_image("Graphics/Characters", filename)?
        } else if page.graphic.tile_id.is_some() {
            atlas.atlas_texture.clone()
        } else {
            return Ok(None);
        };

        let (quads, viewport, sprite_size) = if let Some(id) = page.graphic.tile_id {
            // Why does this have to be + 1?
            let quad = atlas.calc_quad((id + 1) as i16);

            let viewport = primitives::Viewport::new(
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
            let quad = primitives::Quad::new(pos, tex_coords, 0.0);

            let viewport = primitives::Viewport::new(
                glam::Mat4::orthographic_rh(0.0, cw, ch, 0., -1., 1.),
                use_push_constants,
            );

            (quad, viewport, egui::vec2(cw, ch))
        };

        let sprite = primitives::Sprite::new(
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

    pub fn sprite(&self) -> &primitives::Sprite {
        &self.resources.sprite
    }

    pub fn set_proj(&self, proj: glam::Mat4) {
        self.resources.viewport.set_proj(proj);
    }

    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        let resources = self.resources.clone();
        let resource_id = Arc::new(OnceCell::new());

        let prepare_id = resource_id;
        let paint_id = prepare_id.clone();
        let callback = egui_wgpu::CallbackFn::new()
            .prepare(move |_device, _queue, _encoder, paint_callback_resources| {
                let res_hash: &mut ResourcesSlab = paint_callback_resources
                    .entry()
                    .or_insert_with(Default::default);
                let id = res_hash.insert(resources.clone());
                prepare_id.set(id).expect("resources id already set?");
                vec![]
            })
            .paint(move |_info, render_pass, paint_callback_resources| {
                let res_hash: &ResourcesSlab = paint_callback_resources.get().unwrap();
                let id = paint_id.get().copied().expect("resources id is unset");
                let resources = &res_hash[id];
                let Resources { viewport, sprite } = resources.as_ref();

                viewport.bind(render_pass);
                sprite.draw(viewport, render_pass);
            });
        painter.add(egui::PaintCallback {
            callback: Arc::new(callback),
            rect,
        });
    }
}
