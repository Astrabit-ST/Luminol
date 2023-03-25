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
use num_traits::FromPrimitive;
use std::fmt::Display;
use strum::IntoEnumIterator;

use rmxp_types::rpg;
use rmxp_types::NilPadded;

/// Syntax highlighter
pub mod syntax_highlighting;
/// Toasts to be displayed for errors, information, etc.
pub mod toasts;
/// The toolbar for managing the project.
pub mod top_bar;

pub mod command_view;

/// The tilemap.
pub mod tilemap {
    use crate::UpdateInfo;
    use rmxp_types::rpg;

    cfg_if::cfg_if! {
           if #[cfg(feature = "generic-tilemap")] {
                  mod generic_tilemap;
                  pub use generic_tilemap::Tilemap;
           } else {
                  mod hardware_tilemap;
                  pub use hardware_tilemap::Tilemap;
           }
    }

    /// A trait defining how a tilemap should function.
    pub trait TilemapDef {
        /// Create a new tilemap.
        fn new(info: &'static UpdateInfo, id: i32) -> Self;

        /// Display the tilemap.
        fn ui(
            &mut self,
            ui: &mut egui::Ui,
            map: &rpg::Map,
            cursor_pos: &mut egui::Pos2,
            toggled_layers: &[bool],
            selected_layer: usize,
            dragging_event: bool,
        ) -> egui::Response;

        /// Display the tile picker.
        fn tilepicker(&self, ui: &mut egui::Ui, selected_tile: &mut i16);

        /// Check if the textures are loaded yet.
        fn textures_loaded(&self) -> bool;

        /// Return the result of loading the tilemap.
        fn load_result(&self) -> Result<(), String>;
    }
}

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

pub trait NilPaddedStructure: Default {
    fn name(&self) -> String;
    fn id(&self) -> i32;

    fn set_name(&mut self, new_name: impl Into<String>);
}
impl NilPaddedStructure for rpg::Animation {
    fn id(&self) -> i32 {
        self.id
    }
    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: impl Into<String>) {
        self.name = new_name.into();
    }
}
impl NilPaddedStructure for rpg::CommonEvent {
    fn id(&self) -> i32 {
        self.id as i32
    }
    fn name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, new_name: impl Into<String>) {
        self.name = new_name.into();
    }
}

pub struct NilPaddedMenu<'nil, T>
where
    T: NilPaddedStructure,
{
    pub id: &'nil mut i32,
    pub structure_list: &'nil NilPadded<T>,

    default_structure: T,
}
impl<'nil, T> NilPaddedMenu<'nil, T>
where
    T: NilPaddedStructure,
{
    pub fn new(id: &'nil mut i32, structure_list: &'nil NilPadded<T>) -> Self {
        let mut structure = T::default();
        structure.set_name("(None)");
        Self {
            id,
            structure_list,

            default_structure: structure,
        }
    }
}
impl<'nil, T> egui::Widget for NilPaddedMenu<'nil, T>
where
    T: NilPaddedStructure,
{
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        #[allow(clippy::cast_sign_loss)]
        let id = *self.id as usize;
        ui.menu_button(
            if id == 0 {
                String::from("(None)")
            } else {
                self.structure_list
                    .get(id.checked_sub(1).unwrap_or(id))
                    .unwrap_or(&self.default_structure)
                    .name()
            },
            |ui| {
                egui::ScrollArea::both().max_height(600.).show_rows(
                    ui,
                    ui.text_style_height(&egui::TextStyle::Body),
                    self.structure_list.len(),
                    |ui, rows| {
                        ui.selectable_value(self.id, -1, "000: (None)");
                        for (item_id, item) in self
                            .structure_list
                            .iter()
                            .enumerate()
                            .filter(|(element, _)| rows.contains(element))
                        {
                            let item_id = item_id as i32 + 1;
                            if ui
                                .selectable_value(
                                    self.id,
                                    item_id,
                                    format!("{:0>3}: {}", item_id, item.name()),
                                )
                                .clicked()
                            {
                                ui.close_menu();
                            }
                        }
                    },
                )
            },
        )
        .response
    }
}

/// Wrapper for an `egui` button with callback support.
pub struct CallbackButton<'callback> {
    btn: egui::Button,
    on_click: Option<Box<dyn FnOnce() + 'callback>>,
}
impl<'callback> CallbackButton<'callback> {
    pub fn new(text: impl Into<egui::WidgetText>) -> Self {
        Self {
            btn: egui::Button::new(text),
            on_click: None,
        }
    }

    #[must_use]
    pub fn on_click(mut self, new_on_click_callback: impl FnOnce() + 'callback) -> Self {
        self.on_click = Some(Box::new(new_on_click_callback));
        self
    }
}
impl<'callback> egui::Widget for CallbackButton<'callback> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let response = self.btn.ui(ui);

        if let Some(on_click) = self.on_click {
            if response.clicked() {
                on_click();
            }
        }

        response
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
