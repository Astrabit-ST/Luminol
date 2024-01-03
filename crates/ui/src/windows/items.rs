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
    selected_item: usize,
    selected_item_name: Option<String>,

    // ? Icon Graphic Picker ?
    _icon_picker: Option<luminol_modals::graphic_picker::Modal>,

    // ? Menu Sound Effect Picker ?
    _menu_se_picker: Option<luminol_modals::sound_picker::Modal>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            selected_item: 0,
            selected_item_name: None,

            _icon_picker: None,

            _menu_se_picker: None,
        }
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        if let Some(name) = &self.selected_item_name {
            format!("Editing item {:?}", name)
        } else {
            "Item Editor".into()
        }
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Item Editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut items = update_state.data.items();
        self.selected_item = self.selected_item.min(items.data.len().saturating_sub(1));
        let mut modified = false;

        egui::Window::new(self.name())
            .id(egui::Id::new("item_editor"))
            .default_width(480.)
            .open(open)
            .show(ctx, |ui| {
                egui::SidePanel::left(egui::Id::new("item_edit_sidepanel")).show_inside(ui, |ui| {
                    ui.with_layout(
                        egui::Layout {
                            cross_justify: true,
                            ..Default::default()
                        },
                        |ui| {
                            ui.label("Items");
                            egui::ScrollArea::vertical().max_height(600.).show_rows(
                                ui,
                                ui.text_style_height(&egui::TextStyle::Body),
                                items.data.len(),
                                |ui, rows| {
                                    ui.set_width(ui.available_width());

                                    let offset = rows.start;
                                    for (id, item) in items.data[rows].iter().enumerate() {
                                        let id = id + offset;
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
                        },
                    );
                });

                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        ..Default::default()
                    },
                    |ui| {
                        egui::ScrollArea::vertical()
                            .max_height(600.)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                let selected_item = &mut items.data[self.selected_item];

                                let old_name = selected_item.name.clone();
                                ui.add(luminol_components::Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut selected_item.name)
                                        .desired_width(f32::INFINITY),
                                ));
                                if selected_item.name != old_name {
                                    modified = true;
                                }

                                let old_description = selected_item.description.clone();
                                ui.add(luminol_components::Field::new(
                                    "Description",
                                    egui::TextEdit::multiline(&mut selected_item.description)
                                        .desired_width(f32::INFINITY),
                                ));
                                if selected_item.description != old_description {
                                    modified = true;
                                }
                            });
                    },
                );
            });

        if modified {
            update_state.modified.set(true);
            items.modified = true;
        }
    }
}
