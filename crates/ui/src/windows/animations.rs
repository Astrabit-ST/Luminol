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

use egui::Widget;
use luminol_components::UiExt;
use luminol_core::Modal;

use luminol_data::BlendMode;
use luminol_graphics::frame::{FRAME_HEIGHT, FRAME_WIDTH};
use luminol_modals::sound_picker::Modal as SoundPicker;

/// Database - Animations management window.
pub struct Window {
    selected_animation_name: Option<String>,
    previous_animation: Option<usize>,
    previous_timing_frame: Option<i32>,
    frame_edit_state: FrameEditState,

    collapsing_view: luminol_components::CollapsingView,
    timing_se_picker: SoundPicker,
    modals: Modals,
    view: luminol_components::DatabaseView,
}

struct FrameEditState {
    frame_index: usize,
    enable_onion_skin: bool,
    frame_view: Option<luminol_components::AnimationFrameView>,
    cellpicker: Option<luminol_components::Cellpicker>,
}

struct Modals {
    copy_frames: luminol_modals::animations::copy_frames_tool::Modal,
    clear_frames: luminol_modals::animations::clear_frames_tool::Modal,
    tween: luminol_modals::animations::tween_tool::Modal,
    batch_edit: luminol_modals::animations::batch_edit_tool::Modal,
}

impl Modals {
    fn close_all(&mut self) {
        self.copy_frames.close_window();
        self.clear_frames.close_window();
        self.tween.close_window();
        self.batch_edit.close_window();
    }
}

impl Default for Window {
    fn default() -> Self {
        Self {
            selected_animation_name: None,
            previous_animation: None,
            previous_timing_frame: None,
            frame_edit_state: FrameEditState {
                frame_index: 0,
                enable_onion_skin: false,
                frame_view: None,
                cellpicker: None,
            },
            collapsing_view: luminol_components::CollapsingView::new(),
            timing_se_picker: SoundPicker::new(
                luminol_audio::Source::SE,
                "animations_timing_se_picker",
            ),
            modals: Modals {
                copy_frames: luminol_modals::animations::copy_frames_tool::Modal::new(
                    "animations_copy_frames_tool",
                ),
                clear_frames: luminol_modals::animations::clear_frames_tool::Modal::new(
                    "animations_clear_frames_tool",
                ),
                tween: luminol_modals::animations::tween_tool::Modal::new("animations_tween_tool"),
                batch_edit: luminol_modals::animations::batch_edit_tool::Modal::new(
                    "animations_batch_edit_tool",
                ),
            },
            view: luminol_components::DatabaseView::new(),
        }
    }
}

impl Window {
    fn log_atlas_error(
        update_state: &mut luminol_core::UpdateState<'_>,
        animation: &luminol_data::rpg::Animation,
        e: color_eyre::Report,
    ) {
        luminol_core::error!(
            update_state.toasts,
            e.wrap_err(format!(
                "While loading texture {:?} for animation {:0>4} {:?}",
                animation.animation_name,
                animation.id + 1,
                animation.name,
            ),),
        );
    }

    fn show_timing_header(ui: &mut egui::Ui, timing: &luminol_data::rpg::animation::Timing) {
        let mut vec = Vec::with_capacity(3);

        match timing.condition {
            luminol_data::rpg::animation::Condition::None => {}
            luminol_data::rpg::animation::Condition::Hit => vec.push("on hit".into()),
            luminol_data::rpg::animation::Condition::Miss => vec.push("on miss".into()),
        }

        if let Some(path) = &timing.se.name {
            vec.push(format!("play {:?}", path.file_name().unwrap_or_default()));
        };

        match timing.flash_scope {
            luminol_data::rpg::animation::Scope::None => {}
            luminol_data::rpg::animation::Scope::Target => {
                vec.push(format!(
                    "flash target #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                    timing.flash_duration,
                ));
            }
            luminol_data::rpg::animation::Scope::Screen => {
                vec.push(format!(
                    "flash screen #{:0>2x}{:0>2x}{:0>2x}{:0>2x} for {} frames",
                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                    timing.flash_duration,
                ));
            }
            luminol_data::rpg::animation::Scope::HideTarget => {
                vec.push(format!("hide target for {} frames", timing.flash_duration));
            }
        }

        ui.label(format!(
            "Frame {:0>3}: {}",
            timing.frame + 1,
            vec.join(", ")
        ));
    }

