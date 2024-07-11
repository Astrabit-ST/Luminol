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

#[derive(Default, Clone)]
struct IdVecSelectionState {
    pivot: Option<usize>,
    search_string: String,
    search_matched_ids_lock: std::sync::Arc<parking_lot::Mutex<Vec<usize>>>,
}

pub struct IdVecSelection<'a, H, F> {
    id_source: H,
    reference: &'a mut Vec<usize>,
    id_range: std::ops::Range<usize>,
    formatter: F,
    clear_search: bool,
    search_needs_update: bool,
}

pub struct IdVecPlusMinusSelection<'a, H, F> {
    id_source: H,
    plus: &'a mut Vec<usize>,
    minus: &'a mut Vec<usize>,
    id_range: std::ops::Range<usize>,
    formatter: F,
    clear_search: bool,
    search_needs_update: bool,
}

pub struct RankSelection<'a, H, F> {
    id_source: H,
    reference: &'a mut luminol_data::Table1,
    formatter: F,
    clear_search: bool,
    search_needs_update: bool,
}

impl<'a, H, F> IdVecSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    /// Creates a new widget for changing the contents of an `id_vec`.
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        id_source: H,
        reference: &'a mut Vec<usize>,
        id_range: std::ops::Range<usize>,
        formatter: F,
    ) -> Self {
        Self {
            id_source,
            reference,
            id_range,
            formatter,
            clear_search: false,
            search_needs_update: *update_state.modified_during_prev_frame,
        }
    }

    /// Clears the search box.
    pub fn clear_search(&mut self) {
        self.clear_search = true;
    }
}

impl<'a, H, F> IdVecPlusMinusSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    /// Creates a new widget for changing the contents of a pair of `id_vec`s.
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        id_source: H,
        plus: &'a mut Vec<usize>,
        minus: &'a mut Vec<usize>,
        id_range: std::ops::Range<usize>,
        formatter: F,
    ) -> Self {
        Self {
            id_source,
            plus,
            minus,
            id_range,
            formatter,
            clear_search: false,
            search_needs_update: *update_state.modified_during_prev_frame,
        }
    }

    /// Clears the search box.
    pub fn clear_search(&mut self) {
        self.clear_search = true;
    }
}

impl<'a, H, F> RankSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    /// Creates a new widget for changing the contents of a rank table.
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        id_source: H,
        reference: &'a mut luminol_data::Table1,
        formatter: F,
    ) -> Self {
        Self {
            id_source,
            reference,
            formatter,
            clear_search: false,
            search_needs_update: *update_state.modified_during_prev_frame,
        }
    }

    /// Clears the search box.
    pub fn clear_search(&mut self) {
        self.clear_search = true;
    }
}

