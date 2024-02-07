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

use crate::UiExt;

pub struct DatabaseViewResponse<R> {
    /// The returned value of the `inner` closure passed to `show` if the editor pane was rendered,
    /// otherwise `None`.
    pub inner: Option<R>,
    /// Was any individual entry or the number of entries modified by us?
    pub modified: bool,
}

#[derive(Default)]
pub struct DatabaseView {
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
        label: impl Into<egui::WidgetText>,
        project_config: &luminol_config::project::Config,
        vec: &mut Vec<T>,
        formatter: impl Fn(&T) -> String,
        inner: impl FnOnce(&mut egui::Ui, &mut T) -> R,
    ) -> egui::InnerResponse<DatabaseViewResponse<R>>
    where
        T: luminol_data::rpg::DatabaseEntry,
    {
        let mut modified = false;

        let p = project_config.project.persistence_id;

        if self.maximum.is_none() {
            self.maximum = Some(vec.len());
        }

        let button_height = ui.spacing().interact_size.y.max(
            ui.text_style_height(&egui::TextStyle::Button) + 2. * ui.spacing().button_padding.y,
        );
        let drag_value_height = ui.spacing().interact_size.y.max(
            ui.text_style_height(&ui.style().drag_value_text_style)
                + 2. * ui.spacing().button_padding.y,
        );

        self.selected_id = self.selected_id.min(vec.len().saturating_sub(1));

        egui::SidePanel::left(ui.make_persistent_id("sidepanel")).show_inside(ui, |ui| {
            ui.with_right_margin(ui.spacing().window_margin.right, |ui| {
                ui.with_cross_justify(|ui| {
                    ui.label(label);
                    egui::ScrollArea::vertical()
                        .id_source(p)
                        .max_height(
                            ui.available_height()
                                - button_height.max(drag_value_height)
                                - 2. * ui.spacing().item_spacing.y,
                        )
                        .show_rows(ui, button_height, vec.len(), |ui, rows| {
                            ui.set_width(ui.available_width());

                            let offset = rows.start;
                            for (id, entry) in vec[rows].iter_mut().enumerate() {
                                let id = id + offset;

                                ui.with_stripe(id % 2 != 0, |ui| {
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
                                            i.key_down(egui::Key::Delete)
                                                || i.key_down(egui::Key::Backspace)
                                        })
                                    {
                                        *entry = Default::default();
                                        modified = true;
                                    }
                                });
                            }
                        });

                    ui.add_space(ui.spacing().item_spacing.y);

                    ui.horizontal(|ui| {
                        ui.style_mut().wrap = Some(true);

                        ui.add(
                            egui::DragValue::new(self.maximum.as_mut().unwrap())
                                .clamp_range(0..=999),
                        );

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
                            inner: vec.get_mut(self.selected_id).map(|entry| inner(ui, entry)),
                            modified,
                        }
                    })
                    .inner
            })
        })
        .inner
    }
}