    fn resize_frame(frame: &mut luminol_data::rpg::animation::Frame, new_cell_max: usize) {
        let old_capacity = frame.cell_data.xsize();
        let new_capacity = if new_cell_max == 0 {
            0
        } else {
            new_cell_max.next_power_of_two()
        };

        // Instead of resizing `frame.cell_data` every time we call this function, we increase the
        // size of `frame.cell_data` only it's too small and we decrease the size of
        // `frame.cell_data` only if it's at <= 25% capacity for better efficiency
        let capacity_too_low = old_capacity < new_capacity;
        let capacity_too_high = old_capacity >= new_capacity * 4;

        if capacity_too_low {
            frame.cell_data.resize(new_capacity, 8);
            for i in old_capacity..new_capacity {
                frame.cell_data[(i, 0)] = -1;
                frame.cell_data[(i, 1)] = 0;
                frame.cell_data[(i, 2)] = 0;
                frame.cell_data[(i, 3)] = 100;
                frame.cell_data[(i, 4)] = 0;
                frame.cell_data[(i, 5)] = 0;
                frame.cell_data[(i, 6)] = 255;
                frame.cell_data[(i, 7)] = 1;
            }
        } else if capacity_too_high {
            frame.cell_data.resize(new_capacity * 2, 8);
        }

        frame.cell_max = new_cell_max;
    }

