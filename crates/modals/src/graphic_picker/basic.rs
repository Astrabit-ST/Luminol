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
    type Data<'m> = &'m mut Option<camino::Utf8PathBuf>;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
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
                    Some(path) => {
                        // FIXME error handling
                        let sprite =
                            Self::load_preview_sprite(update_state, &self.directory, &path)
                                .unwrap();
                        Selected::Entry { path, sprite }
                    }
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

    fn reset(&mut self, update_state: &mut luminol_core::UpdateState<'_>, data: Self::Data<'_>) {
        self.update_graphic(update_state, data); // we need to update the button sprite to prevent desyncs
        self.state = State::Closed;
    }
}

impl Modal {
    fn load_preview_sprite(
        update_state: &luminol_core::UpdateState<'_>,
        directory: &camino::Utf8Path,
        path: &camino::Utf8Path,
    ) -> color_eyre::Result<PreviewSprite> {
        let texture = update_state
            .graphics
            .texture_loader
            .load_now_dir(update_state.filesystem, directory, path)
            .wrap_err("While loading a preview sprite")?;

        Ok(Self::create_preview_sprite_from_texture(
            update_state,
            &texture,
        ))
    }

    fn create_preview_sprite_from_texture(
        update_state: &luminol_core::UpdateState<'_>,
        texture: &Texture,
    ) -> PreviewSprite {
        let viewport = Viewport::new(
            &update_state.graphics,
            glam::vec2(texture.width() as f32, texture.height() as f32),
        );

        let sprite = Sprite::basic(&update_state.graphics, texture, &viewport);
        PreviewSprite {
            sprite,
            sprite_size: texture.size_vec2(),
            viewport,
        }
    }

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
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open {
            entries,
            filtered_entries,
            search_text,
            selected,
        } = &mut self.state
        else {
            return false;
        };

        egui::Window::new("Graphic Picker")
            .resizable(true)
            .open(&mut win_open)
            .id(self.id_source.with("window"))
            .show(ctx, |ui| {
                egui::SidePanel::left(self.id_source.with("sidebar")).show_inside(ui, |ui| {
                    let out = egui::TextEdit::singleline(search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                    if out.response.changed() {
                        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                        *filtered_entries = entries
                            .iter()
                            .filter(|entry| {
                                matcher
                                    .fuzzy(entry.path.as_str(), search_text, false)
                                    .is_some()
                            })
                            .cloned()
                            .collect();
                    }

                    ui.separator();

                    // Get row height.
                    let row_height = ui.text_style_height(&egui::TextStyle::Body); // i do not trust this
                                                                                   // FIXME scroll to selected on first open
                    ui.with_cross_justify(|ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show_rows(
                                ui,
                                row_height,
                                filtered_entries.len() + 1,
                                |ui, mut rows| {
                                    if rows.contains(&0) {
                                        let res = ui.selectable_label(
                                            matches!(selected, Selected::None),
                                            "(None)",
                                        );
                                        if res.clicked() && !matches!(selected, Selected::None) {
                                            *selected = Selected::None;
                                        }
                                    }

                                    // subtract 2 to account for (None)
                                    rows.start = rows.start.saturating_sub(1);
                                    rows.end = rows.end.saturating_sub(1);

                                    for i in filtered_entries[rows.clone()].iter_mut().enumerate() {
                                        let (i, Entry { path, invalid }) = i;
                                        let checked = matches!(selected, Selected::Entry { path: p, .. } if p == path);
                                        let mut text = egui::RichText::new(path.as_str());
                                        if *invalid {
                                            text = text.color(egui::Color32::LIGHT_RED);
                                        }
                                        let faint = (i + rows.start) % 2 == 1;
                                        ui.with_stripe(faint, |ui| {
                                            let res = ui.add_enabled(!*invalid, egui::SelectableLabel::new(checked, text));

                                            if res.clicked() {
                                                let sprite = Self::load_preview_sprite(update_state, &self.directory, path).unwrap();
                                                *selected = Selected::Entry { path:path.clone(), sprite };
                                            }
                                        });
                                    }
                                },
                            );
                    });
                });

                egui::TopBottomPanel::bottom(self.id_source.with("bottom")).show_inside(ui, |ui| {
                    ui.add_space(ui.style().spacing.item_spacing.y);
                    luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both().auto_shrink([false,false]).show_viewport(ui, |ui, viewport| {
                        match selected {
                            Selected::None => {}
                            Selected::Entry {  sprite,.. } => {
                                let (canvas_rect, _) = ui.allocate_exact_size(
                                    sprite.sprite_size,
                                    egui::Sense::focusable_noninteractive(), // FIXME screen reader hints
                                );

                                let absolute_scroll_rect = ui
                                    .ctx()
                                    .screen_rect()
                                    .intersect(viewport.translate(canvas_rect.min.to_vec2()));
                                let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());
                                sprite.sprite.transform.set_position(
                                    &update_state.graphics.render_state,
                                    glam::vec2(-scroll_rect.left(), -scroll_rect.top()),
                                );

                                sprite.viewport.set(
                                    &update_state.graphics.render_state,
                                    glam::vec2(absolute_scroll_rect.width(), absolute_scroll_rect.height()),
                                    glam::Vec2::ZERO,
                                    glam::Vec2::ONE,
                                );

                                let painter = Painter::new(sprite.sprite.prepare(&update_state.graphics));
                                ui.painter()
                                    .add(luminol_egui_wgpu::Callback::new_paint_callback(
                                        absolute_scroll_rect,
                                        painter,
                                    ));
                            }
                        }
                    });
                });
            });

        if needs_save {
            match selected {
                Selected::None => *data = None,
                Selected::Entry { path, .. } => *data = Some(path.clone()),
            }

            self.update_graphic(update_state, data);
        }

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }

        needs_save
    }
}
