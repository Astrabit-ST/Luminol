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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use color_eyre::eyre::Context;
use egui::Widget;
use luminol_components::UiExt;
use luminol_core::prelude::*;

pub struct Modal {
    state: State,
    id_source: egui::Id,

    button_size: egui::Vec2,
    directory: camino::Utf8PathBuf, // do we make this &'static Utf8Path?

    button_viewport: Viewport,
    button_sprite: Option<Sprite>,
}

enum State {
    Closed,
    Open {
        entries: Vec<Entry>,
        filtered_entries: Vec<Entry>,
        search_text: String,

        selected: Selected,
    },
}

#[derive(Default)]
enum Selected {
    #[default]
    None,
    Entry {
        path: camino::Utf8PathBuf,
        sprite: PreviewSprite,
    },
}

struct PreviewSprite {
    sprite: Sprite,
    sprite_size: egui::Vec2,
    viewport: Viewport,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
struct Entry {
    path: camino::Utf8PathBuf,
    invalid: bool,
}

impl Modal {
    pub fn new(
        update_state: &UpdateState<'_>,
        directory: camino::Utf8PathBuf,
        path: Option<&camino::Utf8Path>,
        button_size: egui::Vec2,
        id_source: egui::Id,
    ) -> Self {
        let button_viewport = Viewport::new(&update_state.graphics, Default::default());
        let button_sprite = path.map(|path| {
            let texture = update_state
                .graphics
                .texture_loader
                .load_now_dir(update_state.filesystem, &directory, path)
                .unwrap(); // FIXME

            Sprite::basic(&update_state.graphics, &texture, &button_viewport)
        });

        Self {
            state: State::Closed,
            id_source,
            button_size,
            directory,
            button_viewport,
            button_sprite,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data = camino::Utf8PathBuf;

    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| todo!()
    }

    fn reset(&mut self, update_state: &mut luminol_core::UpdateState<'_>, data: &Self::Data) {
        todo!()
    }
}