    fn show_timing_body(
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        animation_id: usize,
        animation_frame_max: i32,
        timing_se_picker: &mut SoundPicker,
        previous_timing_frame: &mut Option<i32>,
        timing: (usize, &mut luminol_data::rpg::animation::Timing),
    ) -> egui::Response {
        let (timing_index, timing) = timing;
        let mut modified = false;

        let mut response = egui::Frame::none()
            .show(ui, |ui| {
                ui.columns(2, |columns| {
                    columns[0].columns(2, |columns| {
                        modified |= columns[1]
                            .add(luminol_components::Field::new(
                                "Condition",
                                luminol_components::EnumComboBox::new(
                                    (animation_id, timing_index, "condition"),
                                    &mut timing.condition,
                                ),
                            ))
                            .changed();

                        modified |= columns[0]
                            .add(luminol_components::Field::new(
                                "Frame",
                                |ui: &mut egui::Ui| {
                                    let mut frame =
                                        previous_timing_frame.unwrap_or(timing.frame + 1);
                                    let mut response = egui::DragValue::new(&mut frame)
                                        .clamp_range(1..=animation_frame_max)
                                        .update_while_editing(false)
                                        .ui(ui);
                                    response.changed = false;
                                    if response.dragged() {
                                        *previous_timing_frame = Some(frame);
                                    } else {
                                        timing.frame = frame - 1;
                                        *previous_timing_frame = None;
                                        response.changed = true;
                                    }
                                    response
                                },
                            ))
                            .changed();
                    });

                    modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "SE",
                            timing_se_picker.button(&mut timing.se, update_state),
                        ))
                        .changed();
                });

                if timing.flash_scope == luminol_data::rpg::animation::Scope::None {
                    modified |= ui
                        .add(luminol_components::Field::new(
                            "Flash",
                            luminol_components::EnumComboBox::new(
                                (animation_id, timing_index, "flash_scope"),
                                &mut timing.flash_scope,
                            ),
                        ))
                        .changed();
                } else {
                    ui.columns(2, |columns| {
                        modified |= columns[0]
                            .add(luminol_components::Field::new(
                                "Flash",
                                luminol_components::EnumComboBox::new(
                                    (animation_id, timing_index, "flash_scope"),
                                    &mut timing.flash_scope,
                                ),
                            ))
                            .changed();

                        modified |= columns[1]
                            .add(luminol_components::Field::new(
                                "Flash Duration",
                                egui::DragValue::new(&mut timing.flash_duration)
                                    .clamp_range(1..=animation_frame_max),
                            ))
                            .changed();
                    });
                }

                if matches!(
                    timing.flash_scope,
                    luminol_data::rpg::animation::Scope::Target
                        | luminol_data::rpg::animation::Scope::Screen
                ) {
                    modified |= ui
                        .add(luminol_components::Field::new(
                            "Flash Color",
                            |ui: &mut egui::Ui| {
                                let mut color = [
                                    timing.flash_color.red.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.green.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.blue.clamp(0., 255.).trunc() as u8,
                                    timing.flash_color.alpha.clamp(0., 255.).trunc() as u8,
                                ];
                                ui.spacing_mut().interact_size.x = ui.available_width(); // make the color picker button as wide as possible
                                let response = ui.color_edit_button_srgba_unmultiplied(&mut color);
                                if response.changed() {
                                    timing.flash_color.red = color[0] as f64;
                                    timing.flash_color.green = color[1] as f64;
                                    timing.flash_color.blue = color[2] as f64;
                                    timing.flash_color.alpha = color[3] as f64;
                                }
                                response
                            },
                        ))
                        .changed();
                }
            })
            .response;

        if modified {
            response.mark_changed();
        }
        response
    }

    fn show_frame_edit(
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        clip_rect: egui::Rect,
        modals: &mut Modals,
        animation: &mut luminol_data::rpg::Animation,
        state: &mut FrameEditState,
    ) -> (bool, bool) {
        let mut modified = false;

        let frame_view = if let Some(frame_view) = &mut state.frame_view {
            frame_view
        } else {
            let atlas = match update_state.graphics.atlas_loader.load_animation_atlas(
                &update_state.graphics,
                update_state.filesystem,
                animation,
            ) {
                Ok(atlas) => atlas,
                Err(e) => {
                    Self::log_atlas_error(update_state, animation, e);
                    return (modified, true);
                }
            };
            let mut frame_view = luminol_components::AnimationFrameView::new(update_state, atlas);
            frame_view
                .frame
                .update_all_cells(&update_state.graphics, animation, state.frame_index);
            state.frame_view = Some(frame_view);
            state.frame_view.as_mut().unwrap()
        };

        let cellpicker = if let Some(cellpicker) = &mut state.cellpicker {
            cellpicker
        } else {
            let atlas = frame_view.frame.atlas.clone();
            let cellpicker = luminol_components::Cellpicker::new(&update_state.graphics, atlas);
            state.cellpicker = Some(cellpicker);
            state.cellpicker.as_mut().unwrap()
        };

        ui.horizontal(|ui| {
            ui.add(luminol_components::Field::new(
                "Editor Scale",
                egui::Slider::new(&mut frame_view.scale, 15.0..=300.0)
                    .suffix("%")
                    .logarithmic(true)
                    .fixed_decimals(0),
            ));

            state.frame_index = state
                .frame_index
                .min(animation.frames.len().saturating_sub(1));
            state.frame_index += 1;
            let changed = ui
                .add(luminol_components::Field::new(
                    "Frame",
                    egui::DragValue::new(&mut state.frame_index)
                        .clamp_range(1..=animation.frames.len()),
                ))
                .changed();
            state.frame_index -= 1;
            if changed {
                frame_view.frame.update_all_cells(
                    &update_state.graphics,
                    animation,
                    state.frame_index,
                );
            }

            ui.add(luminol_components::Field::new(
                "Onion Skin",
                egui::Checkbox::without_text(&mut state.enable_onion_skin),
            ));

            ui.with_layout(
                egui::Layout {
                    main_dir: egui::Direction::RightToLeft,
                    cross_align: egui::Align::Max,
                    ..*ui.layout()
                },
                |ui| {
                    egui::Frame::none()
                        .outer_margin(egui::Margin {
                            bottom: 2. * ui.spacing().item_spacing.y,
                            ..egui::Margin::ZERO
                        })
                        .show(ui, |ui| {
                            ui.menu_button("Tools ⏷", |ui| {
                                ui.add_enabled_ui(state.frame_index != 0, |ui| {
                                    if ui.button("Copy previous frame").clicked()
                                        && state.frame_index != 0
                                    {
                                        animation.frames[state.frame_index] =
                                            animation.frames[state.frame_index - 1].clone();
                                        frame_view.frame.update_all_cells(
                                            &update_state.graphics,
                                            animation,
                                            state.frame_index,
                                        );
                                        modified = true;
                                    }
                                });

                                ui.add(modals.copy_frames.button((), update_state));

                                ui.add(modals.clear_frames.button((), update_state));

                                ui.add_enabled_ui(animation.frames.len() >= 3, |ui| {
                                    if animation.frames.len() >= 3 {
                                        ui.add(modals.tween.button((), update_state));
                                    } else {
                                        modals.tween.close_window();
                                    }
                                });

                                ui.add(modals.batch_edit.button((), update_state));
                            });
                        });
                },
            );
        });

        if modals
            .copy_frames
            .show_window(ui.ctx(), state.frame_index, animation.frames.len())
        {
            let mut iter = 0..modals.copy_frames.frame_count;
            while let Some(i) = if modals.copy_frames.dst_frame <= modals.copy_frames.src_frame {
                iter.next()
            } else {
                iter.next_back()
            } {
                animation.frames[modals.copy_frames.dst_frame + i] =
                    animation.frames[modals.copy_frames.src_frame + i].clone();
            }
            frame_view
                .frame
                .update_all_cells(&update_state.graphics, animation, state.frame_index);
            modified = true;
        }

        if modals
            .clear_frames
            .show_window(ui.ctx(), state.frame_index, animation.frames.len())
        {
            for i in modals.clear_frames.start_frame..=modals.clear_frames.end_frame {
                animation.frames[i] = Default::default();
            }
            frame_view
                .frame
                .update_all_cells(&update_state.graphics, animation, state.frame_index);
            modified = true;
        }

        if modals
            .tween
            .show_window(ui.ctx(), state.frame_index, animation.frames.len())
        {
            for i in modals.tween.start_cell..=modals.tween.end_cell {
                let data = &animation.frames[modals.tween.start_frame].cell_data;
                if i >= data.xsize() || data[(i, 0)] < 0 {
                    continue;
                }
                let data = &animation.frames[modals.tween.end_frame].cell_data;
                if i >= data.xsize() || data[(i, 0)] < 0 {
                    continue;
                }

                for j in modals.tween.start_frame..=modals.tween.end_frame {
                    let lerp = |frames: &Vec<luminol_data::rpg::animation::Frame>, property| {
                        (
                            egui::lerp(
                                frames[modals.tween.start_frame].cell_data[(i, property)] as f64
                                    ..=frames[modals.tween.end_frame].cell_data[(i, property)]
                                        as f64,
                                (j - modals.tween.start_frame) as f64
                                    / (modals.tween.end_frame - modals.tween.start_frame) as f64,
                            ),
                            frames[modals.tween.start_frame].cell_data[(i, property)]
                                <= frames[modals.tween.end_frame].cell_data[(i, property)],
                        )
                    };

                    if animation.frames[j].cell_data.xsize() < i + 1 {
                        Self::resize_frame(&mut animation.frames[j], i + 1);
                    } else if animation.frames[j].cell_max < i + 1 {
                        animation.frames[j].cell_max = i + 1;
                    }

                    if modals.tween.tween_pattern {
                        let (val, orientation) = lerp(&animation.frames, 0);
                        animation.frames[j].cell_data[(i, 0)] =
                            if orientation { val.floor() } else { val.ceil() } as i16;
                    } else if animation.frames[j].cell_data[(i, 0)] < 0 {
                        animation.frames[j].cell_data[(i, 0)] = 0;
                    }

                    if modals.tween.tween_position {
                        let (val, orientation) = lerp(&animation.frames, 1);
                        animation.frames[j].cell_data[(i, 1)] =
                            if orientation { val.floor() } else { val.ceil() } as i16;

                        let (val, orientation) = lerp(&animation.frames, 2);
                        animation.frames[j].cell_data[(i, 2)] =
                            if orientation { val.floor() } else { val.ceil() } as i16;

                        let (val, _) = lerp(&animation.frames, 3);
                        animation.frames[j].cell_data[(i, 3)] = val.floor() as i16;

                        let (val, _) = lerp(&animation.frames, 4);
                        animation.frames[j].cell_data[(i, 4)] = val.floor() as i16;
                    }

                    if modals.tween.tween_shading {
                        let (val, _) = lerp(&animation.frames, 6);
                        animation.frames[j].cell_data[(i, 6)] = val.floor() as i16;

                        let (val, _) = lerp(&animation.frames, 7);
                        animation.frames[j].cell_data[(i, 7)] = val.floor() as i16;
                    }
                }
            }
            frame_view
                .frame
                .update_all_cells(&update_state.graphics, animation, state.frame_index);
            modified = true;
        }

        if modals.batch_edit.show_window(
            ui.ctx(),
            state.frame_index,
            animation.frames.len(),
            frame_view.frame.atlas.num_patterns(),
        ) {
            for i in modals.batch_edit.start_frame..=modals.batch_edit.end_frame {
                let data = &mut animation.frames[i].cell_data;
                for j in 0..data.xsize() {
                    if data[(j, 0)] < 0 {
                        continue;
                    }
                    match modals.batch_edit.mode {
                        luminol_modals::animations::batch_edit_tool::Mode::Set => {
                            if modals.batch_edit.set_pattern_enabled {
                                data[(j, 0)] = modals.batch_edit.set_pattern;
                            }
                            if modals.batch_edit.set_x_enabled {
                                data[(j, 1)] = modals.batch_edit.set_x;
                            }
                            if modals.batch_edit.set_y_enabled {
                                data[(j, 2)] = modals.batch_edit.set_y;
                            }
                            if modals.batch_edit.set_scale_enabled {
                                data[(j, 3)] = modals.batch_edit.set_scale;
                            }
                            if modals.batch_edit.set_rotation_enabled {
                                data[(j, 4)] = modals.batch_edit.set_rotation;
                            }
                            if modals.batch_edit.set_flip_enabled {
                                data[(j, 5)] = modals.batch_edit.set_flip;
                            }
                            if modals.batch_edit.set_opacity_enabled {
                                data[(j, 6)] = modals.batch_edit.set_opacity;
                            }
                            if modals.batch_edit.set_blending_enabled {
                                data[(j, 7)] = modals.batch_edit.set_blending;
                            }
                        }
                        luminol_modals::animations::batch_edit_tool::Mode::Add => {
                            data[(j, 0)] = data[(j, 0)]
                                .saturating_add(modals.batch_edit.add_pattern)
                                .clamp(
                                    0,
                                    frame_view.frame.atlas.num_patterns().saturating_sub(1) as i16,
                                );
                            data[(j, 1)] = data[(j, 1)]
                                .saturating_add(modals.batch_edit.add_x)
                                .clamp(-(FRAME_WIDTH as i16 / 2), FRAME_WIDTH as i16 / 2);
                            data[(j, 2)] = data[(j, 2)]
                                .saturating_add(modals.batch_edit.add_y)
                                .clamp(-(FRAME_HEIGHT as i16 / 2), FRAME_HEIGHT as i16 / 2);
                            data[(j, 3)] = data[(j, 3)]
                                .saturating_add(modals.batch_edit.add_scale)
                                .max(1);
                            data[(j, 4)] += modals.batch_edit.add_rotation;
                            if !(0..=360).contains(&data[(j, 4)]) {
                                data[(j, 4)] = data[(j, 4)].rem_euclid(360);
                            }
                            if modals.batch_edit.add_flip {
                                if data[(j, 5)] == 1 {
                                    data[(j, 5)] = 0;
                                } else {
                                    data[(j, 5)] = 1;
                                }
                            }
                            data[(j, 6)] = data[(j, 6)]
                                .saturating_add(modals.batch_edit.add_opacity)
                                .clamp(0, 255);
                            data[(j, 7)] += modals.batch_edit.add_blending;
                            if !(0..3).contains(&data[(j, 7)]) {
                                data[(j, 7)] = data[(j, 7)].rem_euclid(3);
                            }
                        }
                        luminol_modals::animations::batch_edit_tool::Mode::Mul => {
                            data[(j, 0)] =
                                ((data[(j, 0)] + 1) as f64 * modals.batch_edit.mul_pattern)
                                    .clamp(1., frame_view.frame.atlas.num_patterns() as f64)
                                    .round_ties_even() as i16
                                    - 1;
                            data[(j, 1)] = (data[(j, 1)] as f64 * modals.batch_edit.mul_x)
                                .clamp(-(FRAME_WIDTH as f64 / 2.), FRAME_WIDTH as f64 / 2.)
                                .round_ties_even()
                                as i16;
                            data[(j, 2)] = (data[(j, 2)] as f64 * modals.batch_edit.mul_y)
                                .clamp(-(FRAME_HEIGHT as f64 / 2.), FRAME_HEIGHT as f64 / 2.)
                                .round_ties_even()
                                as i16;
                            data[(j, 3)] = (data[(j, 3)] as f64 * modals.batch_edit.mul_scale)
                                .clamp(1., i16::MAX as f64)
                                .round_ties_even()
                                as i16;
                            data[(j, 4)] = (data[(j, 4)] as f64 * modals.batch_edit.mul_rotation)
                                .round_ties_even()
                                as i16;
                            if !(0..=360).contains(&data[(j, 4)]) {
                                data[(j, 4)] = data[(j, 4)].rem_euclid(360);
                            }
                            data[(j, 6)] = (data[(j, 6)] as f64 * modals.batch_edit.mul_opacity)
                                .min(255.)
                                .round_ties_even()
                                as i16;
                        }
                    }
                }
            }
            frame_view
                .frame
                .update_all_cells(&update_state.graphics, animation, state.frame_index);
            modified = true;
        }

        let canvas_rect = egui::Resize::default()
            .resizable([false, true])
            .min_width(ui.available_width())
            .max_width(ui.available_width())
            .show(ui, |ui| {
                egui::Frame::dark_canvas(ui.style())
                    .show(ui, |ui| {
                        let (_, rect) = ui.allocate_space(ui.available_size());
                        rect
                    })
                    .inner
            });

        let frame = &mut animation.frames[state.frame_index];

        if frame_view
            .selected_cell_index
            .is_some_and(|i| i >= frame.cell_data.xsize() || frame.cell_data[(i, 0)] < 0)
        {
            frame_view.selected_cell_index = None;
        }
        if frame_view
            .hovered_cell_index
            .is_some_and(|i| i >= frame.cell_data.xsize() || frame.cell_data[(i, 0)] < 0)
        {
            frame_view.hovered_cell_index = None;
            frame_view.hovered_cell_drag_pos = None;
            frame_view.hovered_cell_drag_offset = None;
        }

        // Handle dragging of cells to move them
        if let (Some(i), Some(drag_pos)) = (
            frame_view.hovered_cell_index,
            frame_view.hovered_cell_drag_pos,
        ) {
            if (frame.cell_data[(i, 1)], frame.cell_data[(i, 2)]) != drag_pos {
                (frame.cell_data[(i, 1)], frame.cell_data[(i, 2)]) = drag_pos;
                frame_view.frame.update_cell(
                    &update_state.graphics,
                    animation,
                    state.frame_index,
                    i,
                );
                modified = true;
            }
        }

        egui::Frame::none().show(ui, |ui| {
            let frame = &mut animation.frames[state.frame_index];
            if let Some(i) = frame_view.selected_cell_index {
                let mut properties_modified = false;

                ui.label(format!("Cell {}", i + 1));

                ui.columns(4, |columns| {
                    let mut pattern = frame.cell_data[(i, 0)] + 1;
                    let changed = columns[0]
                        .add(luminol_components::Field::new(
                            "Pattern",
                            egui::DragValue::new(&mut pattern)
                                .clamp_range(1..=frame_view.frame.atlas.num_patterns() as i16),
                        ))
                        .changed();
                    if changed {
                        frame.cell_data[(i, 0)] = pattern - 1;
                        properties_modified = true;
                    }

                    properties_modified |= columns[1]
                        .add(luminol_components::Field::new(
                            "X",
                            egui::DragValue::new(&mut frame.cell_data[(i, 1)])
                                .clamp_range(-(FRAME_WIDTH as i16 / 2)..=FRAME_WIDTH as i16 / 2),
                        ))
                        .changed();

                    properties_modified |= columns[2]
                        .add(luminol_components::Field::new(
                            "Y",
                            egui::DragValue::new(&mut frame.cell_data[(i, 2)])
                                .clamp_range(-(FRAME_HEIGHT as i16 / 2)..=FRAME_HEIGHT as i16 / 2),
                        ))
                        .changed();

                    properties_modified |= columns[3]
                        .add(luminol_components::Field::new(
                            "Scale",
                            egui::DragValue::new(&mut frame.cell_data[(i, 3)])
                                .clamp_range(1..=i16::MAX)
                                .suffix("%"),
                        ))
                        .changed();
                });

                ui.columns(4, |columns| {
                    properties_modified |= columns[0]
                        .add(luminol_components::Field::new(
                            "Rotation",
                            egui::DragValue::new(&mut frame.cell_data[(i, 4)])
                                .clamp_range(0..=360)
                                .suffix("°"),
                        ))
                        .changed();

                    let mut flip = frame.cell_data[(i, 5)] == 1;
                    let changed = columns[1]
                        .add(luminol_components::Field::new(
                            "Flip",
                            egui::Checkbox::without_text(&mut flip),
                        ))
                        .changed();
                    if changed {
                        frame.cell_data[(i, 5)] = if flip { 1 } else { 0 };
                        properties_modified = true;
                    }

                    properties_modified |= columns[2]
                        .add(luminol_components::Field::new(
                            "Opacity",
                            egui::DragValue::new(&mut frame.cell_data[(i, 6)]).clamp_range(0..=255),
                        ))
                        .changed();

                    let mut blend_mode = match frame.cell_data[(i, 7)] {
                        1 => BlendMode::Add,
                        2 => BlendMode::Subtract,
                        _ => BlendMode::Normal,
                    };
                    let changed = columns[3]
                        .add(luminol_components::Field::new(
                            "Blending",
                            luminol_components::EnumComboBox::new(
                                (animation.id, state.frame_index, i, 7usize),
                                &mut blend_mode,
                            ),
                        ))
                        .changed();
                    if changed {
                        frame.cell_data[(i, 7)] = match blend_mode {
                            BlendMode::Normal => 0,
                            BlendMode::Add => 1,
                            BlendMode::Subtract => 2,
                        };
                        properties_modified = true;
                    }
                });

                if properties_modified {
                    frame_view.frame.update_cell(
                        &update_state.graphics,
                        animation,
                        state.frame_index,
                        i,
                    );
                    modified = true;
                }
            }
        });

        egui::ScrollArea::horizontal().show_viewport(ui, |ui, scroll_rect| {
            cellpicker.ui(update_state, ui, scroll_rect);
        });

        ui.allocate_ui_at_rect(canvas_rect, |ui| {
            frame_view.frame.enable_onion_skin = state.enable_onion_skin && state.frame_index != 0;
            let egui::InnerResponse {
                inner: hover_pos,
                response,
            } = frame_view.ui(ui, update_state, clip_rect);

            // If the pointer is hovering over the frame view, prevent parent widgets
            // from receiving scroll events so that scaling the frame view with the
            // scroll wheel doesn't also scroll the scroll area that the frame view is
            // in
            if response.hovered() {
                ui.ctx()
                    .input_mut(|i| i.smooth_scroll_delta = egui::Vec2::ZERO);
            }

            let frame = &mut animation.frames[state.frame_index];

            // Create new cell on double click
            if let Some((x, y)) = hover_pos {
                if response.double_clicked() {
                    let next_cell_index = (frame.cell_max..frame.cell_data.xsize())
                        .find(|i| frame.cell_data[(*i, 0)] < 0)
                        .unwrap_or(frame.cell_data.xsize());

                    Self::resize_frame(frame, next_cell_index + 1);

                    frame.cell_data[(next_cell_index, 0)] = cellpicker.selected_cell as i16;
                    frame.cell_data[(next_cell_index, 1)] = x;
                    frame.cell_data[(next_cell_index, 2)] = y;
                    frame.cell_data[(next_cell_index, 3)] = 100;
                    frame.cell_data[(next_cell_index, 4)] = 0;
                    frame.cell_data[(next_cell_index, 5)] = 0;
                    frame.cell_data[(next_cell_index, 6)] = 255;
                    frame.cell_data[(next_cell_index, 7)] = 1;

                    frame_view.frame.update_cell(
                        &update_state.graphics,
                        animation,
                        state.frame_index,
                        next_cell_index,
                    );
                    frame_view.selected_cell_index = Some(next_cell_index);

                    modified = true;
                }
            }

            let frame = &mut animation.frames[state.frame_index];

            // Handle pressing delete or backspace to delete cells
            if let Some(i) = frame_view.selected_cell_index {
                if i < frame.cell_data.xsize()
                    && frame.cell_data[(i, 0)] >= 0
                    && response.has_focus()
                    && ui.input(|i| {
                        i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                    })
                {
                    frame.cell_data[(i, 0)] = -1;

                    if i + 1 >= frame.cell_max {
                        Self::resize_frame(
                            frame,
                            (0..frame
                                .cell_data
                                .xsize()
                                .min(frame.cell_max.saturating_sub(1)))
                                .rev()
                                .find_map(|i| (frame.cell_data[(i, 0)] >= 0).then_some(i + 1))
                                .unwrap_or(0),
                        );
                    }

                    frame_view.frame.update_cell(
                        &update_state.graphics,
                        animation,
                        state.frame_index,
                        i,
                    );
                    frame_view.selected_cell_index = None;
                    modified = true;
                }
            }
        });

        (modified, false)
    }
}

