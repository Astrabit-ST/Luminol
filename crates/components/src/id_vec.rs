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

#[derive(Default, Clone, Copy)]
struct IdVecSelectionState {
    pivot: Option<usize>,
    hide_tooltip: bool,
}

pub struct IdVecSelection<'a, F>
where
    F: Fn(usize) -> String,
{
    id_source: egui::Id,
    reference: &'a mut Vec<usize>,
    len: usize,
    formatter: F,
}

impl<'a, F> IdVecSelection<'a, F>
where
    F: Fn(usize) -> String,
{
    /// Creates a new widget for changing the contents of an `id_vec`.
    pub fn new(
        id_source: impl std::hash::Hash,
        reference: &'a mut Vec<usize>,
        len: usize,
        formatter: F,
    ) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            reference,
            len,
            formatter,
        }
    }
}

impl<'a, F> egui::Widget for IdVecSelection<'a, F>
where
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        if !self.reference.is_sorted() {
            self.reference.sort_unstable();
        }

        let state_id = ui.make_persistent_id(self.id_source.with("IdVecSelection"));
        let mut state = ui
            .data(|d| d.get_temp::<IdVecSelectionState>(state_id))
            .unwrap_or_default();
        let mut index = 0;
        let mut clicked_id = None;

        let mut response = ui
            .group(|ui| {
                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        ..Default::default()
                    },
                    |ui| {
                        egui::ScrollArea::vertical()
                            .id_source(self.id_source)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                for id in 0..self.len {
                                    let mut frame = egui::Frame::none();
                                    if id % 2 != 0 {
                                        frame = frame.fill(ui.visuals().faint_bg_color);
                                    }

                                    let is_id_selected =
                                        self.reference.get(index).is_some_and(|x| *x == id);
                                    if is_id_selected {
                                        index += 1;
                                    }

                                    frame.show(ui, |ui| {
                                        if ui
                                            .selectable_label(is_id_selected, (self.formatter)(id))
                                            .clicked()
                                        {
                                            clicked_id = Some(id);
                                        }
                                    });
                                }
                            })
                            .inner
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
