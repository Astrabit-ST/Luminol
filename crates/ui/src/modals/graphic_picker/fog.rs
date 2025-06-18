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

use super::PreviewSprite;
use crate::components::{EnumComboBox, UiExt};
use luminol_core::prelude::*;

use super::Entry;

pub struct Modal {
    state: State,
    id_source: egui::Id,
    scrolled_on_first_open: bool,
}

enum State {
    Closed,
    Open {
        entries: Vec<Entry>,
        filtered_entries: Vec<Entry>,
        search_text: String,
        sprite: Option<PreviewSprite>,
        fog_name: Option<camino::Utf8PathBuf>,
        fog_hue: i32,
        fog_opacity: i32,
        fog_blend_type: luminol_data::BlendMode,
        fog_zoom: i32,
        fog_sx: i32,
        fog_sy: i32,
    },
}

impl Modal {
    pub fn new(id_source: egui::Id) -> Self {
        Self {
            state: State::Closed,
            id_source,
            scrolled_on_first_open: false,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data<'m> = &'m mut luminol_data::rpg::Tileset;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
        update_state: &'m mut UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        move |ui: &mut egui::Ui| {
            let is_open = matches!(self.state, State::Open { .. });

            let button_text = if let Some(name) = &data.fog_name.0 {
                format!("Graphics/Fogs/{name}")
            } else {
                "(None)".to_string()
            };
            let mut response = ui.button(button_text);

            if response.clicked() && !is_open {
                let entries = Entry::load(update_state, "Graphics/Fogs".into());

                let desensitized_fog_name = data.fog_name.0.as_ref().and_then(|name| {
                    update_state
                        .filesystem
                        .desensitize(format!("Graphics/Fogs/{name}"))
                        .ok()
                        .map(|path| camino::Utf8PathBuf::from(path.file_name().unwrap_or_default()))
                });

                let sprite = desensitized_fog_name.as_ref().and_then(|fog_name| {
                    let texture = update_state
                        .graphics
                        .texture_loader
                        .load_now_dir(update_state.filesystem, "Graphics/Fogs", fog_name)
                        .map_err(|e| luminol_core::error!(update_state.toasts, e))
                        .ok()?;
                    let viewport = Viewport::new(&update_state.graphics, Default::default());
                    let sprite = Sprite::basic_hue(
                        &update_state.graphics,
                        data.fog_hue,
                        &texture,
                        &viewport,
                    );
                    Some(PreviewSprite {
                        sprite,
                        sprite_size: texture.size_vec2(),
                        viewport,
                    })
                });

                self.state = State::Open {
                    filtered_entries: entries.clone(),
                    entries,
                    sprite,
                    search_text: String::new(),
                    fog_name: data.fog_name.0.clone(),
                    fog_hue: data.fog_hue,
                    fog_opacity: data.fog_opacity,
                    fog_blend_type: data.fog_blend_type,
                    fog_zoom: data.fog_zoom,
                    fog_sx: data.fog_sx,
                    fog_sy: data.fog_sy,
                };
            }
            if self.show_window(update_state, ui.ctx(), data) {
                response.mark_changed();
            }

            response
        }
    }

    fn reset(&mut self, _update_state: &mut UpdateState<'_>, _data: Self::Data<'_>) {
        self.state = State::Closed;
        self.scrolled_on_first_open = false;
    }
}

