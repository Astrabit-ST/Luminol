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

    button_sprite: Option<ButtonSprite>,
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

struct ButtonSprite {
    sprite: Sprite,
    sprite_size: egui::Vec2,
    viewport: Viewport,
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
        id_source: impl Into<egui::Id>,
    ) -> Self {
        let button_sprite = path.map(|path| {
            let texture = update_state
                .graphics
                .texture_loader
                .load_now_dir(update_state.filesystem, &directory, path)
                .unwrap(); // FIXME

            let button_viewport = Viewport::new(&update_state.graphics, Default::default());
            let sprite = Sprite::basic(&update_state.graphics, &texture, &button_viewport);
            ButtonSprite {
                sprite,
                sprite_size: texture.size_vec2(),
                viewport: button_viewport,
            }
        });

        Self {
            state: State::Closed,
            id_source: id_source.into(),
            button_size,
            directory,
            button_sprite,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data = Option<camino::Utf8PathBuf>;

    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        |ui: &mut egui::Ui| {
            let desired_size = self.button_size + ui.spacing().button_padding * 2.0;
            let (rect, mut response) = ui.allocate_at_least(desired_size, egui::Sense::click());

            let is_open = matches!(self.state, State::Open { .. });
            let visuals = ui.style().interact_selectable(&response, is_open);
            let rect = rect.expand(visuals.expansion);
            ui.painter()
                .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

            if let Some(ButtonSprite {
                sprite,
                sprite_size,
                viewport,
            }) = &mut self.button_sprite
            {
                let translation = (desired_size - *sprite_size) / 2.;
                viewport.set(
                    &update_state.graphics.render_state,
                    glam::vec2(desired_size.x, desired_size.y),
                    glam::vec2(translation.x, translation.y),
                    glam::Vec2::ONE,
                );
                let callback = luminol_egui_wgpu::Callback::new_paint_callback(
                    response.rect,
                    Painter::new(sprite.prepare(&update_state.graphics)),
                );
                ui.painter().add(callback);
            }

            if response.clicked() && !is_open {
                let selected = match data.clone() {
                    Some(path) => todo!(),
                    None => Selected::None,
                };

                // FIXME error handling
                let mut entries: Vec<_> = update_state
                    .filesystem
                    .read_dir(&self.directory)
                    .unwrap()
                    .into_iter()
                    .map(|m| {
                        let path = m
                            .path
                            .strip_prefix(&self.directory)
                            .unwrap_or(&m.path)
                            .with_extension("");
                        Entry {
                            path,
                            invalid: false,
                        }
                    })
                    .collect();
                entries.sort_unstable();

                self.state = State::Open {
                    filtered_entries: entries.clone(),
                    entries,
                    search_text: String::new(),
                    selected,
                };
            }
            if self.show_window(update_state, ui.ctx(), data) {
                response.mark_changed();
            }

            response
        }
    }

    fn reset(&mut self, update_state: &mut luminol_core::UpdateState<'_>, data: &Self::Data) {
        self.update_graphic(update_state, data); // we need to update the button sprite to prevent desyncs
        self.state = State::Closed;
    }
}

impl Modal {
    fn update_graphic(
        &mut self,
        update_state: &UpdateState<'_>,
        data: &Option<camino::Utf8PathBuf>,
    ) {
        self.button_sprite = data.as_ref().map(|path| {
            let texture = update_state
                .graphics
                .texture_loader
                .load_now_dir(update_state.filesystem, &self.directory, path)
                .unwrap(); // FIXME

            let button_viewport = Viewport::new(&update_state.graphics, Default::default());
            let sprite = Sprite::basic(&update_state.graphics, &texture, &button_viewport);
            ButtonSprite {
                sprite,
                sprite_size: texture.size_vec2(),
                viewport: button_viewport,
            }
        });
    }

    fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut Option<camino::Utf8PathBuf>,
    ) -> bool {
        false
    }
}
