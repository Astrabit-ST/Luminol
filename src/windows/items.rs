// Copyright (C) 2022 Lily Lyons
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

use crate::data::rmxp_structs::rpg;

use super::window::Window;

/// The item edit window.
pub struct ItemsWindow {
    items: Vec<rpg::Item>,
    selected_item: usize,
}

impl ItemsWindow {
    /// Create a new window.
    pub fn new(items: Vec<rpg::Item>) -> Self {
        Self {
            items,
            selected_item: 0,
        }
    }
}

impl Window for ItemsWindow {
    fn name(&self) -> String {
        format!("Editing item {}", self.items[self.selected_item].name)
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        let mut win_open = *open;
        egui::Window::new(self.name())
            .id(egui::Id::new("item_edit_window"))
            .open(&mut win_open)
            .show(ctx, |ui| {
                egui::SidePanel::left("item_edit_panel").show_inside(ui, |ui| {
                    egui::ScrollArea::both().max_height(600.).show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        self.items.len(),
                        |ui, rows| {
                            for (id, item) in self.items[rows].iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.selected_item,
                                    id,
                                    format!("{:0>3}: {}", id, item.name),
                                );
                            }
                        },
                    );

                    ui.horizontal(|ui| {
                        let mut save = false;
                        if ui.button("Ok").clicked() {
                            save = true;
                            *open = false;
                        }

                        if ui.button("Cancel").clicked() {
                            *open = false;
                        }

                        if ui.button("Apply").clicked() {
                            save = true;
                        }

                        if save {
                            *info.data_cache.items() = Some(self.items.clone());
                        }
                    });
                });

                let selected_item = &mut self.items[self.selected_item];

                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut selected_item.name);

                    ui.text_edit_singleline(&mut selected_item.icon_name);
                });

                ui.text_edit_singleline(&mut selected_item.description);
            });
        *open &= win_open;
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
