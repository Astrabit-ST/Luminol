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
            let button_text = match data {
                rpg::Graphic {
                    character_name: Some(name),
                    ..
                } => name.to_string(),
                rpg::Graphic {
                    tile_id: Some(id), ..
                } => format!("Tile {id}"),
                _ => "None".to_string(),
            };

            let button_response = ui.button(button_text);
            if button_response.clicked() {
                self.open = true;
            }
            self.show_window(update_state, ui.ctx(), data);

            button_response
        }
    }

    fn reset(&mut self) {
        self.open = false;
    }
}

impl Modal {
    pub fn update_graphic(&mut self, update_state: &UpdateState<'_>, graphic: &rpg::Graphic) {}

    fn show_window(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut rpg::Graphic,
    ) {
    }
}
