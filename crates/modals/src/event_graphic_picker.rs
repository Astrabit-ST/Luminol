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

use color_eyre::eyre::Context;
use egui::Widget;
use luminol_components::UiExt;
use luminol_core::prelude::*;

pub struct Modal {
    entries: Vec<Entry>,
    filtered_entries: Vec<Entry>,
    search_text: String,

    open: bool,
    id_source: egui::Id,

    selected: Selected,
    opacity: i32,
    hue: i32,
    blend_mode: rpg::BlendMode,
    first_open: bool,

    tilepicker: Tilepicker,

    button_viewport: Viewport,
    button_sprite: Option<Event>,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone)]
struct Entry {
    path: camino::Utf8PathBuf,
    invalid: bool,
}

struct PreviewSprite {
    sprite: Sprite,
    sprite_size: egui::Vec2,
    viewport: Viewport,
}

#[derive(Default)]
enum Selected {
    #[default]
    None,
    Tile(usize),
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
        tileset: &rpg::Tileset,
        id_source: egui::Id,
    ) -> Self {
        // TODO error handling
        let mut entries: Vec<_> = update_state
            .filesystem
            .read_dir("Graphics/Characters")
            .unwrap()
            .into_iter()
            .map(|m| {
                let path = m
                    .path
                    .strip_prefix("Graphics/Characters")
                    .unwrap_or(&m.path)
                    .with_extension("");
                Entry {
                    path,
                    invalid: false,
                }
            })
            .collect();
        entries.sort_unstable();

        let mut tilepicker = Tilepicker::new(
            &update_state.graphics,
            tileset,
            update_state.filesystem,
            true,
        )
        .unwrap();
        tilepicker.tiles.auto_opacity = false;

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
            filtered_entries: entries.clone(),
            search_text: String::new(),
            entries,

            open: false,
            id_source,

            selected: Selected::None,
            opacity: graphic.opacity,
            hue: graphic.character_hue,
            blend_mode: graphic.blend_type,
            first_open: false,

            tilepicker,

            button_viewport,
            button_sprite,
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
            let desired_size = egui::vec2(64., 96.) + ui.spacing().button_padding * 2.;
            let (rect, mut response) = ui.allocate_at_least(desired_size, egui::Sense::click());

            let visuals = ui.style().interact_selectable(&response, self.open);
            let rect = rect.expand(visuals.expansion);
            ui.painter()
                .rect(rect, visuals.rounding, visuals.bg_fill, visuals.bg_stroke);

            if let Some(sprite) = &mut self.button_sprite {
                let translation = (desired_size - sprite.sprite_size) / 2.;
                self.button_viewport.set(
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

            if response.clicked() {
                self.selected = if let Some(id) = data.tile_id {
                    Selected::Tile(id)
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
                self.blend_mode = data.blend_type;
                self.hue = data.character_hue;
                self.opacity = data.opacity;
                self.first_open = true;

                self.open = true;
            }
            if self.show_window(update_state, ui.ctx(), data) {
                response.mark_changed();
            }

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
        let mut keep_open = true;
        let mut needs_save = false;

        egui::Window::new("Event Graphic Picker")
            .resizable(true)
            .open(&mut self.open)
            .id(self.id_source.with("window"))
            .show(ctx, |ui| {
                egui::SidePanel::left(self.id_source.with("sidebar")).show_inside(ui, |ui| {
                    let out = egui::TextEdit::singleline(&mut self.search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                    if out.response.changed() {
                        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                        self.filtered_entries = self
                            .entries
                            .iter()
                            .filter(|entry| {
                                matcher
                                    .fuzzy(entry.path.as_str(), &self.search_text, false)
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
                                self.filtered_entries.len() + 2,
                                |ui, mut rows| {
                                    if rows.contains(&0) {
                                        let res = ui.selectable_label(matches!(self.selected, Selected::None), "(None)");
                                        if res.clicked() && !matches!(self.selected, Selected::None) {
                                            self.selected = Selected::None;
                                        }
                                    }

                                    if rows.contains(&1) {
                                        let checked = matches!(self.selected, Selected::Tile(_));
                                        ui.with_stripe(true, |ui| {
                                            let res =  ui.selectable_label(checked, "(Tileset)");
                                            if res.clicked() && !checked {
                                                self.selected = Selected::Tile(384);
                                            }
                                        });
                                    }

                                    // subtract 2 to account for (None) and (Tileset)
                                    rows.start = rows.start.saturating_sub(2);
                                    rows.end = rows.end.saturating_sub(2);

                                    for (i, Entry { path: entry ,invalid}) in self.filtered_entries[rows.clone()].iter_mut().enumerate() {
                                        let checked =
                                            matches!(self.selected, Selected::Graphic { ref path, .. } if path == entry);
                                        let mut text = egui::RichText::new(entry.as_str());
                                        if *invalid {
                                            text = text.color(egui::Color32::LIGHT_RED);
                                        }
                                        let faint = (i + rows.start) % 2 == 1;
                                        ui.with_stripe(faint, |ui| {
                                            let res = ui.add_enabled(!*invalid, egui::SelectableLabel::new(checked, text));

                                            if res.clicked() {
                                                let sprite = match Self::load_preview_sprite(update_state, entry, self.hue, self.opacity) {
                                                    Ok(sprite) => sprite,
                                                    Err(e) => {
                                                        luminol_core::error!(update_state.toasts, e);
                                                        *invalid = true; // FIXME update non-filtered entry too
                                                        return;
                                                    }
                                                };
                                                self.selected = Selected::Graphic { path: entry.clone(), direction: 2, pattern: 0, sprite };
                                            }
                                        });
                                    }
                            });
                    });
                });

                egui::TopBottomPanel::top(self.id_source.with("top")).show_inside(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Opacity");
                        if ui.add(egui::Slider::new(&mut self.opacity, 0..=255)).changed() {
                            self.tilepicker.tiles.display.set_opacity(&update_state.graphics.render_state, self.opacity as f32 / 255., 0);
                            if let Selected::Graphic { sprite,.. } = &mut self.selected {
                                sprite.sprite.graphic.set_opacity(&update_state.graphics.render_state, self.opacity);
                            }
                        }
                        ui.label("Hue");
                        if ui.add(egui::Slider::new(&mut self.hue, 0..=360)).changed() {
                            self.tilepicker.tiles.display.set_hue(&update_state.graphics.render_state, self.hue as f32 / 360.0, 0);
                            if let Selected::Graphic { sprite,.. } = &mut self.selected {
                                sprite.sprite.graphic.set_hue(&update_state.graphics.render_state, self.hue);
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Blend Mode");
                        luminol_components::EnumComboBox::new(self.id_source.with("blend_mode"), &mut self.blend_mode).ui(ui);
                    });
                });
                egui::TopBottomPanel::bottom(self.id_source.with("bottom")).show_inside(ui, |ui| {
                    ui.add_space(ui.style().spacing.item_spacing.y);
                    luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both().auto_shrink([false,false]).show_viewport(ui, |ui, viewport| {
                        match &mut self.selected {
                            Selected::None => {}
                            Selected::Graphic { direction, pattern, sprite, .. } => {
                                let (canvas_rect, response) = ui.allocate_exact_size(
                                    sprite.sprite_size,
                                    egui::Sense::click(),
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

                                let ch = sprite.sprite_size.y / 4.;
                                let cw = sprite.sprite_size.x / 4.;
                                let rect = egui::Rect::from_min_size(egui::pos2(cw ** pattern as f32, ch * (*direction as f32 - 2.) / 2.), egui::vec2(cw, ch)).translate(canvas_rect.min.to_vec2());
                                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

                                if response.clicked() {
                                    let pos = (response.interact_pointer_pos().unwrap() - response.rect.min) / egui::vec2(cw, ch);
                                    *direction = pos.y as i32 * 2 + 2;
                                    *pattern = pos.x as i32;
                                }
                            }
                            Selected::Tile(id) => {
                                let (canvas_rect, response) = ui.allocate_exact_size(
                                    egui::vec2(256., self.tilepicker.atlas.tileset_height as f32),
                                    egui::Sense::click(),
                                );

                                let absolute_scroll_rect = ui
                                    .ctx()
                                    .screen_rect()
                                    .intersect(viewport.translate(canvas_rect.min.to_vec2()));
                                let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());

                                self.tilepicker.grid.display.set_pixels_per_point(
                                    &update_state.graphics.render_state,
                                    ui.ctx().pixels_per_point(),
                                );

                                self.tilepicker.set_position(
                                    &update_state.graphics.render_state,
                                    glam::vec2(-scroll_rect.left(), -scroll_rect.top()),
                                );
                                self.tilepicker.viewport.set(
                                    &update_state.graphics.render_state,
                                    glam::vec2(scroll_rect.width(), scroll_rect.height()),
                                    glam::Vec2::ZERO,
                                    glam::Vec2::ONE,
                                );

                                self.tilepicker
                                    .update_animation(&update_state.graphics.render_state, ui.input(|i| i.time));

                                let painter = Painter::new(self.tilepicker.prepare(&update_state.graphics));
                                ui.painter()
                                    .add(luminol_egui_wgpu::Callback::new_paint_callback(
                                        absolute_scroll_rect,
                                        painter,
                                    ));

                                let tile_x = (*id - 384) % 8;
                                let tile_y = (*id - 384) / 8;
                                let rect = egui::Rect::from_min_size(egui::Pos2::new(tile_x as f32, tile_y as f32) * 32., egui::Vec2::splat(32.)).translate(canvas_rect.min.to_vec2());
                                ui.painter().rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

                                if response.clicked() {
                                    let pos = (response.interact_pointer_pos().unwrap() - response.rect.min) / 32.;
                                    *id = pos.x as usize + pos.y as usize * 8 + 384;
                                }
                            }
                        }
                    });
                });
            });

        self.first_open = false;

        if needs_save {
            match self.selected {
                Selected::None => {
                    data.tile_id = None;
                    data.character_name = None;
                }
                Selected::Tile(id) => {
                    data.tile_id = Some(id);
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
                    data.direction = direction;
                    data.pattern = pattern;
                }
            }
            data.blend_type = self.blend_mode;
            data.character_hue = self.hue;
            data.opacity = self.opacity;
            self.update_graphic(update_state, data);
        }

        if !keep_open {
            self.open = false;
        }

        needs_save
    }
}
