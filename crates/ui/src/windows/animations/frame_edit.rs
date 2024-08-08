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

use luminol_core::Modal;

use luminol_data::BlendMode;
use luminol_graphics::frame::{FRAME_HEIGHT, FRAME_WIDTH};

pub fn show_frame_edit(
    ui: &mut egui::Ui,
    update_state: &mut luminol_core::UpdateState<'_>,
    clip_rect: egui::Rect,
    modals: &mut super::Modals,
    system: &luminol_data::rpg::System,
    animation: &mut luminol_data::rpg::Animation,
    state: &mut super::FrameEditState,
) -> bool {
    let mut modified = false;
    let mut recompute_flash = false;

    let flash_maps = state.flash_maps.get_mut(animation.id).unwrap();

    let frame_view = if let Some(frame_view) = &mut state.frame_view {
        frame_view
    } else {
        let atlas = update_state.graphics.atlas_loader.load_animation_atlas(
            &update_state.graphics,
            update_state.filesystem,
            animation,
        );
        let mut frame_view = luminol_components::AnimationFrameView::new(update_state, atlas);
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
                    super::util::log_battler_error(update_state, system, animation, e);
                }
            }
        }
        frame_view.frame.update_battler(
            &update_state.graphics,
            system,
            animation,
            Some(
                flash_maps
                    .target(state.condition)
                    .compute(state.frame_index),
            ),
            Some(flash_maps.hide(state.condition).compute(state.frame_index)),
        );
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

    // Handle playing of animations
    if let Some(animation_state) = &mut state.animation_state {
        let time = ui.input(|i| i.time);

        if animation_state.start_time.is_nan() {
            animation_state.start_time = time;
        }

        // Determine what frame in the animation we're at by using the egui time and the
        // framerate
        let previous_frame_index = state.frame_index;
        let time_diff = time - animation_state.start_time;
        state.frame_index = (time_diff * state.animation_fps) as usize;

        if state.frame_index != previous_frame_index {
            recompute_flash = true;
        }

        // Play sound effects
        for (i, timing) in animation.timings[animation_state.timing_index..]
            .iter()
            .enumerate()
        {
            if timing.frame > state.frame_index {
                animation_state.timing_index += i;
                break;
            }
            if !super::util::filter_timing(timing, state.condition) {
                continue;
            }
            if let Some(se_name) = &timing.se.name {
                super::util::load_se(update_state, animation_state, state.condition, timing);
                let Some(Some(audio_data)) = animation_state.audio_data.get(se_name.as_str())
                else {
                    continue;
                };
                if let Err(e) = update_state.audio.play_from_slice(
                    audio_data.clone(),
                    false,
                    timing.se.volume,
                    timing.se.pitch,
                    None,
                    update_state
                        .project_config
                        .as_ref()
                        .expect("project not loaded")
                        .project
                        .volume_scale,
                ) {
                    luminol_core::error!(
                        update_state.toasts,
                        e.wrap_err(format!("Error playing animation sound effect {se_name}"))
                    );
                }
            }
        }
        if animation
            .timings
            .last()
            .is_some_and(|timing| state.frame_index >= timing.frame)
        {
            animation_state.timing_index = animation.timings.len();
        }

        // Request a repaint every few frames
        let frame_delay = state.animation_fps.recip();
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(
                frame_delay - time_diff.rem_euclid(frame_delay),
            ));
    }
    if state.frame_index >= animation.frames.len() {
        let animation_state = state.animation_state.take().unwrap();
        state.frame_index = animation_state.saved_frame_index;
    }

    ui.horizontal(|ui| {
        ui.add(luminol_components::Field::new(
            "Editor Scale",
            egui::Slider::new(&mut frame_view.scale, 15.0..=300.0)
                .suffix("%")
                .logarithmic(true)
                .fixed_decimals(0),
        ));

        let max_frame_index = animation.frames.len().saturating_sub(1);
        if let Some(saved_frame_index) = state.saved_frame_index {
            state.frame_index = saved_frame_index.min(max_frame_index);
        } else if state.frame_index > max_frame_index {
            state.frame_index = max_frame_index;
        }
        state.frame_index += 1;
        let changed = ui
            .add_enabled(
                state.animation_state.is_none(),
                luminol_components::Field::new(
                    "Frame",
                    egui::DragValue::new(&mut state.frame_index).range(1..=animation.frames.len()),
                ),
            )
            .changed();
        state.frame_index -= 1;
        if changed {
            recompute_flash = true;
            state.saved_frame_index = Some(state.frame_index);
        }

        recompute_flash |= ui
            .add(luminol_components::Field::new(
                "Condition",
                luminol_components::EnumComboBox::new("condition", &mut state.condition)
                    .max_width(18.)
                    .wrap_mode(egui::TextWrapMode::Extend),
            ))
            .changed();

        ui.add(luminol_components::Field::new(
            "Onion Skin",
            egui::Checkbox::without_text(&mut state.enable_onion_skin),
        ));

        let old_fps = state.animation_fps;
        let changed = ui
            .add(luminol_components::Field::new(
                "FPS",
                egui::DragValue::new(&mut state.animation_fps).range(0.1..=f64::MAX),
            ))
            .changed();
        if changed {
            // If the animation is playing, recalculate the start time so that the
            // animation playback progress stays the same with the new FPS
            if let Some(animation_state) = &mut state.animation_state {
                if animation_state.start_time.is_finite() {
                    let time = ui.input(|i| i.time);
                    let diff = animation_state.start_time - time;
                    animation_state.start_time = time + diff * (old_fps / state.animation_fps);
                }
            }
        }

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
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);

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

                        if ui.button("Play").clicked() {
                            if let Some(animation_state) = state.animation_state.take() {
                                state.frame_index = animation_state.saved_frame_index;
                            } else {
                                state.animation_state = Some(super::AnimationState {
                                    saved_frame_index: state.frame_index,
                                    start_time: f64::NAN,
                                    timing_index: 0,
                                    audio_data: Default::default(),
                                });
                                state.frame_index = 0;

                                // Preload the audio files used by the animation for
                                // performance reasons
                                for timing in &animation.timings {
                                    super::util::load_se(
                                        update_state,
                                        state.animation_state.as_mut().unwrap(),
                                        state.condition,
                                        timing,
                                    );
                                }
                            }
                        }
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
                                ..=frames[modals.tween.end_frame].cell_data[(i, property)] as f64,
                            (j - modals.tween.start_frame) as f64
                                / (modals.tween.end_frame - modals.tween.start_frame) as f64,
                        ),
                        frames[modals.tween.start_frame].cell_data[(i, property)]
                            <= frames[modals.tween.end_frame].cell_data[(i, property)],
                    )
                };

                if animation.frames[j].cell_data.xsize() < i + 1 {
                    super::util::resize_frame(&mut animation.frames[j], i + 1);
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
                        data[(j, 0)] = ((data[(j, 0)] + 1) as f64 * modals.batch_edit.mul_pattern)
                            .clamp(1., frame_view.frame.atlas.num_patterns() as f64)
                            .round_ties_even() as i16
                            - 1;
                        data[(j, 1)] = (data[(j, 1)] as f64 * modals.batch_edit.mul_x)
                            .clamp(-(FRAME_WIDTH as f64 / 2.), FRAME_WIDTH as f64 / 2.)
                            .round_ties_even() as i16;
                        data[(j, 2)] = (data[(j, 2)] as f64 * modals.batch_edit.mul_y)
                            .clamp(-(FRAME_HEIGHT as f64 / 2.), FRAME_HEIGHT as f64 / 2.)
                            .round_ties_even() as i16;
                        data[(j, 3)] = (data[(j, 3)] as f64 * modals.batch_edit.mul_scale)
                            .clamp(1., i16::MAX as f64)
                            .round_ties_even() as i16;
                        data[(j, 4)] = (data[(j, 4)] as f64 * modals.batch_edit.mul_rotation)
                            .round_ties_even() as i16;
                        if !(0..=360).contains(&data[(j, 4)]) {
                            data[(j, 4)] = data[(j, 4)].rem_euclid(360);
                        }
                        data[(j, 6)] = (data[(j, 6)] as f64 * modals.batch_edit.mul_opacity)
                            .min(255.)
                            .round_ties_even() as i16;
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
        .default_height(240.)
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

    if frame_view.selected_cell_index.is_none()
        && state
            .saved_selected_cell_index
            .is_some_and(|i| i < frame.cell_data.xsize() && frame.cell_data[(i, 0)] >= 0)
    {
        frame_view.selected_cell_index = state.saved_selected_cell_index;
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
    if let (Some(i), Some(drag_pos), true) = (
        frame_view.hovered_cell_index,
        frame_view.hovered_cell_drag_pos,
        state.animation_state.is_none(),
    ) {
        if (frame.cell_data[(i, 1)], frame.cell_data[(i, 2)]) != drag_pos {
            (frame.cell_data[(i, 1)], frame.cell_data[(i, 2)]) = drag_pos;
            frame_view
                .frame
                .update_cell(&update_state.graphics, animation, state.frame_index, i);
            modified = true;
        }
    }

    egui::Frame::none().show(ui, |ui| {
        let frame = &mut animation.frames[state.frame_index];
        if let (Some(i), true) = (
            frame_view.selected_cell_index,
            state.animation_state.is_none(),
        ) {
            let mut properties_modified = false;

            ui.label(format!("Cell {}", i + 1));

            ui.columns(4, |columns| {
                let mut pattern = frame.cell_data[(i, 0)] + 1;
                let changed = columns[0]
                    .add(luminol_components::Field::new(
                        "Pattern",
                        egui::DragValue::new(&mut pattern)
                            .range(1..=frame_view.frame.atlas.num_patterns() as i16),
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
                            .range(-(FRAME_WIDTH as i16 / 2)..=FRAME_WIDTH as i16 / 2),
                    ))
                    .changed();

                properties_modified |= columns[2]
                    .add(luminol_components::Field::new(
                        "Y",
                        egui::DragValue::new(&mut frame.cell_data[(i, 2)])
                            .range(-(FRAME_HEIGHT as i16 / 2)..=FRAME_HEIGHT as i16 / 2),
                    ))
                    .changed();

                properties_modified |= columns[3]
                    .add(luminol_components::Field::new(
                        "Scale",
                        egui::DragValue::new(&mut frame.cell_data[(i, 3)])
                            .range(1..=i16::MAX)
                            .suffix("%"),
                    ))
                    .changed();
            });

            ui.columns(4, |columns| {
                properties_modified |= columns[0]
                    .add(luminol_components::Field::new(
                        "Rotation",
                        egui::DragValue::new(&mut frame.cell_data[(i, 4)])
                            .range(0..=360)
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
                        egui::DragValue::new(&mut frame.cell_data[(i, 6)]).range(0..=255),
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

    if recompute_flash {
        frame_view.frame.update_battler(
            &update_state.graphics,
            system,
            animation,
            Some(
                flash_maps
                    .target(state.condition)
                    .compute(state.frame_index),
            ),
            Some(flash_maps.hide(state.condition).compute(state.frame_index)),
        );
        frame_view
            .frame
            .update_all_cells(&update_state.graphics, animation, state.frame_index);
    }

    egui::ScrollArea::horizontal().show_viewport(ui, |ui, scroll_rect| {
        cellpicker.ui(update_state, ui, scroll_rect);
    });

    ui.allocate_ui_at_rect(canvas_rect, |ui| {
        frame_view.frame.enable_onion_skin =
            state.enable_onion_skin && state.frame_index != 0 && state.animation_state.is_none();
        let egui::InnerResponse {
            inner: hover_pos,
            response,
        } = frame_view.ui(
            ui,
            update_state,
            clip_rect,
            flash_maps
                .screen(state.condition)
                .compute(state.frame_index),
            state.animation_state.is_none(),
        );
        if response.clicked() {
            state.saved_selected_cell_index = frame_view.selected_cell_index;
        }

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

                super::util::resize_frame(frame, next_cell_index + 1);

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
        if let (Some(i), true) = (
            frame_view.selected_cell_index,
            state.animation_state.is_none(),
        ) {
            if i < frame.cell_data.xsize()
                && frame.cell_data[(i, 0)] >= 0
                && response.has_focus()
                && ui.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                })
            {
                frame.cell_data[(i, 0)] = -1;

                if i + 1 >= frame.cell_max {
                    super::util::resize_frame(
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

    modified
}
