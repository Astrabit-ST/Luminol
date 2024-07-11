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

use super::{ButtonSprite, Entry, PreviewSprite};

pub struct Modal {
    state: State,
    id_source: egui::Id,

    tileset_id: usize,

    button_sprite: Option<ButtonSprite>,
}

enum State {
    Closed,
    Open {
        entries: Vec<Entry>,
        filtered_entries: Vec<Entry>,
        search_text: String,

        selected: Selected,

        opacity: i32,
        hue: i32,
        blend_mode: rpg::BlendMode,
    },
}

#[derive(Default)]
enum Selected {
    #[default]
    None,
    Tile {
        tile_id: usize,
        tilepicker: Tilepicker,
    },
    Graphic {
        path: camino::Utf8PathBuf,
        sprite: PreviewSprite,
        direction: i32,
        pattern: i32,
    },
}

// FIXME DEAR GOD THE FORMATTING
impl Modal {
    pub fn new(
        update_state: &UpdateState<'_>,
        graphic: &rpg::Graphic,
        tileset_id: usize,
        id_source: egui::Id,
    ) -> Self {
        let atlas = update_state.graphics.atlas_loader.get_expect(tileset_id); // atlas should be loaded by this point

        let viewport = Viewport::new(&update_state.graphics, Default::default());
        let button_sprite = Event::new_standalone(
            &update_state.graphics,
            update_state.filesystem,
            &viewport,
            graphic,
            &atlas,
        )
        .unwrap() // FIXME
        .map(|sprite| ButtonSprite {
            sprite: sprite.sprite,
            sprite_size: sprite.sprite_size,
            viewport,
        });

        Self {
            state: State::Closed,
            id_source,

            tileset_id,

            button_sprite,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data<'m> = &'m mut luminol_data::rpg::Graphic;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
        update_state: &'m mut UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        move |ui: &mut egui::Ui| {
            let desired_size = egui::vec2(64., 96.) + ui.spacing().button_padding * 2.;
            let is_open = matches!(self.state, State::Open { .. });
            let mut response = ButtonSprite::ui(
                self.button_sprite.as_mut(),
                ui,
                update_state,
                is_open,
                desired_size,
            );

            if response.clicked() && !is_open {
                let selected = if let Some(tile_id) = data.tile_id {
                    let tilepicker = Self::load_tilepicker(update_state, self.tileset_id).unwrap(); // TODO handle

                    Selected::Tile {
                        tile_id,
                        tilepicker,
                    }
                } else if let Some(path) = data.character_name.clone() {
                    let sprite = match Self::load_preview_sprite(
                        update_state,
                        &path,
                        data.character_hue,
                        data.opacity,
                    ) {
                        Ok(sprite) => sprite,
                        Err(e) => {
                            luminol_core::error!(update_state.toasts, e);
                            let placeholder =
                                update_state.graphics.texture_loader.placeholder_texture();
                            Self::create_preview_sprite_from_texture(
                                update_state,
                                &placeholder,
                                data.character_hue,
                                data.opacity,
                            )
                        }
                    };
                    Selected::Graphic {
                        path,
                        direction: data.direction,
                        pattern: data.pattern,
                        sprite,
                    }
                } else {
                    Selected::None
                };

                let entries = Entry::load(update_state, "Graphics/Characters".into());

                self.state = State::Open {
                    filtered_entries: entries.clone(),
                    entries,
                    search_text: String::new(),
                    selected,
                    opacity: data.opacity,
                    hue: data.character_hue,
                    blend_mode: data.blend_type,
                };
            }
            if self.show_window(update_state, ui.ctx(), data) {
                response.mark_changed();
            }

            response
        }
    }

    fn reset(&mut self, update_state: &mut UpdateState<'_>, data: Self::Data<'_>) {
        self.update_graphic(update_state, data); // we need to update the button sprite to prevent desyncs
        self.state = State::Closed;
    }
}

impl Modal {
    fn update_graphic(&mut self, update_state: &UpdateState<'_>, graphic: &rpg::Graphic) {
        let atlas = update_state
            .graphics
            .atlas_loader
            .get_expect(self.tileset_id); // atlas should be loaded by this point

        let viewport = Viewport::new(&update_state.graphics, Default::default());
        self.button_sprite = Event::new_standalone(
            &update_state.graphics,
            update_state.filesystem,
            &viewport,
            graphic,
            &atlas,
        )
        .unwrap() // FIXME
        .map(|sprite| ButtonSprite {
            sprite: sprite.sprite,
            sprite_size: sprite.sprite_size,
            viewport,
        });
    }

    fn load_tilepicker(
        update_state: &UpdateState<'_>,
        tileset_id: usize,
    ) -> color_eyre::Result<Tilepicker> {
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets.data[tileset_id];

        let mut tilepicker = Tilepicker::new(
            &update_state.graphics,
            tileset,
            update_state.filesystem,
            true,
        )?;
        tilepicker.tiles.auto_opacity = false;

        Ok(tilepicker)
    }

