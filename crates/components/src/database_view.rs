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

use crate::UiExt;
use itertools::Itertools;

pub struct DatabaseViewResponse<R> {
    /// The returned value of the `inner` closure passed to `show` if the editor pane was rendered,
    /// otherwise `None`.
    pub inner: Option<R>,
    /// Was any individual entry or the number of entries modified by us?
    pub modified: bool,
}

#[derive(Default)]
pub struct DatabaseView {
    show_called_at_least_once: bool,
    selected_id: usize,
    maximum: Option<usize>,
}

impl DatabaseView {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn show<T, R>(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        label: impl Into<egui::WidgetText>,
        vec: &mut Vec<T>,
        formatter: impl Fn(&T) -> String,
        inner: impl FnOnce(&mut egui::Ui, &mut Vec<T>, usize, &mut luminol_core::UpdateState<'_>) -> R,
    ) -> egui::InnerResponse<DatabaseViewResponse<R>>
    where
        T: luminol_data::rpg::DatabaseEntry,
    {
        let mut modified = false;

        let p = update_state
            .project_config
            .as_ref()
            .expect("project not loaded")
            .project
            .persistence_id;

        if self.maximum.is_none() {
            self.maximum = Some(vec.len());
        }

        let button_height = ui.spacing().interact_size.y.max(
            ui.text_style_height(&egui::TextStyle::Button) + 2. * ui.spacing().button_padding.y,
        );

        egui::SidePanel::left(ui.make_persistent_id("sidepanel")).show_inside(ui, |ui| {
            ui.with_right_margin(ui.spacing().window_margin.right, |ui| {
                ui.with_cross_justify(|ui| {
                    ui.with_layout(
                        egui::Layout::bottom_up(ui.layout().horizontal_align()),
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                                ui.add(egui::DragValue::new(self.maximum.as_mut().unwrap()));

                                if ui
                                    .add_enabled(
                                        self.maximum != Some(vec.len()),
                                        egui::Button::new(ui.truncate_text("Set Maximum")),
                                    )
                                    .clicked()
                                {
                                    modified = true;
                                    let mut index = vec.len();
                                    vec.resize_with(self.maximum.unwrap(), || {
                                        let item = T::default_with_id(index);
                                        index += 1;
                                        item
                                    });
                                };
                            });

                            if vec.len() <= 999 && self.maximum.is_some_and(|m| m > 999) {
                                egui::Frame::none().show(ui, |ui| {
                                    ui.style_mut()
                                        .visuals
                                        .widgets
                                        .noninteractive
                                        .bg_stroke
                                        .color = ui.style().visuals.warn_fg_color;
                                    egui::Frame::group(ui.style())
                                        .fill(ui.visuals().gray_out(ui.visuals().gray_out(
                                            ui.visuals().gray_out(ui.style().visuals.warn_fg_color),
                                        )))
                                        .show(ui, |ui| {
                                            ui.set_width(ui.available_width());
                                            ui.label(egui::RichText::new("Setting the maximum above 999 may introduce performance issues and instability").color(ui.style().visuals.warn_fg_color));
                                        });
                                });
                            }

                            ui.add_space(ui.spacing().item_spacing.y);

                            ui.with_layout(
                                egui::Layout::top_down(ui.layout().horizontal_align()),
                                |ui| {
                                    ui.with_cross_justify(|ui| {
                                        ui.label(label);

                                        let state_id = ui.make_persistent_id("DatabaseView");

                                        // Get cached search string and search matches from egui memory
                                        let (mut search_string, search_matched_ids_lock) = self
                                            .show_called_at_least_once
                                            .then(|| ui.data(|d| d.get_temp(state_id)))
                                            .flatten()
                                            .unwrap_or_else(|| {
                                                (
                                                    String::new(),
                                                    // We use a mutex here because if we just put the Vec directly into
                                                    // memory, egui will clone it every time we get it from memory
                                                    std::sync::Arc::new(parking_lot::Mutex::new(
                                                        (0..vec.len()).collect_vec(),
                                                    )),
                                                )
                                            });
                                        let mut search_matched_ids = search_matched_ids_lock.lock();

                                        self.selected_id =
                                            self.selected_id.min(vec.len().saturating_sub(1));

                                        let search_box_response = ui.add(
                                            egui::TextEdit::singleline(&mut search_string)
                                                .hint_text("Search"),
                                        );

                                        ui.add_space(ui.spacing().item_spacing.y);

                                        // If the user edited the contents of the search box or if the data cache changed
                                        // this frame, recalculate the search results
                                        let search_needs_update = modified
                                            || *update_state.modified_during_prev_frame
                                            || search_box_response.changed();
                                        if search_needs_update {
                                            let matcher =
                                                fuzzy_matcher::skim::SkimMatcherV2::default();
                                            search_matched_ids.clear();
                                            search_matched_ids.extend(
                                                vec.iter().enumerate().filter_map(|(id, entry)| {
                                                    matcher
                                                        .fuzzy(
                                                            &formatter(entry),
                                                            &search_string,
                                                            false,
                                                        )
                                                        .is_some()
                                                        .then_some(id)
                                                }),
                                            );
                                        }

                                        egui::ScrollArea::vertical().id_source(p).show_rows(
                                            ui,
                                            button_height,
                                            search_matched_ids.len(),
                                            |ui, range| {
                                                ui.set_width(ui.available_width());

                                                let mut is_faint = range.start % 2 != 0;

                                                for id in search_matched_ids[range].iter().copied()
                                                {
                                                    let entry = &mut vec[id];

                                                    ui.with_stripe(is_faint, |ui| {
                                                        let response = ui
                                                            .selectable_value(
                                                                &mut self.selected_id,
                                                                id,
                                                                ui.truncate_text(formatter(entry)),
                                                            )
                                                            .interact(egui::Sense::click());

                                                        if response.clicked() {
                                                            response.request_focus();
                                                        }

                                                        // Reset this entry if delete or backspace
                                                        // is pressed while this entry is focused
                                                        if response.has_focus()
                                                            && ui.input(|i| {
                                                                i.key_pressed(egui::Key::Delete)
                                                                    || i.key_pressed(
                                                                        egui::Key::Backspace,
                                                                    )
                                                            })
                                                        {
                                                            *entry = T::default_with_id(id);
                                                            modified = true;
                                                        }
                                                    });

                                                    is_faint = !is_faint;
                                                }
                                            },
                                        );

                                        // Save the search string and the search results back into egui memory
                                        drop(search_matched_ids);
                                        ui.data_mut(|d| {
                                            d.insert_temp(
                                                state_id,
                                                (search_string, search_matched_ids_lock),
                                            )
                                        });

                                        self.show_called_at_least_once = true;
                                    });
                                },
                            );
                        },
                    );
                });
            });
        });

        ui.with_left_margin(ui.spacing().window_margin.left, |ui| {
            ui.with_cross_justify(|ui| {
                egui::ScrollArea::vertical()
                    .id_source(p)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_width(
                            2. * (ui.spacing().slider_width + ui.spacing().interact_size.x)
                                + ui.spacing().indent
                                + 12. // `egui::Frame::group` inner margins are hardcoded to 6
                                      // points on each side
                                + 5. * ui.spacing().item_spacing.x,
                        );

                        DatabaseViewResponse {
                            inner: (self.selected_id < vec.len())
                                .then(|| inner(ui, vec, self.selected_id, update_state)),
                            modified,
                        }
                    })
                    .inner
            })
        })
        .inner
    }
}
