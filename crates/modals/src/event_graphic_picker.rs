// Copyright (C) 2024 Lily Lyons
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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use luminol_core::prelude::*;

pub struct Modal {
    entries: Vec<camino::Utf8PathBuf>,
    open: bool,
    id_source: egui::Id,

    tilepicker: Tilepicker,

    button_viewport: Viewport,
    button_sprite: Option<Event>,

    sprite: Option<(Viewport, Event)>,
}

impl Modal {
    pub fn new(
        update_state: &UpdateState<'_>,
        graphic: &rpg::Graphic,
        tileset: &rpg::Tileset,
        id_source: egui::Id,
    ) -> Self {
        // TODO error handling
        let entries = update_state
            .filesystem
            .read_dir("Graphics/Characters")
            .unwrap()
            .into_iter()
            .map(|m| {
                m.path
                    .strip_prefix("Graphics/Characters")
                    .unwrap_or(&m.path)
                    .with_extension("")
            })
            .collect();

        let tilepicker = Tilepicker::new(
            &update_state.graphics,
            tileset,
            update_state.filesystem,
            false,
        )
        .unwrap();

        let button_viewport = Viewport::new(&update_state.graphics, Default::default());
        let button_sprite = Event::new_standalone(
            &update_state.graphics,
            update_state.filesystem,
            &button_viewport,
            graphic,
            &tilepicker.atlas,
        )
        .unwrap();

        Self {
            entries,
            open: false,
            id_source,

            tilepicker,

            button_viewport,
            button_sprite,

            sprite: None,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data = luminol_data::rpg::Graphic;

    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        move |ui: &mut egui::Ui| {
            let desired_size = self
                .button_sprite
                .as_ref()
                .map(|s| s.sprite_size)
                .unwrap_or(egui::vec2(32., 32.));
            let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::click());

            if let Some(sprite) = &mut self.button_sprite {
                self.button_viewport.set_size(
                    &update_state.graphics.render_state,
                    glam::vec2(desired_size.x, desired_size.y),
                );
                let callback = luminol_egui_wgpu::Callback::new_paint_callback(
                    response.rect,
                    Painter::new(sprite.prepare(&update_state.graphics)),
                );
                painter.add(callback);
            }

            if response.clicked() {
                self.open = true;
            }
            self.show_window(update_state, ui.ctx(), data);

            response
        }
    }

    fn reset(&mut self) {
        self.open = false;
    }
}

impl Modal {
    pub fn update_graphic(&mut self, update_state: &UpdateState<'_>, graphic: &rpg::Graphic) {
        self.button_sprite = Event::new_standalone(
            &update_state.graphics,
            update_state.filesystem,
            &self.button_viewport,
            graphic,
            &self.tilepicker.atlas,
        )
        .unwrap();
        self.sprite = None;
    }

    fn show_window(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut rpg::Graphic,
    ) {
    }
}
