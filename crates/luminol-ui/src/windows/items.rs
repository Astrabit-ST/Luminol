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

/// Database - Items management window.
pub struct Window {
    // ? Items ?
    items: luminol_data::rpg::Items,
    selected_item: usize,

    // ? Icon Graphic Picker ?
    icon_picker_open: bool,

    // ? Menu Sound Effect Picker ?
    menu_se_picker: luminol_components::SoundTab,
    menu_se_picker_open: bool,
}

impl Default for Window {
    fn default() -> Self {
        let items = state!().data_cache.items().clone();

        Self {
            items,
            selected_item: 0,

            icon_picker_open: false,

            menu_se_picker: sound_test::SoundTab::new(crate::audio::Source::SE, true),
            menu_se_picker_open: false,
        }
    }
}

impl window::Window for Window {
    fn name(&self) -> String {
        format!("Editing item {}", self.items[self.selected_item].name)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Item Editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        let _selected_item = &self.items[self.selected_item];
        let animations = state!().data_cache.animations();

        let common_events = state!().data_cache.common_events();

        /*#[allow(clippy::cast_sign_loss)]
        if animations
            .get(selected_item.animation1_id as usize)
            .is_none()
        {
            info.toasts.error(format!(
                "Tried to get an animation with an ID of `{}`, but it doesn't exist.",
                selected_item.animation1_id
            ));
            return;
        }*/

        egui::Window::new(self.name())
            .id(egui::Id::new("item_editor"))
            .default_width(480.)
            .open(open)
            .show(ctx, |ui| {
                egui::SidePanel::left(egui::Id::new("item_edit_sidepanel")).show_inside(ui, |ui| {
                    ui.label("Items");
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

                    if ui.button("Change maximum...").clicked() {
                        eprintln!("`Change maximum...` button trigger");
                    }
                });
                let selected_item = &mut self.items[self.selected_item];
                egui::Grid::new("item_edit_central_grid").show(ui, |ui| {
                    ui.add(Field::new(
                        "Name",
                        egui::TextEdit::singleline(&mut selected_item.name),
                    ));

                    ui.end_row();

                    ui.add(Field::new(
                        "Description",
                        egui::TextEdit::singleline(&mut selected_item.description),
                    ));
                    ui.end_row();

                    egui::Grid::new("item_edit_central_left_grid").show(ui, |_ui| {});
                });

                if self.menu_se_picker_open {
                    egui::Window::new("Menu Sound Effect Picker")
                        .id(egui::Id::new("menu_se_picker"))
                        .show(ctx, |ui| {
                            self.menu_se_picker.ui(ui);
                        });
                }
            });
    }
}
