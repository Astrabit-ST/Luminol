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

use crate::prelude::*;

#[derive(Debug)]
pub struct Events {
    events: Vec<Event>,
}

impl Events {
    pub fn new(map: &rpg::Map, atlas: &Atlas) -> Result<Self, String> {
        let mut events: Vec<_> = map
            .events
            .iter()
            .map(|(_, event)| Event::new(event, atlas))
            .flatten_ok()
            .try_collect()?;
        events.sort_unstable_by(|e1, e2| e1.sprite.blend_mode.cmp(&e2.sprite.blend_mode));

        Ok(Self { events })
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        render_pass.push_debug_group("tilemap event renderer");
        for event in self.events.iter() {
            event.draw(render_pass);
        }
        render_pass.pop_debug_group();
    }
}

#[derive(Debug)]
struct Event {
    sprite: Sprite,
}

impl Event {
    // code smell, fix
    pub fn new(event: &rpg::Event, atlas: &Atlas) -> Result<Option<Self>, String> {
        let Some(page) = event.pages.first() else {
            return Err("event does not have first page".to_string())
        };

        let texture = if let Some(ref filename) = page.graphic.character_name {
            state!()
                .image_cache
                .load_wgpu_image("Graphics/Characters", filename)?
        } else if page.graphic.tile_id > 0 {
            atlas.atlas_texture.clone()
        } else {
            return Ok(None);
        };

        let quads = if page.graphic.tile_id > 0 {
            let mut tile_quads = vec![];
            atlas.calc_quads(
                page.graphic.tile_id as i16,
                event.x as usize,
                event.y as usize,
                &mut tile_quads,
            );
            tile_quads
        } else {
            let cw = texture.width() as f32 / 4.;
            let ch = texture.height() as f32 / 4.;
            let pos = egui::Rect::from_min_size(
                egui::pos2(
                    (event.x as f32 * 32.) + (16. - (cw / 2.)),
                    (event.y as f32 * 32.) + (32. - ch),
                ),
                egui::vec2(cw, ch),
            );

            let tex_coords = egui::Rect::from_min_size(
                egui::pos2(
                    page.graphic.pattern as f32 * cw,
                    (page.graphic.direction as f32 - 2.) / 2. * ch,
                ),
                egui::vec2(cw, ch),
            );
            let quad = Quad::new(pos, tex_coords, 0.0);

            vec![quad]
        };

        let sprite = Sprite::new(
            &quads,
            texture,
            page.graphic.blend_type.try_into()?,
            page.graphic.character_hue,
            page.graphic.opacity,
        );

        Ok(Some(Self { sprite }))
    }

    pub fn draw<'rpass>(&'rpass self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.sprite.draw(render_pass);
    }
}
