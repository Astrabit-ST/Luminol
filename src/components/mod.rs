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

use num_traits::FromPrimitive;
use std::fmt::Display;
use strum::IntoEnumIterator;

/// Syntax highlighter
pub mod syntax_highlighting;
/// Toasts to be displayed for errors, information, etc.
mod toasts;
/// The toolbar for managing the project.
mod top_bar;

mod command_view;

pub use command_view::CommandView;

pub use tilemap::{Tilemap, TilemapDef};
pub use toasts::Toasts;
pub use top_bar::TopBar;

/// The tilemap.
mod tilemap;
pub use tilemap::{SelectedLayer, SelectedTile, Tilemap};

// btw there's a buncha places this could be used
// uhh in event edit there's an array of strings that gets itered over to do what this does lol
// TODO: Replace dropbox mechanism in event edit with this method
pub trait Enumeration: Display + FromPrimitive + IntoEnumIterator {}
impl<T> Enumeration for T where T: Display + FromPrimitive + IntoEnumIterator {}
pub struct EnumMenuButton<T, F>
where
    T: Enumeration,
    F: FnMut(T),
{
    current_value: i32,
    _enumeration: T,
    on_select: F,
}

impl<T: Enumeration, F: FnMut(T)> EnumMenuButton<T, F> {
    pub fn new(current_value: i32, enumeration: T, on_select: F) -> Self {
        Self {
            current_value,
            _enumeration: enumeration,
            on_select,
        }
    }
}

impl<T: Enumeration, F: FnMut(T)> egui::Widget for EnumMenuButton<T, F> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.menu_button(T::from_i32(self.current_value).unwrap().to_string(), |ui| {
            for enumeration_item in T::iter() {
                if ui.button(enumeration_item.to_string()).clicked() {
                    (self.on_select)(enumeration_item);
                    ui.close_menu();
                }
            }
        })
        .response
    }
}

pub struct Field<T>
where
    T: egui::Widget,
{
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
        ui.vertical(|ui| {
            ui.label(format!("{}:", self.name));
            ui.add(self.widget);
        })
        .response
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