impl luminol_core::Window for Window {
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

        let mut modified = false;

        self.selected_animation_name = None;

        let name = if let Some(name) = &self.selected_animation_name {
            format!("Editing animation {:?}", name)
        } else {
            "Animation Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Animations",
                    &mut animations.data,
                    |animation| format!("{:0>4}: {}", animation.id + 1, animation.name),
                    |ui, animations, id, update_state| {
                        let animation = &mut animations[id];
                        self.selected_animation_name = Some(animation.name.clone());

                        let clip_rect = ui.clip_rect();

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut animation.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        let abort = ui
                            .with_padded_stripe(true, |ui| {
                                if self.previous_animation != Some(animation.id) {
                                    self.modals.close_all();
                                    self.frame_edit_state.frame_index = self
                                        .frame_edit_state
                                        .frame_index
                                        .min(animation.frames.len().saturating_sub(1));

                                    let atlas = match update_state
                                        .graphics
                                        .atlas_loader
                                        .load_animation_atlas(
                                            &update_state.graphics,
                                            update_state.filesystem,
                                            animation,
                                        ) {
                                        Ok(atlas) => atlas,
                                        Err(e) => {
                                            Self::log_atlas_error(update_state, animation, e);
                                            return true;
                                        }
                                    };

                                    if let Some(frame_view) = &mut self.frame_edit_state.frame_view
                                    {
                                        frame_view.frame.atlas = atlas.clone();
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
                                    );
                                    cellpicker.selected_cell = selected_cell;
                                    self.frame_edit_state.cellpicker = Some(cellpicker);
                                }

                                let (inner_modified, abort) = Self::show_frame_edit(
                                    ui,
                                    update_state,
                                    clip_rect,
                                    &mut self.modals,
                                    animation,
                                    &mut self.frame_edit_state,
                                );

                                modified |= inner_modified;

                                abort
                            })
                            .inner;

                        if abort {
                            return true;
                        }

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "SE and Flash",
                                    |ui: &mut egui::Ui| {
                                        if *update_state.modified_during_prev_frame {
                                            self.collapsing_view.request_sort();
                                        }
                                        if self.previous_animation != Some(animation.id) {
                                            self.collapsing_view.clear_animations();
                                            self.timing_se_picker.close_window();
                                        } else if self.collapsing_view.is_animating() {
                                            self.timing_se_picker.close_window();
                                        }
                                        self.collapsing_view.show_with_sort(
                                            ui,
                                            animation.id,
                                            &mut animation.timings,
                                            |ui, _i, timing| Self::show_timing_header(ui, timing),
                                            |ui, i, timing| {
                                                Self::show_timing_body(
                                                    ui,
                                                    update_state,
                                                    animation.id,
                                                    animation.frame_max,
                                                    &mut self.timing_se_picker,
                                                    &mut self.previous_timing_frame,
                                                    (i, timing),
                                                )
                                            },
                                            |a, b| a.frame.cmp(&b.frame),
                                        )
                                    },
                                ))
                                .changed();
                        });

                        self.previous_animation = Some(animation.id);
                        false
                    },
                )
            });

        if response
            .as_ref()
            .is_some_and(|ir| ir.inner.as_ref().is_some_and(|ir| ir.inner.modified))
        {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            animations.modified = true;
        }

        drop(animations);

        *update_state.data = data; // restore data

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.inner == Some(true))) {
            *open = false;
        }
    }
}
