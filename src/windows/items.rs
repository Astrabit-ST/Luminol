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
use super::{
    graphic_picker::GraphicPicker,
    sound_test::{SoundTab, SoundTest},
    window::Window,
};
use crate::{
    components::enum_menu_button,
    data::{
        nil_padded::NilPadded,
        rmxp_structs::rpg::{self, animation::Animation},
    },
    filesystem::Filesystem,
    UpdateInfo,
};
use num_traits::ToPrimitive;
use poll_promise::Promise;
use strum::IntoEnumIterator;

#[allow(clippy::module_name_repetitions)]
/// Database - Items management window.
pub struct ItemsWindow {
    // ? Items ?
    items: NilPadded<rpg::Item>,
    selected_item: usize,

    // ? Icon Graphic Picker ?
    icon_picker: GraphicPicker,
    icon_picker_open: bool,

    // ? Menu Sound Effect Picker ?
    menu_se_picker: SoundTab,
    menu_se_picker_open: bool,
}

impl ItemsWindow {
    /// Create a new window.
    #[must_use]
    pub fn new(items: NilPadded<rpg::Item>, info: &'static UpdateInfo) -> Self {
        let icon_paths = match Promise::spawn_local(info.filesystem.dir_children("Graphics/Icons"))
            .block_and_take()
        {
            Ok(icons) => icons,
            Err(why) => {
                info.toasts
                    .error(format!("Error while reading `Graphics/Icons`: {why}"));
                Vec::new()
            }
        };
        let icon_picker = GraphicPicker::new(icon_paths, info);
        Self {
            items,
            selected_item: 0,

            icon_picker,
            icon_picker_open: false,

            menu_se_picker: SoundTab::new(crate::audio::Source::SE, info, true),
            menu_se_picker_open: false,
        }
    }
}

impl Window for ItemsWindow {
    fn name(&self) -> String {
        format!("Editing item {}", self.items[self.selected_item].name)
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        let selected_item = &self.items[self.selected_item];
        let animations = info.data_cache.animations();
        let animations = animations.as_ref().unwrap();

        let common_events = info.data_cache.common_events();
        let common_events = common_events.as_ref().unwrap();

        if let None = animations.get(selected_item.animation1_id as usize) {
            info.toasts.error(format!(
                "Tried to get an animation with an ID of `{}`, but it doesn't exist.",
                selected_item.animation1_id
            ));
            return;
        }

        egui::Window::new(self.name())
            .id(egui::Id::new("item_edit_window"))
            .min_width(480.)
            .default_width(480.)
            .resizable(false)
            .collapsible(false)
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
                        println!("`Change maximum...` button trigger");
                    }
                });
                let selected_item = &mut self.items[self.selected_item];
                egui::Grid::new("item_edit_central_grid").show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut selected_item.name);
                    });

                    ui.vertical(|ui| {
                        ui.label("Icon:");
                        if ui.button(selected_item.icon_name.clone()).clicked() {
                            self.icon_picker_open = !self.icon_picker_open;
                        }
                    });
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label("Description");
                        ui.text_edit_singleline(&mut selected_item.description);
                    });
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label("Scope:");
                        enum_menu_button(ui, selected_item.scope, rpg::ItemScope::None, |scope| {
                            selected_item.scope = scope as i32;
                        });
                    });

                    ui.vertical(|ui| {
                        ui.label("Occasion:");
                        enum_menu_button(
                            ui,
                            selected_item.occasion,
                            rpg::ItemOccasion::Always,
                            |occasion| {
                                selected_item.occasion = occasion as i32;
                            },
                        );
                    });
                    ui.end_row();

                    macro_rules! nilpadded_menu {
                        ($ui:expr, $id:expr, $np:expr) => {
                            $ui.menu_button(
                                if $id == 0 {
                                    String::from("(None)")
                                } else {
                                    $np.get($id as usize).unwrap().name.clone()
                                },
                                |ui| {
                                    for item in $np.iter() {
                                        if ui
                                            .button(format!("{}: {}", item.id, item.name.clone()))
                                            .clicked()
                                        {
                                            $id = item.id as i32;
                                        }
                                    }
                                },
                            );
                        };
                    }

                    ui.vertical(|ui| {
                        ui.label("User Animation:");
                        nilpadded_menu!(ui, selected_item.animation1_id, animations);
                    });
                    ui.vertical(|ui| {
                        ui.label("Target Animation:");
                        nilpadded_menu!(ui, selected_item.animation2_id, animations);
                    });
                    ui.end_row();

                    ui.vertical(|ui| {
                        ui.label("Menu Use SE:");
                        if ui.button(selected_item.menu_se.name.clone()).clicked() {
                            self.menu_se_picker_open = true;
                        }
                    });
                    ui.vertical(|ui| {
                        ui.label("Common Event:");
                        nilpadded_menu!(ui, selected_item.common_event_id, common_events);
                    });
                });

                if self.icon_picker_open {
                    self.icon_picker.show(
                        ctx,
                        &mut self.icon_picker_open,
                        info,
                        &mut selected_item.icon_name,
                    );
                }
                if self.menu_se_picker_open {
                    egui::Window::new("Menu Sound Effect Picker")
                        .id(egui::Id::new("menu_se_picker"))
                        .show(ctx, |ui| {
                            self.menu_se_picker.ui(info, ui);
                        });
                }
            });
    }
}
