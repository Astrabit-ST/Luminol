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

use luminol_components::UiExt;
use luminol_core::Modal;
use strum::IntoEnumIterator;

use super::util::update_flash_maps;
use luminol_data::rpg::animation::{Position, Scope};

impl luminol_core::Window for super::Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("animation_editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let data = std::mem::take(update_state.data); // take data to avoid borrow checker issues
        let mut animations = data.animations();
        let animations_len = animations.data.len();
        let system = data.system();

        let mut modified = false;

        self.selected_animation_name = None;

        let name = if let Some(name) = &self.selected_animation_name {
            format!("Editing animation {:?}", name)
        } else {
            "Animation Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(720.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Animations",
                    &mut animations.data,
                    |animation| format!("{:0>4}: {}", animation.id + 1, animation.name),
                    |ui, animations, id, update_state| {
                        for i in animations.len()..animations_len {
                            self.frame_edit_state.history.remove_animation(i);
                        }

                        let animation = &mut animations[id];
                        self.selected_animation_name = Some(animation.name.clone());
                        if animation.frames.is_empty() {
                            animation.frames.push(Default::default());
                            animation.frame_max = 1;
                        }

                        let clip_rect = ui.clip_rect();

                        if !self.frame_edit_state.flash_maps.contains(id) {
                            if !luminol_core::slice_is_sorted_by_key(&animation.timings, |timing| {
                                timing.frame
                            }) {
                                animation.timings.sort_by_key(|timing| timing.frame);
                            }
                            self.frame_edit_state
                                .flash_maps
                                .insert(id, super::util::FlashMaps::new(&animation.timings));
                        }

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut animation.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            let changed = ui
                                .add(luminol_components::Field::new(
                                    "Battler Position",
                                    |ui: &mut egui::Ui| {
                                        let mut modified = false;
                                        let mut response = egui::Frame::none()
                                            .show(ui, |ui| {
                                                ui.columns(Position::iter().count(), |columns| {
                                                    for (i, position) in
                                                        Position::iter().enumerate()
                                                    {
                                                        if columns[i]
                                                            .radio_value(
                                                                &mut animation.position,
                                                                position,
                                                                position.to_string(),
                                                            )
                                                            .changed()
                                                        {
                                                            modified = true;
                                                        }
                                                    }
                                                });
                                            })
                                            .response;
                                        if modified {
                                            response.mark_changed();
                                        }
                                        response
                                    },
                                ))
                                .changed();
                            if changed {
                                if let Some(frame_view) = &mut self.frame_edit_state.frame_view {
                                    frame_view.frame.update_battler(
                                        &update_state.graphics,
                                        &system,
                                        animation,
                                        None,
                                        None,
                                    );
                                }
                                modified = true;
                            }
                        });

                        ui.with_padded_stripe(false, |ui| {
                            if self.previous_battler_name != system.battler_name {
                                if let Some(frame_view) = &mut self.frame_edit_state.frame_view {
                                    if let Some(battler_name) = &system.battler_name {
                                        match update_state.graphics.texture_loader.load_now(
                                            update_state.filesystem,
                                            format!("Graphics/Battlers/{battler_name}"),
                                        ) {
                                            Ok(texture) => {
                                                frame_view.frame.battler_texture = Some(texture);
                                            }
                                            Err(e) => {
                                                frame_view.frame.battler_texture = None;
                                                super::util::log_battler_error(
                                                    update_state,
                                                    &system,
                                                    animation,
                                                    e,
                                                );
                                            }
                                        }
                                    }
                                    frame_view.frame.rebuild_battler(
                                        &update_state.graphics,
                                        &system,
                                        animation,
                                        luminol_data::Color {
                                            red: 255.,
                                            green: 255.,
                                            blue: 255.,
                                            alpha: 0.,
                                        },
                                        true,
                                    );
                                }

                                self.previous_battler_name.clone_from(&system.battler_name);
                            }

                            if self.previous_animation != Some(animation.id) {
                                self.modals.close_all();
                                self.frame_edit_state.frame_index = self
                                    .frame_edit_state
                                    .frame_index
                                    .min(animation.frames.len().saturating_sub(1));

                                if let Some(modal) =
                                    &mut self.frame_edit_state.animation_graphic_picker
                                {
                                    // reset the modal if the animation has changed (this is practically a no-op)
                                    modal.reset(update_state, animation);
                                }

                                if self.frame_edit_state.animation_state.is_some() {
                                    // Stop the currently playing animation
                                    let animation_state =
                                        self.frame_edit_state.animation_state.take().unwrap();
                                    self.frame_edit_state.frame_index =
                                        animation_state.saved_frame_index;
                                    self.frame_edit_state.saved_frame_index =
                                        Some(animation_state.saved_frame_index);
                                }

                                let atlas =
                                    update_state.graphics.atlas_loader.load_animation_atlas(
                                        &update_state.graphics,
                                        update_state.filesystem,
                                        animation.animation_name.as_deref(),
                                    );

                                if let Some(frame_view) = &mut self.frame_edit_state.frame_view {
                                    let flash_maps =
                                        self.frame_edit_state.flash_maps.get(id).unwrap();
                                    frame_view.frame.atlas = atlas.clone();
                                    frame_view.frame.update_battler(
                                        &update_state.graphics,
                                        &system,
                                        animation,
                                        Some(
                                            flash_maps
                                                .target(self.frame_edit_state.condition)
                                                .compute(self.frame_edit_state.frame_index),
                                        ),
                                        Some(
                                            flash_maps
                                                .hide(self.frame_edit_state.condition)
                                                .compute(self.frame_edit_state.frame_index),
                                        ),
                                    );
                                    frame_view.frame.rebuild_all_cells(
                                        &update_state.graphics,
                                        animation,
                                        self.frame_edit_state.frame_index,
                                    );
                                }

                                let selected_cell = self
                                    .frame_edit_state
                                    .cellpicker
                                    .as_ref()
                                    .map(|cellpicker| cellpicker.selected_cell)
                                    .unwrap_or_default()
                                    .min(atlas.num_patterns().saturating_sub(1));
                                let mut cellpicker = luminol_components::Cellpicker::new(
                                    &update_state.graphics,
                                    atlas,
                                    None,
                                    0.5,
                                );
                                cellpicker.view.display.set_hue(
                                    &update_state.graphics.render_state,
                                    animation.animation_hue as f32 / 360.,
                                );
                                cellpicker.selected_cell = selected_cell;
                                self.frame_edit_state.cellpicker = Some(cellpicker);
                            }

                            let inner_modified = super::frame_edit::show_frame_edit(
                                ui,
                                update_state,
                                clip_rect,
                                &mut self.modals,
                                &system,
                                animation,
                                &mut self.frame_edit_state,
                            );

                            modified |= inner_modified;
                        });

                        let mut collapsing_view_inner = Default::default();
                        let flash_maps = self.frame_edit_state.flash_maps.get_mut(id).unwrap();

                        ui.with_padded_stripe(true, |ui| {
                            let changed = ui
                                .add(luminol_components::Field::new(
                                    "SE and Flash",
                                    |ui: &mut egui::Ui| {
                                        if *update_state.modified_during_prev_frame {
                                            self.collapsing_view.request_sort();
                                        }
                                        if self.previous_animation != Some(animation.id) {
                                            self.collapsing_view.clear_animations();
                                            self.timing_edit_state.se_picker.close_window();
                                        } else if self.collapsing_view.is_animating() {
                                            self.timing_edit_state.se_picker.close_window();
                                        }

                                        let mut timings = std::mem::take(&mut animation.timings);
                                        let egui::InnerResponse { inner, response } =
                                            self.collapsing_view.show_with_sort(
                                                ui,
                                                animation.id,
                                                &mut timings,
                                                |ui, _i, timing| {
                                                    super::timing::show_timing_header(ui, timing)
                                                },
                                                |ui, i, previous_timings, timing| {
                                                    super::timing::show_timing_body(
                                                        ui,
                                                        update_state,
                                                        animation,
                                                        flash_maps,
                                                        &mut self.timing_edit_state,
                                                        (i, previous_timings, timing),
                                                    )
                                                },
                                                |a, b| a.frame.cmp(&b.frame),
                                            );
                                        collapsing_view_inner = inner;
                                        animation.timings = timings;
                                        response
                                    },
                                ))
                                .changed();
                            if changed {
                                if let Some(frame_view) = &mut self.frame_edit_state.frame_view {
                                    frame_view.frame.update_battler(
                                        &update_state.graphics,
                                        &system,
                                        animation,
                                        Some(
                                            flash_maps
                                                .target(self.frame_edit_state.condition)
                                                .compute(self.frame_edit_state.frame_index),
                                        ),
                                        Some(
                                            flash_maps
                                                .hide(self.frame_edit_state.condition)
                                                .compute(self.frame_edit_state.frame_index),
                                        ),
                                    );
                                }
                                modified = true;
                            }
                        });

                        if let Some(i) = collapsing_view_inner.created_entry {
                            let timing = &animation.timings[i];
                            update_flash_maps(timing.condition, |condition| {
                                match timing.flash_scope {
                                    Scope::Target => {
                                        flash_maps
                                            .target_mut(condition)
                                            .append(timing.frame, timing.into());
                                    }
                                    Scope::Screen => {
                                        flash_maps
                                            .screen_mut(condition)
                                            .append(timing.frame, timing.into());
                                    }
                                    Scope::HideTarget => {
                                        flash_maps
                                            .hide_mut(condition)
                                            .append(timing.frame, timing.into());
                                    }
                                    Scope::None => {}
                                }
                            });
                            self.frame_edit_state
                                .frame_view
                                .as_mut()
                                .unwrap()
                                .frame
                                .update_battler(
                                    &update_state.graphics,
                                    &system,
                                    animation,
                                    Some(
                                        flash_maps
                                            .target(self.frame_edit_state.condition)
                                            .compute(self.frame_edit_state.frame_index),
                                    ),
                                    Some(
                                        flash_maps
                                            .hide(self.frame_edit_state.condition)
                                            .compute(self.frame_edit_state.frame_index),
                                    ),
                                );
                        }

                        if let Some((i, timing)) = collapsing_view_inner.deleted_entry {
                            update_flash_maps(timing.condition, |condition| {
                                let rank = |frame, scope| {
                                    animation.timings[..i]
                                        .iter()
                                        .rev()
                                        .take_while(|t| t.frame == frame)
                                        .filter(|t| {
                                            t.flash_scope == scope
                                                && super::util::filter_timing(t, condition)
                                        })
                                        .count()
                                };
                                match timing.flash_scope {
                                    Scope::Target => {
                                        flash_maps.target_mut(condition).remove(
                                            timing.frame,
                                            rank(timing.frame, Scope::Target),
                                        );
                                    }
                                    Scope::Screen => {
                                        flash_maps.screen_mut(condition).remove(
                                            timing.frame,
                                            rank(timing.frame, Scope::Screen),
                                        );
                                    }
                                    Scope::HideTarget => {
                                        flash_maps.hide_mut(condition).remove(
                                            timing.frame,
                                            rank(timing.frame, Scope::HideTarget),
                                        );
                                    }
                                    Scope::None => {}
                                }
                            });

                            self.frame_edit_state
                                .frame_view
                                .as_mut()
                                .unwrap()
                                .frame
                                .update_battler(
                                    &update_state.graphics,
                                    &system,
                                    animation,
                                    Some(
                                        flash_maps
                                            .target(self.frame_edit_state.condition)
                                            .compute(self.frame_edit_state.frame_index),
                                    ),
                                    Some(
                                        flash_maps
                                            .hide(self.frame_edit_state.condition)
                                            .compute(self.frame_edit_state.frame_index),
                                    ),
                                );
                        }

                        self.previous_animation = Some(animation.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            animations.modified = true;
        }

        drop(animations);
        drop(system);

        *update_state.data = data; // restore data
    }
}
