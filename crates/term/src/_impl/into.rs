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

use alacritty_terminal::{
    term::color::Colors,
    vte::ansi::{Color, NamedColor},
};

pub fn color_to_egui(color: Color) -> egui::Color32 {
    match color {
        Color::Named(named) => match named {
            NamedColor::Black => egui::Color32::from_rgb(26, 26, 26),
            NamedColor::Red => egui::Color32::from_rgb(128, 0, 0),
            NamedColor::Green => egui::Color32::from_rgb(0, 128, 0),
            NamedColor::Yellow => egui::Color32::from_rgb(128, 128, 0),
            NamedColor::Blue => egui::Color32::from_rgb(0, 0, 128),
            NamedColor::Magenta => egui::Color32::from_rgb(128, 0, 128),
            NamedColor::Cyan => egui::Color32::from_rgb(0, 128, 128),
            NamedColor::White => egui::Color32::from_rgb(128, 128, 128),
            NamedColor::BrightBlack => egui::Color32::from_rgb(48, 48, 48),
            NamedColor::BrightRed => egui::Color32::from_rgb(255, 0, 0),
            NamedColor::BrightGreen => egui::Color32::from_rgb(0, 255, 0),
            NamedColor::BrightYellow => egui::Color32::from_rgb(255, 255, 0),
            NamedColor::BrightBlue => egui::Color32::from_rgb(0, 0, 255),
            NamedColor::BrightMagenta => egui::Color32::from_rgb(255, 0, 255),
            NamedColor::BrightCyan => egui::Color32::from_rgb(0, 255, 255),
            NamedColor::BrightWhite => egui::Color32::from_rgb(255, 255, 255),
            NamedColor::Foreground => egui::Color32::from_rgb(0, 0, 0),
            NamedColor::Background => egui::Color32::from_rgb(0, 0, 0),
            NamedColor::Cursor => egui::Color32::from_rgb(128, 128, 128),
            NamedColor::DimBlack => egui::Color32::from_rgb(0, 0, 0),
            NamedColor::DimRed => egui::Color32::from_rgb(96, 0, 0),
            NamedColor::DimGreen => egui::Color32::from_rgb(0, 96, 0),
            NamedColor::DimYellow => egui::Color32::from_rgb(96, 96, 0),
            NamedColor::DimBlue => egui::Color32::from_rgb(0, 0, 96),
            NamedColor::DimMagenta => egui::Color32::from_rgb(96, 0, 96),
            NamedColor::DimCyan => egui::Color32::from_rgb(0, 96, 96),
            NamedColor::DimWhite => egui::Color32::from_rgb(96, 96, 96),
            NamedColor::BrightForeground => egui::Color32::from_rgb(0, 0, 0),
            NamedColor::DimForeground => egui::Color32::from_rgb(0, 0, 0),
        },
        Color::Spec(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        Color::Indexed(index) => egui::Color32::from_rgb(index, 0, 0),
    }
}
