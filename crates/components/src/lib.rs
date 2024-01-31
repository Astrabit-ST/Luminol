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
pub use id_vec::{IdVecPlusMinusSelection, IdVecSelection};

mod ui_ext;
pub use ui_ext::UiExt;

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

pub struct EnumComboBox<'a, H, T> {
    id_source: H,
    reference: &'a mut T,
}

impl<'a, H, T> EnumComboBox<'a, H, T>
where
    H: std::hash::Hash,
{
    /// Creates a combo box that can be used to change the variant of an enum that implements
    /// `strum::IntoEnumIterator + ToString`.
    pub fn new(id_source: H, reference: &'a mut T) -> Self {
        Self {
            id_source,
            reference,
        }
    }
}

impl<'a, H, T> egui::Widget for EnumComboBox<'a, H, T>
where
    H: std::hash::Hash,
    T: strum::IntoEnumIterator + ToString,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let mut changed = false;
        let mut response = egui::ComboBox::from_id_source(&self.id_source)
            .wrap(true)
            .width(ui.available_width() - ui.spacing().item_spacing.x)
            .selected_text(self.reference.to_string())
            .show_ui(ui, |ui| {
                for (i, variant) in T::iter().enumerate() {
                    ui.with_stripe(i % 2 != 0, |ui| {
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

pub struct OptionalIdComboBox<'a, H, F> {
    id_source: H,
    reference: &'a mut Option<usize>,
    len: usize,
    formatter: F,
    retain_search: bool,
}

impl<'a, H, F> OptionalIdComboBox<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    /// Creates a combo box that can be used to change the ID of an `optional_id` field in the data
    /// cache.
    pub fn new(id_source: H, reference: &'a mut Option<usize>, len: usize, formatter: F) -> Self {
        Self {
            id_source,
            reference,
            len,
            formatter,
            retain_search: true,
        }
    }

    /// Clears the search box for this combo box.
    pub fn clear_search(&mut self) {
        self.retain_search = false;
    }
}

impl<'a, H, F> egui::Widget for OptionalIdComboBox<'a, H, F>
where
    H: std::hash::Hash,
    F: Fn(usize) -> String,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let state_id = ui
            .make_persistent_id(egui::Id::new(&self.id_source))
            .with("OptionalIdComboBox");

        let mut changed = false;
        let inner_response = egui::ComboBox::from_id_source(&self.id_source)
            .wrap(true)
            .width(ui.available_width() - ui.spacing().item_spacing.x)
            .selected_text(if let Some(id) = *self.reference {
                (self.formatter)(id)
            } else {
                "(None)".into()
            })
            .show_ui(ui, |ui| {
                let mut search_string = self
                    .retain_search
                    .then(|| ui.data(|d| d.get_temp(state_id)))
                    .flatten()
                    .unwrap_or_else(String::new);
                let search_box_response =
                    ui.add(egui::TextEdit::singleline(&mut search_string).hint_text("Search"));
                let search_box_clicked = search_box_response.clicked()
                    || search_box_response.secondary_clicked()
                    || search_box_response.middle_clicked()
                    || search_box_response.clicked_by(egui::PointerButton::Extra1)
                    || search_box_response.clicked_by(egui::PointerButton::Extra2);

                let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if ui
                        .selectable_label(self.reference.is_none(), "(None)")
                        .clicked()
                    {
                        *self.reference = None;
                        changed = true;
                    }

                    let mut is_faint = true;

                    for id in 0..self.len {
                        let formatted = (self.formatter)(id);
                        if matcher.fuzzy(&formatted, &search_string, false).is_none() {
                            continue;
                        }

                        ui.with_stripe(is_faint, |ui| {
                            if ui
                                .selectable_label(*self.reference == Some(id), formatted)
                                .clicked()
                            {
                                *self.reference = Some(id);
                                changed = true;
                            }
                        });
                        is_faint = !is_faint;
                    }
                });

                ui.data_mut(|d| d.insert_temp(state_id, search_string));

                search_box_clicked
            });
        let mut response = inner_response.response;

        if inner_response.inner == Some(true) {
            // Force the combo box to stay open if the search box was clicked
            ui.memory_mut(|m| {
                m.open_popup(
                    ui.make_persistent_id(egui::Id::new(&self.id_source))
                        .with("popup"),
                )
            });
        } else if inner_response.inner.is_none()
            && ui.data(|d| {
                d.get_temp::<String>(state_id)
                    .is_some_and(|s| !s.is_empty())
            })
        {
            // Clear the search box if the combo box is closed
            ui.data_mut(|d| d.insert_temp(state_id, String::new()));
        }

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