    fn load_preview_sprite(
        update_state: &luminol_core::UpdateState<'_>,
        path: &camino::Utf8Path,
        hue: i32,
        opacity: i32,
    ) -> color_eyre::Result<PreviewSprite> {
        let texture = update_state
            .graphics
            .texture_loader
            .load_now_dir(update_state.filesystem, "Graphics/Characters", path)
            .wrap_err("While loading a preview sprite")?;

        Ok(Self::create_preview_sprite_from_texture(
            update_state,
            &texture,
            hue,
            opacity,
        ))
    }

    fn create_preview_sprite_from_texture(
        update_state: &luminol_core::UpdateState<'_>,
        texture: &Texture,
        hue: i32,
        opacity: i32,
    ) -> PreviewSprite {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, texture.size_vec2());
        let quad = Quad::new(rect, rect);
        let viewport = Viewport::new(
            &update_state.graphics,
            glam::vec2(texture.width() as f32, texture.height() as f32),
        );

        let sprite = Sprite::new(
            &update_state.graphics,
            quad,
            hue,
            opacity,
            rpg::BlendMode::Normal,
            texture,
            &viewport,
            Transform::unit(&update_state.graphics),
        );
        PreviewSprite {
            sprite,
            sprite_size: texture.size_vec2(),
            viewport,
        }
    }

    fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut rpg::Graphic,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open {
            entries,
            filtered_entries,
            search_text,
            selected,
            opacity,
            hue,
            blend_mode,
        } = &mut self.state
        else {
            return false;
        };

        egui::Window::new("Event Graphic Picker")
            .resizable(true)
            .open(&mut win_open)
            .id(self.id_source.with("window"))
            .show(ctx, |ui| {
                egui::SidePanel::left(self.id_source.with("sidebar")).show_inside(ui, |ui| {
                    let out = egui::TextEdit::singleline(search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                    if out.response.changed() {
                        *filtered_entries = Entry::filter(entries, search_text);
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
                                filtered_entries.len() + 2,
                                |ui, mut rows| {
                                    if rows.contains(&0) {
                                        let res = ui.selectable_label(matches!(selected, Selected::None), "(None)");
                                        if res.clicked() && !matches!(selected, Selected::None) {
                                            *selected = Selected::None;
                                        }
                                    }

                                    if rows.contains(&1) {
                                        let checked = matches!(selected, Selected::Tile {..});
                                        ui.with_stripe(true, |ui| {
                                            let res =  ui.selectable_label(checked, "(Tileset)");
                                            if res.clicked() && !checked {
                                                let tilepicker = Self::load_tilepicker(update_state, self.tileset_id).unwrap(); // TODO handle
                                                *selected = Selected::Tile { tile_id: 384, tilepicker };
                                            }
                                        });
                                    }

                                    // subtract 2 to account for (None) and (Tileset)
                                    rows.start = rows.start.saturating_sub(2);
                                    rows.end = rows.end.saturating_sub(2);

                                    for (i, Entry { path ,invalid}) in filtered_entries[rows.clone()].iter_mut().enumerate() {
                                        let checked =
                                            matches!(selected, Selected::Graphic { path: p, .. } if p == path);
                                        let mut text = egui::RichText::new(path.as_str());
                                        if *invalid {
                                            text = text.color(egui::Color32::LIGHT_RED);
                                        }
                                        let faint = (i + rows.start) % 2 == 1;
                                        ui.with_stripe(faint, |ui| {
                                            let res = ui.add_enabled(!*invalid, egui::SelectableLabel::new(checked, text));

                                            if res.clicked() {
                                                let sprite = match Self::load_preview_sprite(update_state, path, *hue, *opacity) {
                                                    Ok(sprite) => sprite,
                                                    Err(e) => {
                                                        luminol_core::error!(update_state.toasts, e);
                                                        *invalid = true; // FIXME update non-filtered entry too
                                                        return;
                                                    }
                                                };
                                                *selected = Selected::Graphic { path: path.clone(), direction: 2, pattern: 0, sprite };
                                            }
                                        });
                                    }
                            });
                    });
                });

                egui::TopBottomPanel::top(self.id_source.with("top")).show_inside(ui, |ui| {
                    ui.add_space(1.0); // pad out the top
                    ui.horizontal(|ui| {
                        ui.label("Opacity");
                        if ui.add(egui::Slider::new(opacity, 0..=255)).changed() {
                            match selected {
                                Selected::Graphic { sprite,.. } => {
                                    sprite.sprite.graphic.set_opacity(&update_state.graphics.render_state, *opacity)
                                },
                                Selected::Tile { tilepicker,.. } => {
                                    tilepicker.tiles.display.set_opacity(&update_state.graphics.render_state, *opacity as f32 / 255., 0)
                                }
                                Selected::None => {}
                            }

                        }
                        ui.label("Hue");
                        if ui.add(egui::Slider::new(hue, 0..=360)).changed() {
                            match selected {
                                Selected::Graphic { sprite,.. } => {
                                    sprite.sprite.graphic.set_hue(&update_state.graphics.render_state, *hue)
                                },
                                Selected::Tile { tilepicker,.. } => {
                                    tilepicker.tiles.display.set_hue(&update_state.graphics.render_state, *hue as f32 / 360., 0)
                                }
                                Selected::None => {}
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Blend Mode");
                        luminol_components::EnumComboBox::new(self.id_source.with("blend_mode"), blend_mode).ui(ui);
                    });
                    ui.add_space(1.0); // pad out the bottom
                });
                egui::TopBottomPanel::bottom(self.id_source.with("bottom")).show_inside(ui, |ui| {
                    ui.add_space(ui.style().spacing.item_spacing.y);
                    luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both().auto_shrink([false,false]).show_viewport(ui, |ui, viewport| {
                        match selected {
                            Selected::None => {}
                            Selected::Graphic { direction, pattern, sprite, .. } => {
                                let response = sprite.ui(ui, viewport, update_state);

                                let ch = sprite.sprite_size.y / 4.;
                                let cw = sprite.sprite_size.x / 4.;

                                let min = egui::pos2(*pattern as f32 * cw, (*direction as f32 - 2.) * ch / 2.);
                                let size = egui::vec2(cw, ch);
                                let rect = egui::Rect::from_min_size(min, size).translate(response.rect.min.to_vec2());
                                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

                                if response.clicked() {
                                    let pos = (response.interact_pointer_pos().unwrap() - response.rect.min) / egui::vec2(cw, ch);
                                    *direction = pos.y as i32 * 2 + 2;
                                    *pattern = pos.x as i32;
                                }
                            }
                            Selected::Tile { tile_id, tilepicker } => {
                                let (canvas_rect, response) = ui.allocate_exact_size(
                                    egui::vec2(256., tilepicker.atlas.tileset_height as f32),
                                    egui::Sense::click(),
                                );

                                let absolute_scroll_rect = ui
                                    .ctx()
                                    .screen_rect()
                                    .intersect(viewport.translate(canvas_rect.min.to_vec2()));
                                let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());

                                tilepicker.grid.display.set_pixels_per_point(
                                    &update_state.graphics.render_state,
                                    ui.ctx().pixels_per_point(),
                                );

                                tilepicker.set_position(
                                    &update_state.graphics.render_state,
                                    glam::vec2(-scroll_rect.left(), -scroll_rect.top()),
                                );
                                tilepicker.viewport.set(
                                    &update_state.graphics.render_state,
                                    glam::vec2(scroll_rect.width(), scroll_rect.height()),
                                    glam::Vec2::ZERO,
                                    glam::Vec2::ONE,
                                );

                                tilepicker
                                    .update_animation(&update_state.graphics.render_state, ui.input(|i| i.time));

                                let painter = Painter::new(tilepicker.prepare(&update_state.graphics));
                                ui.painter()
                                    .add(luminol_egui_wgpu::Callback::new_paint_callback(
                                        absolute_scroll_rect,
                                        painter,
                                    ));

                                let tile_x = (*tile_id - 384) % 8;
                                let tile_y = (*tile_id - 384) / 8;
                                let rect = egui::Rect::from_min_size(egui::Pos2::new(tile_x as f32, tile_y as f32) * 32., egui::Vec2::splat(32.)).translate(canvas_rect.min.to_vec2());
                                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

                                if response.clicked() {
                                    let pos = (response.interact_pointer_pos().unwrap() - response.rect.min) / 32.;
                                    *tile_id = pos.x as usize + pos.y as usize * 8 + 384;
                                }
                            }
                        }
                    });
                });
            });

        if needs_save {
            match selected {
                Selected::None => {
                    data.tile_id = None;
                    data.character_name = None;
                }
                Selected::Tile { tile_id, .. } => {
                    data.tile_id = Some(*tile_id);
                    data.character_name = None;
                }
                Selected::Graphic {
                    ref path,
                    direction,
                    pattern,
                    ..
                } => {
                    data.tile_id = None;
                    data.character_name = Some(path.clone());
                    data.direction = *direction;
                    data.pattern = *pattern;
                }
            }
            data.blend_type = *blend_mode;
            data.character_hue = *hue;
            data.opacity = *opacity;
            self.update_graphic(update_state, data);
        }

        if !(win_open && keep_open) {
            self.state = State::Closed;
        }

        needs_save
    }
}
