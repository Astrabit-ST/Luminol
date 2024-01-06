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
#![feature(is_sorted)]

/// Syntax highlighter
pub mod syntax_highlighting;

/// The tilemap.
mod map_view;
pub use map_view::{MapView, SelectedLayer};
mod tilepicker;
pub use tilepicker::{SelectedTile, Tilepicker};

mod sound_tab;
pub use sound_tab::SoundTab;

mod command_view;
pub use command_view::CommandView;

mod filesystem_view;
pub use filesystem_view::FileSystemView;

mod id_vec;
pub use id_vec::IdVecSelection;

pub struct EnumMenuButton<'e, T> {
    current_value: &'e mut T,
    id: egui::Id,
}

impl<'e, T> EnumMenuButton<'e, T> {
    pub fn new(current_value: &'e mut T, id: egui::Id) -> Self {
        Self { current_value, id }
    }
}

impl<'e, T: ToString + PartialEq + strum::IntoEnumIterator> egui::Widget for EnumMenuButton<'e, T> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        egui::ComboBox::from_id_source(self.id)
            .selected_text(self.current_value.to_string())
            .show_ui(ui, |ui| {
                for variant in T::iter() {
                    let text = variant.to_string();
                    ui.selectable_value(self.current_value, variant, text);
                }
            })
            .response
    }
}

pub struct Field<T> {
    name: String,
    widget: T,
}
impl<T> Field<T>
where
    T: egui::Widget,
{
    /// Creates a new vertical input widget with specified name.
    // * Design notes:
    // * Why not use `ToString` trait in `name` argument? Isn't it specifically built for casting to a string?
    // * Yes, but there's a fundamental differences between `to_string` and `into`, which has to do with move semantics.
    // * TLDR; `to_string` simply creates a string without consuming the original value, which may result in failed to move exceptions.
    // *       `into`, on the other hand, *consumes* the value and converts it to a `String`.
    // * It's not like we're going to use `name` argument after creating the field, so, we can consume it.
    pub fn new(name: impl Into<String>, widget: T) -> Self {
        Self {
            name: name.into(),
            widget,
        }
    }
}

impl<T> egui::Widget for Field<T>
where
    T: egui::Widget,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = ui
            .vertical(|ui| {
                ui.add(egui::Label::new(format!("{}:", self.name)).truncate(true));
                if ui.add(self.widget).changed() {
                    changed = true;
                };
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

pub struct EnumComboBox<'a, T> {
    id_source: egui::Id,
    reference: &'a mut T,
}

impl<'a, T> EnumComboBox<'a, T> {
    /// Creates a combo box that can be used to change the variant of an enum that implements
    /// `strum::IntoEnumIterator + ToString`.
    pub fn new(id_source: impl std::hash::Hash, reference: &'a mut T) -> Self {
        Self {
            id_source: egui::Id::new(id_source),
            reference,
        }
    }
}

impl<'a, T> egui::Widget for EnumComboBox<'a, T>
where
    T: strum::IntoEnumIterator + ToString,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = egui::ComboBox::from_id_source(self.id_source)
            .wrap(true)
            .width(ui.available_width() - ui.spacing().item_spacing.x)
            .selected_text(self.reference.to_string())
            .show_ui(ui, |ui| {
                for (i, variant) in T::iter().enumerate() {
                    let mut frame = egui::Frame::none();
                    if i % 2 != 0 {
                        frame = frame.fill(ui.visuals().faint_bg_color);
                    }
                    frame.show(ui, |ui| {
                        if ui
                            .selectable_label(
                                std::mem::discriminant(self.reference)
                                    == std::mem::discriminant(&variant),
                                variant.to_string(),
                            )
                            .clicked()
                        {
                            *self.reference = variant;
                            changed = true;
                        }
                    });
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

pub struct OptionalIdComboBox<'a, F>
where
    F: Fn(usize) -> String,
{
    id_source: egui::Id,
    reference: &'a mut Option<usize>,
    len: usize,
    formatter: F,
}

impl<'a, F> OptionalIdComboBox<'a, F>
where
    F: Fn(usize) -> String,
{
    /// Creates a combo box that can be used to change the ID of an `optional_id` field in the data
    /// cache.
    pub fn new(
        id_source: impl std::hash::Hash,
        reference: &'a mut Option<usize>,
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

impl<'a, F> egui::Widget for OptionalIdComboBox<'a, F>
where
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = egui::ComboBox::from_id_source(self.id_source)
            .wrap(true)
            .width(ui.available_width() - ui.spacing().item_spacing.x)
            .selected_text(if let Some(id) = *self.reference {
                (self.formatter)(id)
            } else {
                "(None)".into()
            })
            .show_ui(ui, |ui| {
                if ui
                    .selectable_label(self.reference.is_none(), "(None)")
                    .clicked()
                {
                    *self.reference = None;
                    changed = true;
                }
                for id in 0..self.len {
                    let mut frame = egui::Frame::none();
                    if id % 2 == 0 {
                        frame = frame.fill(ui.visuals().faint_bg_color);
                    }
                    frame.show(ui, |ui| {
                        if ui
                            .selectable_label(*self.reference == Some(id), (self.formatter)(id))
                            .clicked()
                        {
                            *self.reference = Some(id);
                            changed = true;
                        }
                    });
                }
            })
            .response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

pub fn close_options_ui(ui: &mut egui::Ui, open: &mut bool, save: &mut bool) {
    ui.horizontal(|ui| {
        if ui.button("Ok").clicked() {
            *open = false;
            *save = true;
        }

        if ui.button("Cancel").clicked() {
            *open = false;
        }

        if ui.button("Apply").clicked() {
            *save = true;
        }
    });
}
