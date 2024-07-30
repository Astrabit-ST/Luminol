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

/// A component that shows many copies of a layout and only allows one of them to be expanded at a
/// time.
#[derive(Default)]
pub struct CollapsingView {
    depersisted_entries: usize,
    expanded_entry: luminol_data::OptionVec<Option<usize>>,
    disable_animations: bool,
    is_animating: bool,
    need_sort: bool,
}

#[derive(Clone, Copy)]
pub struct CollapsingViewInner<T> {
    pub created_entry: Option<usize>,
    pub deleted_entry: Option<(usize, T)>,
}

impl<T> Default for CollapsingViewInner<T> {
    fn default() -> Self {
        Self {
            created_entry: None,
            deleted_entry: None,
        }
    }
}

impl CollapsingView {
    pub fn new() -> Self {
        Default::default()
    }

    /// Cancels all pending animations for expanding and collapsing entries and expands/collapses
    /// them immediately this frame.
    pub fn clear_animations(&mut self) {
        self.disable_animations = true;
        self.is_animating = false;
    }

    /// Force the next invocation of `.show_with_sort` to sort `vec`.
    pub fn request_sort(&mut self) {
        self.need_sort = true;
    }

    /// Determines if a collapsing header is currently transitioning from open to closed or
    /// vice-versa. If this is false, it's guaranteed at most one collapsing header body is
    /// currently visible, so there will be at most one call to `show_body`.
    pub fn is_animating(&self) -> bool {
        self.is_animating
    }

    /// Shows the widget with no sorting applied to `vec`.
    ///
    /// ui: `egui::Ui` where the widget should be shown.
    ///
    /// state_id: arbitrary integer that can be used to maintain more than one state for this
    /// widget. This can be useful if you're showing this inside of a `DatabaseView` so that each
    /// database item can get its own state for this widget. If you don't need more than one state,
    /// set this to 0.
    ///
    /// vec: vector containing things that will be passed as argument to `show_header` and
    /// `show_body`.
    ///
    /// show_header: this will be called exactly once for each item in `vec` to draw the headers of
    /// the collapsing headers.
    ///
    /// show_body: this will be called at most once for each item in `vec` to draw the bodies of
    /// the collapsing headers.
    pub fn show<T>(
        &mut self,
        ui: &mut egui::Ui,
        state_id: usize,
        vec: &mut Vec<T>,
        show_header: impl FnMut(&mut egui::Ui, usize, &T),
        mut show_body: impl FnMut(&mut egui::Ui, usize, &mut T) -> egui::Response,
    ) -> egui::InnerResponse<CollapsingViewInner<T>>
    where
        T: Default,
    {
        self.show_impl(
            ui,
            state_id,
            vec,
            show_header,
            |ui, index, _before, item| show_body(ui, index, item),
            |_vec, _expanded_entry| false,
        )
    }

    /// Shows the widget, also using a comparator function to sort `vec` when a new item is added
    /// or when `.request_sort` is called.
    ///
    /// ui: `egui::Ui` where the widget should be shown.
    ///
    /// state_id: arbitrary integer that can be used to maintain more than one state for this
    /// widget. This can be useful if you're showing this inside of a `DatabaseView` so that each
    /// database item can get its own state for this widget. If you don't need more than one state,
    /// set this to 0.
    ///
    /// vec: vector containing things that will be passed as argument to `show_header` and
    /// `show_body`.
    ///
    /// cmp: comparator that will be used to sort the `vec` when a new item is added or when
    /// `.request_sort` is called.
    ///
    /// show_header: this will be called exactly once for each item in `vec` to draw the headers of
    /// the collapsing headers.
    ///
    /// show_body: this will be called at most once for each item in `vec` to draw the bodies of
    /// the collapsing headers.
    pub fn show_with_sort<T>(
        &mut self,
        ui: &mut egui::Ui,
        state_id: usize,
        vec: &mut Vec<T>,
        show_header: impl FnMut(&mut egui::Ui, usize, &T),
        show_body: impl FnMut(&mut egui::Ui, usize, &[T], &mut T) -> egui::Response,
        mut cmp: impl FnMut(&T, &T) -> std::cmp::Ordering,
    ) -> egui::InnerResponse<CollapsingViewInner<T>>
    where
        T: Default,
    {
        self.show_impl(
            ui,
            state_id,
            vec,
            show_header,
            show_body,
            |vec, expanded_entry| {
                // Sort `vec` using the provided comparator function (if applicable) and
                // update `expanded_entry` to account for the sort
                if !luminol_core::slice_is_sorted_by(vec, &mut cmp) {
                    if expanded_entry.is_some() {
                        let (before, after) = vec.split_at(expanded_entry.unwrap());
                        if let Some((cmp_item, after)) = after.split_first() {
                            *expanded_entry = Some(
                                before
                                    .iter()
                                    .filter(|item| {
                                        cmp(item, cmp_item) != std::cmp::Ordering::Greater
                                    })
                                    .count()
                                    + after
                                        .iter()
                                        .filter(|item| {
                                            cmp(item, cmp_item) == std::cmp::Ordering::Less
                                        })
                                        .count(),
                            );
                        } else {
                            *expanded_entry = None;
                        }
                    }

                    vec.sort_by(&mut cmp);
                    true
                } else {
                    false
                }
            },
        )
    }

