// Copyright (C) 2022 Lily Lyons
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

use crate::UpdateInfo;
use strum::Display;
use strum::EnumIter;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct Toolbar {
    state: ToolbarState,
}

// TODO: Move to UpdateInfo
#[derive(Default)]
pub struct ToolbarState {
    pub pencil: Pencil,
}

#[derive(Default, EnumIter, Display, PartialEq, Eq, Clone, Copy)]
pub enum Pencil {
    #[default]
    Pen,
    Circle,
    Rectangle,
    Fill,
}

impl Toolbar {
    #[allow(unused_variables)]
    pub fn ui(&mut self, info: &'static UpdateInfo, ui: &mut egui::Ui) {
        for e in Pencil::iter() {
            ui.radio_value(&mut self.state.pencil, e, e.to_string());
        }
    }
}