impl<'a, H, F> egui::Widget for IdVecSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if self.search_needs_update && luminol_core::slice_is_sorted(self.reference) {
            self.reference.sort_unstable();
        }

        let first_id = self.id_range.start;

        let state_id = ui.make_persistent_id(egui::Id::new(self.id_source).with("IdVecSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_else(|| {
                IdVecSelectionState {
                    // We use a mutex here because if we just put the Vec directly into
                    // memory, egui will clone it every time we get it from memory
                    search_matched_ids_lock: std::sync::Arc::new(parking_lot::Mutex::new(
                        self.id_range.clone().collect_vec(),
                    )),
                    ..Default::default()
                }
            });
        if self.clear_search {
            state.search_string = String::new();
        }
        if self.search_needs_update {
            state.search_matched_ids_lock =
                std::sync::Arc::new(parking_lot::Mutex::new(self.id_range.clone().collect_vec()));
        }
        let mut search_matched_ids = state.search_matched_ids_lock.lock();

        let mut clicked_id = None;

        let mut response = ui
            .group(|ui| {
                ui.with_cross_justify(|ui| {
                    ui.set_width(ui.available_width());

                    let search_box_response = ui.add(
                        egui::TextEdit::singleline(&mut state.search_string).hint_text("Search"),
                    );

                    ui.add_space(ui.spacing().item_spacing.y);

                    // If the user edited the contents of the search box or if the data cache changed
                    // this frame, recalculate the search results
                    let search_needs_update =
                        self.search_needs_update || search_box_response.changed();
                    if search_needs_update {
                        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                        search_matched_ids.clear();
                        search_matched_ids.extend(self.id_range.filter(|id| {
                            matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_some()
                        }));
                    }

                    let button_height = ui.spacing().interact_size.y.max(
                        ui.text_style_height(&egui::TextStyle::Button)
                            + 2. * ui.spacing().button_padding.y,
                    );
                    egui::ScrollArea::vertical()
                        .id_source(state_id.with("scroll_area"))
                        .min_scrolled_height(200.)
                        .show_rows(ui, button_height, search_matched_ids.len(), |ui, range| {
                            let mut is_faint = range.start % 2 != 0;

                            for id in search_matched_ids[range].iter().copied() {
                                let is_id_selected =
                                    self.reference.binary_search(&(id - first_id)).is_ok();

                                ui.with_stripe(is_faint, |ui| {
                                    if ui
                                        .selectable_label(
                                            is_id_selected,
                                            ui.truncate_text((self.formatter)(id)),
                                        )
                                        .clicked()
                                    {
                                        clicked_id = Some(id - first_id);
                                    }
                                });
                                is_faint = !is_faint;
                            }
                        });
                })
                .inner
            })
            .response;

        if let Some(clicked_id) = clicked_id {
            let modifiers = ui.input(|i| i.modifiers);

            let is_pivot_selected = state
                .pivot
                .is_some_and(|pivot| self.reference.contains(&pivot));

            let old_len = self.reference.len();

            // Select all the entries between this one and the pivot if shift is
            // held and the pivot is selected, or deselect them if the pivot is
            // deselected
            if modifiers.shift && state.pivot.is_some() {
                let pivot = state.pivot.unwrap();
                let range = if pivot < clicked_id {
                    pivot..=clicked_id
                } else {
                    clicked_id..=pivot
                };

                if is_pivot_selected {
                    let mut index = self
                        .reference
                        .iter()
                        .position(|id| range.contains(id))
                        .unwrap_or_default();
                    for id in range {
                        let is_id_selected =
                            index < old_len && self.reference.get(index).is_some_and(|x| *x == id);
                        if is_id_selected {
                            index += 1;
                        } else {
                            self.reference.push(id);
                        }
                    }
                } else {
                    self.reference.retain(|id| !range.contains(id));
                }
            } else {
                state.pivot = Some(clicked_id);
                if let Some(position) = self.reference.iter().position(|x| *x == clicked_id) {
                    self.reference.remove(position);
                } else {
                    self.reference.push(clicked_id);
                }
            }

            response.mark_changed();
        }

        drop(search_matched_ids);
        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}

