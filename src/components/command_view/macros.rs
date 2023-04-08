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

#[macro_export]
macro_rules! color_text {
    ($text:expr, $color:expr) => {
        egui::RichText::new($text).monospace().color($color)
    };
}

#[macro_export]
macro_rules! error {
    ($text:expr) => {
        color_text!($text, egui::Color32::RED)
    };
}

#[macro_export]
macro_rules! get_or_resize {
    ($var:expr, $index:expr) => {
        if let Some(v) = $var.get_mut($index) {
            v
        } else {
            $var.resize_with($index + 1, Default::default);
            &mut $var[$index]
        }
    };
}

#[macro_export]
macro_rules! get_or_return {
    ($var:expr, $index:expr) => {
        if let Some(v) = $var.get_mut($index) {
            v
        } else {
            return;
        }
    };
}
