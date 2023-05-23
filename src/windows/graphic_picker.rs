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

pub struct Graphic {
    pub name: String,
    pub image: Arc<RetainedImage>,
}

pub struct Window<'win> {
    icons: Vec<Graphic>,
    selected_icon: usize,
    icon_mut_ptr: Option<&'win mut String>,
}

impl<'win> Window<'win> {
    #[must_use]
    pub fn new(icons: Vec<String>) -> Self {
        let mut retained_images = Vec::new();

        for icon_path in icons {
            let icon_path = icon_path;
            let split = icon_path.split('.').collect::<Vec<&str>>();

            let icon_path = String::from(split[0]);

            let image = match interfaces!()
                .image_cache
                .load_egui_image("Graphics/Icons", &icon_path)
            {
                Ok(ri) => ri,
                Err(why) => {
                    interfaces!()
                        .toasts
                        .error(format!("Cannot load `{icon_path}` icon: {why}"));
                    continue;
                }
            };
            retained_images.push(Graphic {
                name: icon_path,
                image,
            });
        }

        Self {
            icons: retained_images,
            selected_icon: 0,
            icon_mut_ptr: None,
        }
    }

    pub fn set_active_icon(&mut self, active_icon_index: usize) {
        if active_icon_index < self.icons.len() {
            self.selected_icon = active_icon_index;
        }
    }
    pub fn set_icon_ptr(&mut self, new_ptr: &'win mut String) {
        self.icon_mut_ptr = Some(new_ptr);
    }
}

impl<'win> crate::WindowExt for Window<'win> {
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .id(self.id())
            .resize(|res| res.min_width(480.))
            .open(open)
            .show(ctx, |ui| {
                egui::SidePanel::left(egui::Id::new("item_picker_sidebar")).show_inside(ui, |ui| {
                    egui::ScrollArea::both().max_height(600.).show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        self.icons.len(),
                        |ui, rows| {
                            for (id, icon) in self
                                .icons
                                .iter()
                                .enumerate()
                                .filter(|(ele, _)| rows.contains(ele))
                            {
                                ui.selectable_value(&mut self.selected_icon, id, icon.name.clone());
                                if let Some(icon_mut_ptr) = &mut self.icon_mut_ptr {
                                    **icon_mut_ptr =
                                        self.icons.get(self.selected_icon).unwrap().name.clone();
                                } else {
                                    core::panic!("icon_mut_ptr is not set");
                                }
                            }
                        },
                    );
                });

                let icon = &self.icons[self.selected_icon];
                icon.image.show_scaled(ui, 3.);
            });
    }

    fn name(&self) -> String {
        String::from("Graphic Picker")
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("icon_picker")
    }

    fn requires_filesystem(&self) -> bool {
        false
    }
}

impl<'win> From<Window<'win>> for crate::Window<'win> {
    fn from(value: Window<'win>) -> crate::Window<'win> {
        crate::Window::GraphicPicker(value)
    }
}