impl Modal {
    fn show_window(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ctx: &egui::Context,
        data: &mut luminol_data::rpg::Tileset,
    ) -> bool {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open {
            entries,
            filtered_entries,
            search_text,
            sprite,
            fog_name,
            fog_hue,
            fog_opacity,
            fog_blend_type,
            fog_zoom,
            fog_sx,
            fog_sy,
        } = &mut self.state
        else {
            self.scrolled_on_first_open = false;
            return false;
        };

        let desensitized_fog_name = fog_name.as_ref().and_then(|name| {
            update_state
                .filesystem
                .desensitize(format!("Graphics/Fogs/{name}"))
                .ok()
                .map(|path| camino::Utf8PathBuf::from(path.file_name().unwrap_or_default()))
        });

        egui::Window::new("Fog Graphic Picker")
            .min_width(640.)
            .default_size([640., 480.])
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
                    let row_height = ui.spacing().interact_size.y.max(
                        ui.text_style_height(&egui::TextStyle::Button)
                            + 2. * ui.spacing().button_padding.y,
                    );
                    ui.with_cross_justify(|ui| {
                        let mut scroll_area_output = egui::ScrollArea::vertical()
                            .auto_shrink([false, true])
                            .show_rows(
                                ui,
                                row_height,
                                filtered_entries.len() + 1,
                                |ui, mut rows| {
                                    ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Truncate);

                                    if rows.contains(&0) {
                                        let checked = fog_name.is_none();
                                        let res = ui.selectable_label(checked, "(None)");
                                        if res.clicked() && fog_name.is_some() {
                                            *fog_name = None;
                                            *sprite = None;
                                        }
                                    }

                                    // subtract 1 to account for (None)
                                    rows.start = rows.start.saturating_sub(1);
                                    rows.end = rows.end.saturating_sub(1);

                                    for (i, Entry { path, invalid }) in
                                        filtered_entries[rows.clone()].iter_mut().enumerate()
                                    {
                                        let checked = desensitized_fog_name.as_ref() == Some(path);
                                        let mut text = egui::RichText::new(path.as_str());
                                        if *invalid {
                                            text = text.color(egui::Color32::LIGHT_RED);
                                        }
                                        let faint = (i + rows.start) % 2 == 0;
                                        ui.with_stripe(faint, |ui| {
                                            let res = ui.add_enabled(
                                                !*invalid,
                                                egui::SelectableLabel::new(checked, text),
                                            );

                                            if res.clicked() {
                                                *fog_name = Some(
                                                    path.file_stem()
                                                        .unwrap_or(path.as_str())
                                                        .into(),
                                                );

                                                let fog_name = fog_name.as_ref().and_then(|name| {
                                                    update_state
                                                        .filesystem
                                                        .desensitize(format!(
                                                            "Graphics/Fogs/{name}"
                                                        ))
                                                        .ok()
                                                        .map(|path| {
                                                            camino::Utf8PathBuf::from(
                                                                path.file_name()
                                                                    .unwrap_or_default(),
                                                            )
                                                        })
                                                });

                                                *sprite = fog_name.as_ref().and_then(|fog_name| {
                                                    let texture = update_state
                                                        .graphics
                                                        .texture_loader
                                                        .load_now_dir(
                                                            update_state.filesystem,
                                                            "Graphics/Fogs",
                                                            fog_name,
                                                        )
                                                        .map_err(|e| {
                                                            luminol_core::error!(
                                                                update_state.toasts,
                                                                e
                                                            )
                                                        })
                                                        .ok()?;
                                                    let viewport = Viewport::new(
                                                        &update_state.graphics,
                                                        Default::default(),
                                                    );
                                                    let sprite = Sprite::basic(
                                                        &update_state.graphics,
                                                        &texture,
                                                        &viewport,
                                                    );
                                                    Some(PreviewSprite {
                                                        sprite,
                                                        sprite_size: texture.size_vec2(),
                                                        viewport,
                                                    })
                                                });
                                            }
                                        });
                                    }
                                },
                            );

                        // Scroll the selected item into view
                        if !self.scrolled_on_first_open {
                            let row = if fog_name.is_none() {
                                Some(0)
                            } else {
                                filtered_entries.iter().enumerate().find_map(|(i, entry)| {
                                    (desensitized_fog_name.as_ref() == Some(&entry.path))
                                        .then_some(i + 1)
                                })
                            };
                            if let Some(row) = row {
                                let spacing = ui.spacing().item_spacing.y;
                                let max = row as f32 * (row_height + spacing) + spacing;
                                let min = row as f32 * (row_height + spacing) + row_height
                                    - spacing
                                    - scroll_area_output.inner_rect.height();
                                if scroll_area_output.state.offset.y > max {
                                    scroll_area_output.state.offset.y = max;
                                    scroll_area_output
                                        .state
                                        .store(ui.ctx(), scroll_area_output.id);
                                } else if scroll_area_output.state.offset.y < min {
                                    scroll_area_output.state.offset.y = min;
                                    scroll_area_output
                                        .state
                                        .store(ui.ctx(), scroll_area_output.id);
                                }
                            }
                            self.scrolled_on_first_open = true;
                        }
                    });
                });

                egui::TopBottomPanel::top(self.id_source.with("top")).show_inside(ui, |ui| {
                    ui.add_space(1.0); // pad out the top

                    ui.columns(2, |columns| {
                        columns[0].horizontal(|ui| {
                            ui.label("Hue");
                            if ui.add(egui::Slider::new(fog_hue, 0..=360)).changed() {
                                if let Some(sprite) = sprite {
                                    sprite
                                        .sprite
                                        .graphic
                                        .set_hue(&update_state.graphics.render_state, *fog_hue);
                                }
                            }
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Opacity");
                            if ui.add(egui::Slider::new(fog_opacity, 0..=255)).changed() {
                                if let Some(sprite) = sprite {
                                    sprite.sprite.graphic.set_opacity(
                                        &update_state.graphics.render_state,
                                        *fog_opacity,
                                    );
                                }
                            }
                        });
                    });

                    ui.columns(2, |columns| {
                        columns[0].horizontal(|ui| {
                            ui.label("Blend Type");
                            ui.add(EnumComboBox::new("blend_type", fog_blend_type));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("Zoom");
                            ui.add(egui::DragValue::new(fog_zoom).range(100..=800));
                        });
                    });

                    ui.columns(2, |columns| {
                        columns[0].horizontal(|ui| {
                            ui.label("SX");
                            ui.add(egui::DragValue::new(fog_sx).range(-256..=256));
                        });

                        columns[1].horizontal(|ui| {
                            ui.label("SY");
                            ui.add(egui::DragValue::new(fog_sy).range(-256..=256));
                        });
                    });

                    ui.add_space(1.0); // pad out the bottom
                });
                egui::TopBottomPanel::bottom(self.id_source.with("bottom")).show_inside(ui, |ui| {
                    ui.add_space(ui.style().spacing.item_spacing.y);
                    crate::components::close_options_ui(ui, &mut keep_open, &mut needs_save);
                });

                egui::CentralPanel::default().show_inside(ui, |ui| {
                    egui::ScrollArea::both()
                        .auto_shrink([false, false])
                        .show_viewport(ui, |ui, scroll_rect| {
                            if let Some(sprite) = sprite {
                                sprite.ui(ui, scroll_rect, update_state);
                            }
                        });
                });
            });

        if needs_save {
            data.fog_name.0.clone_from(fog_name);
        }

        if !(win_open && keep_open) {
            self.state = State::Closed;
            self.scrolled_on_first_open = false;
        }

        needs_save
    }
}
