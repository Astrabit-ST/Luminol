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

#[derive(Default, Clone)]
struct IdVecSelectionState {
    pivot: Option<usize>,
    hide_tooltip: bool,
    search_string: String,
}

pub struct IdVecSelection<'a, H, F> {
    id_source: H,
    reference: &'a mut Vec<usize>,
    len: usize,
    formatter: F,
    clear_search: bool,
}

pub struct IdVecPlusMinusSelection<'a, H, F> {
    id_source: H,
    plus: &'a mut Vec<usize>,
    minus: &'a mut Vec<usize>,
    len: usize,
    formatter: F,
    clear_search: bool,
}

impl<'a, H, F> IdVecSelection<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    /// Creates a new widget for changing the contents of an `id_vec`.
    pub fn new(id_source: H, reference: &'a mut Vec<usize>, len: usize, formatter: F) -> Self {
        Self {
            id_source,
            reference,
            len,
            formatter,
            clear_search: false,
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
        id_source: H,
        plus: &'a mut Vec<usize>,
        minus: &'a mut Vec<usize>,
        len: usize,
        formatter: F,
    ) -> Self {
        Self {
            id_source,
            plus,
            minus,
            len,
            formatter,
            clear_search: false,
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
        if !self.reference.is_sorted() {
            self.reference.sort_unstable();
        }

        let state_id = ui.make_persistent_id(egui::Id::new(self.id_source).with("IdVecSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_default();
        if self.clear_search {
            state.search_string = String::new();
        }

        let mut index = 0;
        let mut clicked_id = None;

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let mut response = ui
            .group(|ui| {
                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        ..Default::default()
                    },
                    |ui| {
                        ui.set_width(ui.available_width());

                        ui.add(
                            egui::TextEdit::singleline(&mut state.search_string)
                                .hint_text("Search"),
                        );

                        let mut is_faint = false;

                        for id in 0..self.len {
                            let is_id_selected =
                                self.reference.get(index).is_some_and(|x| *x == id);
                            if is_id_selected {
                                index += 1;
                            }

                            let formatted = (self.formatter)(id);
                            if matcher
                                .fuzzy(&formatted, &state.search_string, false)
                                .is_none()
                            {
                                continue;
                            }

                            let mut frame = egui::Frame::none();
                            if is_faint {
                                frame = frame.fill(ui.visuals().faint_bg_color);
                            }
                            is_faint = !is_faint;

                            frame.show(ui, |ui| {
                                if ui
                                    .selectable_label(is_id_selected, (self.formatter)(id))
                                    .clicked()
                                {
                                    clicked_id = Some(id);
                                }
                            });
                        }
                    },
                )
                .inner
            })
            .response;

        if let Some(clicked_id) = clicked_id {
            state.hide_tooltip = true;

            let modifiers = ui.input(|i| i.modifiers);

            let is_pivot_selected = !modifiers.command
                || state
                    .pivot
                    .is_some_and(|pivot| self.reference.contains(&pivot));

            // Unless control is held, deselect everything before doing anything
            if !modifiers.command {
                self.reference.clear();
            }

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
                        } else if matcher
                            .fuzzy(&(self.formatter)(id), &state.search_string, false)
                            .is_some()
                        {
                            self.reference.push(id);
                        }
                    }
                } else {
                    self.reference.retain(|id| {
                        !range.contains(id)
                            || matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_none()
                    });
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

        if !state.hide_tooltip {
            response = response.on_hover_ui_at_pointer(|ui| {
                ui.label("Click to select single entries");
                ui.label("Ctrl+click to select multiple entries or deselect entries");
                ui.label("Shift+click to select a range");
                ui.label("To select multiple ranges or deselect a range, Ctrl+click the first endpoint and Ctrl+Shift+click the second endpoint");
            });
        }

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
        if !self.plus.is_sorted() {
            self.plus.sort_unstable();
        }
        if !self.minus.is_sorted() {
            self.minus.sort_unstable();
        }

        let state_id =
            ui.make_persistent_id(egui::Id::new(self.id_source).with("IdVecPlusMinusSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_default();
        if self.clear_search {
            state.search_string = String::new();
        }

        let mut plus_index = 0;
        let mut minus_index = 0;
        let mut clicked_id = None;

        let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

        let mut response = ui
            .group(|ui| {
                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        ..Default::default()
                    },
                    |ui| {
                        ui.set_width(ui.available_width());

                        ui.add(
                            egui::TextEdit::singleline(&mut state.search_string)
                                .hint_text("Search"),
                        );

                        let mut is_faint = false;

                        for id in 0..self.len {
                            let is_id_plus = self.plus.get(plus_index).is_some_and(|x| *x == id);
                            if is_id_plus {
                                plus_index += 1;
                            }
                            let is_id_minus = self.minus.get(minus_index).is_some_and(|x| *x == id);
                            if is_id_minus {
                                minus_index += 1;
                            }

                            let formatted = (self.formatter)(id);
                            if matcher
                                .fuzzy(&formatted, &state.search_string, false)
                                .is_none()
                            {
                                continue;
                            }

                            let mut frame = egui::Frame::none();
                            if is_faint {
                                frame = frame.fill(ui.visuals().faint_bg_color);
                            }
                            is_faint = !is_faint;

                            frame.show(ui, |ui| {
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
                                        if is_id_plus {
                                            format!("+ {label}")
                                        } else if is_id_minus {
                                            format!("‒ {label}")
                                        } else {
                                            label
                                        },
                                    )
                                    .clicked()
                                {
                                    clicked_id = Some(id);
                                }
                            });
                        }
                    },
                )
                .inner
            })
            .response;

        if let Some(clicked_id) = clicked_id {
            state.hide_tooltip = true;

            let modifiers = ui.input(|i| i.modifiers);

            // Unless control is held, deselect everything before doing anything
            if !modifiers.command {
                let plus_contains_clicked_id = self.plus.contains(&clicked_id);
                let minus_contains_pivot = state.pivot.and_then(|pivot| {
                    (modifiers.shift && self.minus.contains(&pivot)).then_some(pivot)
                });
                self.plus.clear();
                self.minus.clear();
                if plus_contains_clicked_id {
                    self.plus.push(clicked_id);
                }
                if let Some(pivot) = minus_contains_pivot {
                    self.minus.push(pivot);
                }
            }

            let is_pivot_minus = state.pivot.is_some_and(|pivot| self.minus.contains(&pivot));
            let is_pivot_plus = (!modifiers.command && !is_pivot_minus)
                || state.pivot.is_some_and(|pivot| self.plus.contains(&pivot));

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
                    self.minus.retain(|id| {
                        !range.contains(id)
                            || matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_none()
                    });
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
                        } else if matcher
                            .fuzzy(&(self.formatter)(id), &state.search_string, false)
                            .is_some()
                        {
                            self.plus.push(id);
                        }
                    }
                } else if is_pivot_minus {
                    self.plus.retain(|id| {
                        !range.contains(id)
                            || matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_none()
                    });
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
                        } else if matcher
                            .fuzzy(&(self.formatter)(id), &state.search_string, false)
                            .is_some()
                        {
                            self.minus.push(id);
                        }
                    }
                } else {
                    self.plus.retain(|id| {
                        !range.contains(id)
                            || matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_none()
                    });
                    self.minus.retain(|id| {
                        !range.contains(id)
                            || matcher
                                .fuzzy(&(self.formatter)(*id), &state.search_string, false)
                                .is_none()
                    });
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

        if !state.hide_tooltip {
            response = response.on_hover_ui_at_pointer(|ui| {
                ui.label("Click to select single entries");
                ui.label("Ctrl+click to select multiple entries or deselect entries");
                ui.label("Shift+click to select a range");
                ui.label("To select multiple ranges or deselect a range, Ctrl+click the first endpoint and Ctrl+Shift+click the second endpoint");
            });
        }

        ui.data_mut(|d| d.insert_temp(state_id, state));

        response
    }
}