impl<'a, H, F> egui::Widget for IdVecPlusMinusSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if self.search_needs_update {
            if !luminol_core::slice_is_sorted(self.plus) {
                self.plus.sort_unstable();
            }
            if !luminol_core::slice_is_sorted(self.minus) {
                self.minus.sort_unstable();
            }
        }

        let first_id = self.id_range.start;

        let state_id =
            ui.make_persistent_id(egui::Id::new(self.id_source).with("IdVecPlusMinusSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_else(|| {
                IdVecSelectionState {
                    // We use a mutex here because if we just put the Vec directly into
                    // memory, egui will clone it every time we get it from memory
                    search_matched_ids_lock: std::sync::Arc::new(parking_lot::Mutex::new(
                        self.id_range.clone().collect_vec(),
                    )),
                    ..Default::default()
                }
            });
        if self.clear_search {
            state.search_string = String::new();
        }
        if self.search_needs_update {
            state.search_matched_ids_lock =
                std::sync::Arc::new(parking_lot::Mutex::new(self.id_range.clone().collect_vec()));
        }
        let mut search_matched_ids = state.search_matched_ids_lock.lock();

        let mut clicked_id = None;

        let mut response = ui
            .group(|ui| {
                ui.with_cross_justify(|ui| {
                    ui.set_width(ui.available_width());

                    let search_box_response = ui.add(
                        egui::TextEdit::singleline(&mut state.search_string).hint_text("Search"),
                    );

                    ui.add_space(ui.spacing().item_spacing.y);

                    // If the user edited the contents of the search box or if the data cache changed
                    // this frame, recalculate the search results
                    let search_needs_update =
                        self.search_needs_update || search_box_response.changed();
                    if search_needs_update {
                        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                        search_matched_ids.clear();
                        search_matched_ids.extend(self.id_range.filter(|id| {
                            matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_some()
                        }));
                    }

                    let button_height = ui.spacing().interact_size.y.max(
                        ui.text_style_height(&egui::TextStyle::Button)
                            + 2. * ui.spacing().button_padding.y,
                    );
                    egui::ScrollArea::vertical()
                        .id_source(state_id.with("scroll_area"))
                        .min_scrolled_height(200.)
                        .show_rows(ui, button_height, search_matched_ids.len(), |ui, range| {
                            let mut is_faint = range.start % 2 != 0;

                            for id in search_matched_ids[range].iter().copied() {
                                let is_id_plus = self.plus.binary_search(&(id - first_id)).is_ok();
                                let is_id_minus =
                                    self.minus.binary_search(&(id - first_id)).is_ok();

                                ui.with_stripe(is_faint, |ui| {
                                    // Make the background of the selectable label red if it's
                                    // a minus
                                    if is_id_minus {
                                        ui.visuals_mut().selection.bg_fill =
                                            ui.visuals().gray_out(ui.visuals().error_fg_color);
                                    }

                                    let label = (self.formatter)(id);
                                    if ui
                                        .selectable_label(
                                            is_id_plus || is_id_minus,
                                            ui.truncate_text(if is_id_plus {
                                                format!("+ {label}")
                                            } else if is_id_minus {
                                                format!("‒ {label}")
                                            } else {
                                                label
                                            }),
                                        )
                                        .clicked()
                                    {
                                        clicked_id = Some(id - first_id);
                                    }
                                });
                                is_faint = !is_faint;
                            }
                        });
                })
                .inner
            })
            .response;

        if let Some(clicked_id) = clicked_id {
            let modifiers = ui.input(|i| i.modifiers);

            let is_pivot_minus = state.pivot.is_some_and(|pivot| self.minus.contains(&pivot));
            let is_pivot_plus = state.pivot.is_some_and(|pivot| self.plus.contains(&pivot));

            let old_plus_len = self.plus.len();
            let old_minus_len = self.minus.len();

            // Select all the entries between this one and the pivot if shift is
            // held and the pivot is selected, or deselect them if the pivot is
            // deselected
            if modifiers.shift && state.pivot.is_some() {
                let pivot = state.pivot.unwrap();
                let range = if pivot < clicked_id {
                    pivot..=clicked_id
                } else {
                    clicked_id..=pivot
                };

                if is_pivot_plus {
                    self.minus.retain(|id| !range.contains(id));
                    let mut plus_index = self
                        .plus
                        .iter()
                        .position(|id| range.contains(id))
                        .unwrap_or_default();
                    for id in range {
                        let is_id_plus = plus_index < old_plus_len
                            && self.plus.get(plus_index).is_some_and(|x| *x == id);
                        if is_id_plus {
                            plus_index += 1;
                        } else {
                            self.plus.push(id);
                        }
                    }
                } else if is_pivot_minus {
                    self.plus.retain(|id| !range.contains(id));
                    let mut minus_index = self
                        .minus
                        .iter()
                        .position(|id| range.contains(id))
                        .unwrap_or_default();
                    for id in range {
                        let is_id_minus = minus_index < old_minus_len
                            && self.minus.get(minus_index).is_some_and(|x| *x == id);
                        if is_id_minus {
                            minus_index += 1;
                        } else {
                            self.minus.push(id);
                        }
                    }
                } else {
                    self.plus.retain(|id| !range.contains(id));
                    self.minus.retain(|id| !range.contains(id));
                }
            } else {
                state.pivot = Some(clicked_id);
                if let Some(position) = self.plus.iter().position(|x| *x == clicked_id) {
                    self.plus.remove(position);
                    if !self.minus.contains(&clicked_id) {
                        self.minus.push(clicked_id);
                    }
                } else if let Some(position) = self.minus.iter().position(|x| *x == clicked_id) {
                    self.minus.remove(position);
                } else {
                    self.plus.push(clicked_id);
                }
            }

            response.mark_changed();
        }

        drop(search_matched_ids);
        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}

