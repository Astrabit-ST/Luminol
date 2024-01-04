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
#[derive(Default)]
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
        Default::default()
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
        let change_maximum_text = "Change maximum...";

        let mut items = update_state.data.items();
        self.selected_item = self.selected_item.min(items.data.len().saturating_sub(1));
        self.selected_item_name = items
            .data
            .get(self.selected_item)
            .map(|item| item.name.clone());
        let mut modified = false;

        egui::Window::new(self.name())
            .id(egui::Id::new("item_editor"))
            .default_width(480.)
            .open(open)
            .show(ctx, |ui| {
                let button_height = ui.spacing().interact_size.y.max(
                    ui.text_style_height(&egui::TextStyle::Button)
                        + 2. * ui.spacing().button_padding.y,
                );
                let button_width = ui.spacing().interact_size.x.max(
                    egui::WidgetText::from(change_maximum_text)
                        .into_galley(ui, None, f32::INFINITY, egui::TextStyle::Button)
                        .galley
                        .rect
                        .width()
                        + 2. * ui.spacing().button_padding.x,
                );

                egui::SidePanel::left(egui::Id::new("item_edit_sidepanel")).show_inside(ui, |ui| {
                    egui::Frame::none()
                        .outer_margin(egui::Margin {
                            right: ui.spacing().window_margin.right,
                            ..egui::Margin::ZERO
                        })
                        .show(ui, |ui| {
                            ui.with_layout(
                                egui::Layout {
                                    cross_justify: true,
                                    ..Default::default()
                                },
                                |ui| {
                                    ui.label("Items");
                                    egui::ScrollArea::both()
                                        .min_scrolled_width(
                                            button_width + ui.spacing().item_spacing.x,
                                        )
                                        .max_height(
                                            ui.available_height()
                                                - button_height
                                                - ui.spacing().item_spacing.y,
                                        )
                                        .show_rows(
                                            ui,
                                            button_height,
                                            items.data.len(),
                                            |ui, rows| {
                                                ui.set_width(ui.available_width());

                                                let offset = rows.start;
                                                for (id, item) in
                                                    items.data[rows].iter().enumerate()
                                                {
                                                    let id = id + offset;
                                                    let mut frame = egui::containers::Frame::none();
                                                    if id % 2 != 0 {
                                                        frame =
                                                            frame.fill(ui.visuals().faint_bg_color);
                                                    }

                                                    frame.show(ui, |ui| {
                                                        ui.style_mut().wrap = Some(false);
                                                        ui.selectable_value(
                                                            &mut self.selected_item,
                                                            id,
                                                            format!("{:0>3}: {}", id, item.name),
                                                        );
                                                    });
                                                }
                                            },
                                        );

                                    if ui
                                        .add(egui::Button::new(change_maximum_text).wrap(false))
                                        .clicked()
                                    {
                                        luminol_core::basic!(
                                            update_state.toasts,
                                            "`Change maximum...` button trigger"
                                        );
                                    }
                                },
                            );
                        });
                });

                egui::Frame::none()
                    .outer_margin(egui::Margin {
                        left: ui.spacing().window_margin.left,
                        ..egui::Margin::ZERO
                    })
                    .show(ui, |ui| {
                        ui.with_layout(
                            egui::Layout {
                                cross_justify: true,
                                ..Default::default()
                            },
                            |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    ui.set_width(ui.available_width());

                                    let selected_item = &mut items.data[self.selected_item];

                                    modified |= ui
                                        .add(luminol_components::Field::new(
                                            "Name",
                                            egui::TextEdit::singleline(&mut selected_item.name)
                                                .desired_width(f32::INFINITY),
                                        ))
                                        .changed();

                                    modified |= ui
                                        .add(luminol_components::Field::new(
                                            "Description",
                                            egui::TextEdit::multiline(
                                                &mut selected_item.description,
                                            )
                                            .desired_width(f32::INFINITY),
                                        ))
                                        .changed();

                                    ui.columns(2, |columns| {
                                        modified |= columns[0]
                                            .add(luminol_components::Field::new(
                                                "Scope",
                                                luminol_components::EnumComboBox::new(
                                                    "scope",
                                                    &mut selected_item.scope,
                                                ),
                                            ))
                                            .changed();

                                        modified |= columns[1]
                                            .add(luminol_components::Field::new(
                                                "Occasion",
                                                luminol_components::EnumComboBox::new(
                                                    "occasion",
                                                    &mut selected_item.occasion,
                                                ),
                                            ))
                                            .changed();
                                    });
                                });
                            },
                        );
                    });
            });

        if modified {
            update_state.modified.set(true);
            items.modified = true;
        }
    }
}
