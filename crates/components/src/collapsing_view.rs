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
}

impl CollapsingView {
    pub fn new() -> Self {
        Default::default()
    }

    /// Cancels all pending animations for expanding and collapsing entries and expands/collapses
    /// them immediately this frame.
    pub fn clear_animations(&mut self) {
        self.disable_animations = true;
    }

    pub fn show<T>(
        &mut self,
        ui: &mut egui::Ui,
        id: usize,
        vec: &mut Vec<T>,
        mut show_header: impl FnMut(&mut egui::Ui, usize, &T),
        mut show_body: impl FnMut(&mut egui::Ui, usize, &mut T) -> egui::Response,
    ) -> egui::Response
    where
        T: Default,
    {
        let mut inner_response = ui.with_cross_justify(|ui| {
            let mut modified = false;
            let mut deleted_entry = None;
            let mut new_entry = false;

            ui.group(|ui| {
                if self.expanded_entry.get(id).is_none() {
                    self.expanded_entry.insert(id, None);
                }
                let expanded_entry = self.expanded_entry.get_mut(id).unwrap();

                for (i, entry) in vec.iter_mut().enumerate() {
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

                    let layout = *ui.layout();
                    let (expand_button_response, _, _) = header
                        .show_header(ui, |ui| {
                            ui.with_layout(layout, |ui| {
                                show_header(ui, i, entry);
                            });
                        })
                        .body(|ui| {
                            ui.with_layout(layout, |ui| {
                                modified |= show_body(ui, i, entry).changed();

                                if ui.button("Delete").clicked() {
                                    modified = true;
                                    deleted_entry = Some(i);
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
                }
            });

            self.disable_animations = false;

            if let Some(i) = deleted_entry {
                if let Some(expanded_entry) = self.expanded_entry.get_mut(id) {
                    if *expanded_entry == Some(i) {
                        self.disable_animations = true;
                        *expanded_entry = None;
                    } else if expanded_entry.is_some() && *expanded_entry > Some(i) {
                        self.disable_animations = true;
                        *expanded_entry = Some(expanded_entry.unwrap() - 1);
                    }
                }

                vec.remove(i);
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
        inner_response.response
    }
}