    fn show_impl<T>(
        &mut self,
        ui: &mut egui::Ui,
        state_id: usize,
        vec: &mut Vec<T>,
        mut show_header: impl FnMut(&mut egui::Ui, usize, &T),
        mut show_body: impl FnMut(&mut egui::Ui, usize, &[T], &mut T) -> egui::Response,
        mut sort_impl: impl FnMut(&mut Vec<T>, &mut Option<usize>) -> bool,
    ) -> egui::InnerResponse<CollapsingViewInner<T>>
    where
        T: Default,
    {
        self.is_animating = false;

        let mut created_entry_index = None;
        let mut deleted_entry_index = None;
        let mut deleted_entry = None;

        let mut inner_response = ui.with_cross_justify(|ui| {
            let mut modified = false;
            let mut new_entry = false;

            ui.group(|ui| {
                if self.expanded_entry.get(state_id).is_none() {
                    self.expanded_entry.insert(state_id, None);
                }
                let expanded_entry = self.expanded_entry.get_mut(state_id).unwrap();

                if self.need_sort {
                    self.need_sort = false;
                    if sort_impl(vec, expanded_entry) {
                        modified = true;
                        self.disable_animations = true;
                    }
                }

                for i in 0..vec.len() {
                    let (before, entry_and_after) = vec.split_at_mut(i);
                    let entry = &mut entry_and_after[0];
                    let ui_id = ui.make_persistent_id(i);

                    // Forget whether the collapsing header was open from the last time
                    // the editor was open
                    let depersisted = i < self.depersisted_entries;
                    if !depersisted {
                        self.depersisted_entries += 1;
                        if let Some(h) =
                            egui::collapsing_header::CollapsingState::load(ui.ctx(), ui_id)
                        {
                            h.remove(ui.ctx());
                        }
                        ui.ctx().animate_bool_with_time(ui_id, false, 0.);
                    }

                    let mut header =
                        egui::collapsing_header::CollapsingState::load_with_default_open(
                            ui.ctx(),
                            ui_id,
                            false,
                        );
                    let expanded =
                        (self.disable_animations || depersisted) && *expanded_entry == Some(i);
                    header.set_open(expanded);
                    if self.disable_animations {
                        ui.ctx().animate_bool_with_time(ui_id, expanded, 0.);
                    }

                    let openness = header.openness(ui.ctx());
                    if openness > 0. && openness < 1. {
                        self.is_animating = true;
                    }

                    let layout = *ui.layout();
                    let (expand_button_response, _, _) = header
                        .show_header(ui, |ui| {
                            ui.with_layout(layout, |ui| {
                                show_header(ui, i, entry);
                            });
                        })
                        .body(|ui| {
                            ui.with_layout(layout, |ui| {
                                modified |= show_body(ui, i, before, entry).changed();

                                if ui.button("Delete").clicked() {
                                    modified = true;
                                    deleted_entry_index = Some(i);
                                }

                                ui.add_space(ui.spacing().item_spacing.y);
                            });
                        });

                    if expand_button_response.clicked() {
                        *expanded_entry = (*expanded_entry != Some(i)).then_some(i);
                    }
                }

                ui.add_space(2. * ui.spacing().item_spacing.y);

                if ui.button("New").clicked() {
                    modified = true;
                    *expanded_entry = Some(vec.len());
                    vec.push(Default::default());
                    new_entry = true;

                    sort_impl(vec, expanded_entry);

                    created_entry_index = *expanded_entry;
                }
            });

            self.disable_animations = false;

            if let Some(i) = deleted_entry_index {
                if let Some(expanded_entry) = self.expanded_entry.get_mut(state_id) {
                    if *expanded_entry == Some(i) {
                        self.disable_animations = true;
                        *expanded_entry = None;
                    } else if expanded_entry.is_some() && *expanded_entry > Some(i) {
                        self.disable_animations = true;
                        *expanded_entry = Some(expanded_entry.unwrap() - 1);
                    }
                }

                deleted_entry = Some(vec.remove(i));
            }

            self.depersisted_entries = vec.len();
            if new_entry {
                self.depersisted_entries -= 1;
            }

            modified
        });

        if inner_response.inner {
            inner_response.response.mark_changed();
        }
        egui::InnerResponse {
            inner: CollapsingViewInner {
                created_entry: created_entry_index,
                deleted_entry: if let (Some(i), Some(e)) = (deleted_entry_index, deleted_entry) {
                    Some((i, e))
                } else {
                    None
                },
            },
            response: inner_response.response,
        }
    }
}
