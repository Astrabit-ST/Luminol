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
use crate::prelude::*;

/// Database - Items management window.
pub struct Window<'win> {
    // ? Items ?
    items: NilPadded<rpg::Item>,
    selected_item: usize,

    // ? Icon Graphic Picker ?
    icon_picker: graphic_picker::Window<'win>,
    icon_picker_open: bool,

    // ? Menu Sound Effect Picker ?
    menu_se_picker: sound_test::SoundTab,
    menu_se_picker_open: bool,
}

impl<'win> Default for Window<'win> {
    fn default() -> Self {
        let items = interfaces!().data_cache.items().clone();
        let icon_paths = match interfaces!()
            .filesystem
            .dir_children_strings("Graphics/Icons")
        {
            Ok(icons) => icons,
            Err(why) => {
                interfaces!()
                    .toasts
                    .error(format!("Error while reading `Graphics/Icons`: {why}"));
                Vec::new()
            }
        };
        let icon_picker = graphic_picker::Window::new(icon_paths);
        Self {
            items,
            selected_item: 0,

            icon_picker,
            icon_picker_open: false,

            menu_se_picker: sound_test::SoundTab::new(crate::audio::Source::SE, true),
            menu_se_picker_open: false,
        }
    }
}

impl<'win> window::WindowExt for Window<'win> {
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
        let animations = interfaces!().data_cache.animations();

        let common_events = interfaces!().data_cache.commonevents();

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

                    ui.add(Field::new(
                        "Icon",
                        CallbackButton::new(selected_item.icon_name.clone())
                            .on_click(|| self.icon_picker_open = !self.icon_picker_open),
                    ));
                    ui.end_row();

                    ui.add(Field::new(
                        "Description",
                        egui::TextEdit::singleline(&mut selected_item.description),
                    ));
                    ui.end_row();

                    ui.add(Field::new(
                        "Scope",
                        EnumMenuButton::new(selected_item.scope, rpg::ItemScope::None, |scope| {
                            selected_item.scope = scope as i32;
                        }),
                    ));

                    ui.add(Field::new(
                        "Occasion",
                        EnumMenuButton::new(
                            selected_item.occasion,
                            rpg::ItemOccasion::Always,
                            |occasion| {
                                selected_item.occasion = occasion as i32;
                            },
                        ),
                    ));
                    ui.end_row();

                    ui.add(Field::new(
                        "User Animation",
                        NilPaddedMenu::new(&mut selected_item.animation1_id, &*animations),
                    ));
                    ui.add(Field::new(
                        "Target Animation",
                        NilPaddedMenu::new(&mut selected_item.animation2_id, &*animations),
                    ));
                    ui.end_row();

                    ui.add(Field::new(
                        "Menu Use SE",
                        CallbackButton::new(selected_item.menu_se.name.clone()).on_click(|| {
                            self.menu_se_picker_open = true;
                        }),
                    ));
                    ui.add(Field::new(
                        "Common Event",
                        NilPaddedMenu::new(&mut selected_item.common_event_id, &*common_events),
                    ));
                    ui.end_row();

                    egui::Grid::new("item_edit_central_left_grid").show(ui, |_ui| {});
                });

                if self.icon_picker_open {
                    self.icon_picker.set_icon_ptr(&mut selected_item.icon_name);
                    self.icon_picker.show(ctx, &mut self.icon_picker_open);
                }
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

impl<'win> Into<crate::Window<'win>> for Window<'win> {
    fn into(self) -> crate::Window<'win> {
        crate::Window::Items(self)
    }
}
