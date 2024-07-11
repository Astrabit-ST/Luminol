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

use color_eyre::eyre::Context;

use crate::{Atlas, GraphicsState, Quad, Renderable, Sprite, Transform, Viewport};

pub struct Event {
    pub sprite: Sprite,
    pub sprite_size: egui::Vec2,
}

impl Event {
    // code smell, fix
    pub fn new_map(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        viewport: &Viewport,
        event: &luminol_data::rpg::Event,
        atlas: &Atlas,
    ) -> color_eyre::Result<Option<Self>> {
        let Some(page) = event.pages.first() else {
            color_eyre::eyre::bail!("event does not have first page");
        };

        let mut is_placeholder = false;
        let texture = if let Some(ref filename) = page.graphic.character_name {
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Characters", filename)
                .wrap_err_with(|| format!("Error loading event character graphic {filename:?}"));
            match texture {
                Ok(t) => t,
                Err(e) => {
                    graphics_state.send_texture_error(e);
                    is_placeholder = true;
                    graphics_state.texture_loader.placeholder_texture()
                }
            }
        } else if page.graphic.tile_id.is_some() {
            atlas.atlas_texture.clone()
        } else {
            return Ok(None);
        };

        let (quad, sprite_size) = if let Some(id) = page.graphic.tile_id {
            // Why does this have to be + 1?
            let quad = atlas.calc_quad((id + 1) as i16);

            (quad, egui::vec2(32., 32.))
        } else if is_placeholder {
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(32., 32.0));
            let quad = Quad::new(rect, rect);

            (quad, egui::vec2(32., 32.))
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
            let quad = Quad::new(pos, tex_coords);

            (quad, egui::vec2(cw, ch))
        };

        let x = event.x as f32 * 32. + (32. - sprite_size.x) / 2.;
        let y = event.y as f32 * 32. + (32. - sprite_size.y);
        let transform = Transform::new_position(graphics_state, glam::vec2(x, y));

        let sprite = Sprite::new(
            graphics_state,
            quad,
            page.graphic.character_hue,
            page.graphic.opacity,
            page.graphic.blend_type,
            &texture,
            viewport,
            transform,
        );

        Ok(Some(Self {
            sprite,
            sprite_size,
        }))
    }

    pub fn new_standalone(
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        viewport: &Viewport,
        graphic: &luminol_data::rpg::Graphic,
        atlas: &Atlas,
    ) -> color_eyre::Result<Option<Self>> {
        let mut is_placeholder = false;
        let texture = if let Some(ref filename) = graphic.character_name {
            let texture = graphics_state
                .texture_loader
                .load_now_dir(filesystem, "Graphics/Characters", filename)
                .wrap_err_with(|| format!("Error loading event character graphic {filename:?}"));
            match texture {
                Ok(t) => t,
                Err(e) => {
                    graphics_state.send_texture_error(e);
                    is_placeholder = true;
                    graphics_state.texture_loader.placeholder_texture()
                }
            }
        } else if graphic.tile_id.is_some() {
            atlas.atlas_texture.clone()
        } else {
            return Ok(None);
        };

        let (quad, sprite_size) = if let Some(id) = graphic.tile_id {
            // Why does this have to be + 1?
            let quad = atlas.calc_quad((id + 1) as i16);

            (quad, egui::vec2(32., 32.))
        } else if is_placeholder {
            let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(32., 32.0));
            let quad = Quad::new(rect, rect);

            (quad, egui::vec2(32., 32.))
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
                    graphic.pattern as f32 * cw + 0.01,
                    (graphic.direction as f32 - 2.) / 2. * ch + 0.01,
                ),
                egui::vec2(cw - 0.02, ch - 0.02),
            );
            let quad = Quad::new(pos, tex_coords);

            (quad, egui::vec2(cw, ch))
        };

        let transform = Transform::unit(graphics_state);

        let sprite = Sprite::new(
            graphics_state,
            quad,
            graphic.character_hue,
            graphic.opacity,
            graphic.blend_type,
            &texture,
            viewport,
            transform,
        );

        Ok(Some(Self {
            sprite,
            sprite_size,
        }))
    }

    pub fn set_position(&mut self, render_state: &luminol_egui_wgpu::RenderState, x: i32, y: i32) {
        let x = x as f32 * 32. + (32. - self.sprite_size.x) / 2.;
        let y = y as f32 * 32. + (32. - self.sprite_size.y);
        self.sprite
            .transform
            .set_position(render_state, glam::vec2(x, y));
    }

    pub fn sprite(&self) -> &Sprite {
        &self.sprite
    }
}

impl Renderable for Event {
    type Prepared = <Sprite as Renderable>::Prepared;

    fn prepare(&mut self, graphics_state: &std::sync::Arc<GraphicsState>) -> Self::Prepared {
        self.sprite.prepare(graphics_state)
    }
}