impl<'a, H, F> egui::Widget for RankSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let state_id = ui.make_persistent_id(egui::Id::new(self.id_source).with("RankSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_else(|| {
                IdVecSelectionState {
                    // We use a mutex here because if we just put the Vec directly into
                    // memory, egui will clone it every time we get it from memory
                    search_matched_ids_lock: std::sync::Arc::new(parking_lot::Mutex::new(
                        (0..self.reference.xsize() - 1).collect_vec(),
                    )),
                    ..Default::default()
                }
            });
        if self.clear_search {
            state.search_string = String::new();
        }
        if self.search_needs_update {
            state.search_matched_ids_lock = std::sync::Arc::new(parking_lot::Mutex::new(
                (0..self.reference.xsize() - 1).collect_vec(),
            ));
        }
        let mut search_matched_ids = state.search_matched_ids_lock.lock();

        let mut clicked_id = None;

        let mut response = ui
            .group(|ui| {
                ui.with_cross_justify(|ui| {
                    ui.set_width(ui.available_width());

                    let search_box_response = ui.add(
                        egui::TextEdit::singleline(&mut state.search_string).hint_text("Search"),
                    );

                    ui.add_space(ui.spacing().item_spacing.y);

                    // If the user edited the contents of the search box or if the data cache changed
                    // this frame, recalculate the search results
                    let search_needs_update =
                        self.search_needs_update || search_box_response.changed();
                    if search_needs_update {
                        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();
                        search_matched_ids.clear();
                        search_matched_ids.extend((0..self.reference.xsize() - 1).filter(|id| {
                            matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_some()
                        }));
                    }

                    let button_height = ui.spacing().interact_size.y.max(
                        ui.text_style_height(&egui::TextStyle::Button)
                            + 2. * ui.spacing().button_padding.y,
                    );
                    egui::ScrollArea::vertical()
                        .id_source(state_id.with("scroll_area"))
                        .min_scrolled_height(200.)
                        .show_rows(ui, button_height, search_matched_ids.len(), |ui, range| {
                            let mut is_faint = range.start % 2 != 0;

                            for (id, rank) in search_matched_ids[range]
                                .iter()
                                .copied()
                                .map(|id| (id, self.reference[id + 1]))
                            {
                                ui.with_stripe(is_faint, |ui| {
                                    // Color the background of the selectable label depending on the
                                    // rank
                                    ui.visuals_mut().selection.bg_fill = match rank {
                                        2 => ui.visuals().gray_out(ui.visuals().selection.bg_fill),
                                        4 => ui.visuals().gray_out(ui.visuals().gray_out(
                                            ui.visuals().gray_out(ui.visuals().error_fg_color),
                                        )),
                                        5 => ui.visuals().gray_out(
                                            ui.visuals().gray_out(ui.visuals().error_fg_color),
                                        ),
                                        6 => ui.visuals().gray_out(ui.visuals().error_fg_color),
                                        _ => ui.visuals().selection.bg_fill,
                                    };

                                    let label = (self.formatter)(id);
                                    if ui
                                        .selectable_label(
                                            matches!(rank, 1 | 2 | 4 | 5 | 6),
                                            ui.truncate_text(format!(
                                                "{} - {label}",
                                                match rank {
                                                    1 => 'A',
                                                    2 => 'B',
                                                    3 => 'C',
                                                    4 => 'D',
                                                    5 => 'E',
                                                    6 => 'F',
                                                    _ => '?',
                                                }
                                            )),
                                        )
                                        .clicked()
                                    {
                                        clicked_id = Some(id);
                                    }
                                });
                                is_faint = !is_faint;
                            }
                        });
                })
                .inner
            })
            .response;

        if let Some(clicked_id) = clicked_id {
            let modifiers = ui.input(|i| i.modifiers);

            let pivot_rank = state
                .pivot
                .and_then(|pivot| self.reference.as_slice()[1..].get(pivot).copied());

            // Select all the entries between this one and the pivot if shift is
            // held and the pivot is selected, or deselect them if the pivot is
            // deselected
            if modifiers.shift && state.pivot.is_some() {
                let pivot = state.pivot.unwrap();
                let range = if pivot < clicked_id {
                    pivot..=clicked_id
                } else {
                    clicked_id..=pivot
                };

                for id in range {
                    self.reference[id + 1] = pivot_rank.unwrap_or(3);
                }
            } else {
                if Some(clicked_id) == state.pivot {
                    self.reference[clicked_id + 1] = pivot_rank.unwrap_or(3);
                }
                state.pivot = Some(clicked_id);
                let id = clicked_id + 1;
                self.reference[id] = self.reference[id].saturating_sub(1);
                if self.reference[id] == 0 {
                    self.reference[id] = 6;
                } else if self.reference[id] < 0 || self.reference[id] >= 6 {
                    self.reference[id] = 3;
                }
            }

            response.mark_changed();
        }

        drop(search_matched_ids);
        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}
