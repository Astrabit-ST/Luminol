// Copyright (C) 2024 Lily Lyons
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

use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};

#[derive(Debug, Clone)]
#[derive(serde::Deserialize, serde::Serialize)]
pub struct Theme {
    ansi_colors: HashMap<u8, egui::Color32>,
}

// adapted from https://github.com/Harzu/iced_term/blob/master/src/theme.rs

impl Default for Theme {
    fn default() -> Self {
        let mut ansi_colors = HashMap::with_capacity(u8::MAX as usize - 16);

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    // Reserve the first 16 colors for config.
                    let index = 16 + r * 36 + g * 6 + b;
                    let color = egui::Color32::from_rgb(
                        if r == 0 { 0 } else { r * 40 + 55 },
                        if g == 0 { 0 } else { g * 40 + 55 },
                        if b == 0 { 0 } else { b * 40 + 55 },
                    );
                    ansi_colors.insert(index, color);
                }
            }
        }

        for i in 0..24 {
            let value = i * 10 + 8;
            ansi_colors.insert(232 + i, egui::Color32::from_rgb(value, value, value));
        }

        Self { ansi_colors }
    }
}

impl Index<u8> for Theme {
    type Output = egui::Color32;

    fn index(&self, index: u8) -> &Self::Output {
        self.ansi_colors
            .get(&index)
            .expect("no entry found for key")
    }
}

impl IndexMut<u8> for Theme {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        self.ansi_colors
            .get_mut(&index)
            .expect("no entry found for key")
    }
}

impl Theme {
    pub fn get_ansi_color(&self, color: AnsiColor) -> egui::Color32 {
        match color {
            AnsiColor::Spec(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            AnsiColor::Indexed(index) => {
                // maybe allow editing of colors?
                if index <= 15 {
                    return match index {
                        // Default terminal reserved colors
                        0 => egui::Color32::from_rgb(40, 39, 39),
                        1 => egui::Color32::from_rgb(203, 35, 29),
                        2 => egui::Color32::from_rgb(152, 150, 26),
                        3 => egui::Color32::from_rgb(214, 152, 33),
                        4 => egui::Color32::from_rgb(69, 132, 135),
                        5 => egui::Color32::from_rgb(176, 97, 133),
                        6 => egui::Color32::from_rgb(104, 156, 105),
                        7 => egui::Color32::from_rgb(168, 152, 131),
                        // Bright terminal reserved colors
                        8 => egui::Color32::from_rgb(146, 130, 115),
                        9 => egui::Color32::from_rgb(250, 72, 52),
                        10 => egui::Color32::from_rgb(184, 186, 38),
                        11 => egui::Color32::from_rgb(249, 188, 47),
                        12 => egui::Color32::from_rgb(131, 164, 151),
                        13 => egui::Color32::from_rgb(210, 133, 154),
                        14 => egui::Color32::from_rgb(142, 191, 123),
                        15 => egui::Color32::from_rgb(235, 218, 177),
                        _ => egui::Color32::from_rgb(0, 0, 0),
                    };
                }

                // Other colors
                match self.ansi_colors.get(&index) {
                    Some(color) => *color,
                    None => egui::Color32::from_rgb(0, 0, 0),
                }
            }
            AnsiColor::Named(c) => match c {
                NamedColor::Foreground => egui::Color32::from_rgb(235, 218, 177),
                NamedColor::Background => egui::Color32::from_rgb(40, 39, 39),
                // Default terminal reserved colors
                NamedColor::Black => egui::Color32::from_rgb(40, 39, 39),
                NamedColor::Red => egui::Color32::from_rgb(203, 35, 29),
                NamedColor::Green => egui::Color32::from_rgb(152, 150, 26),
                NamedColor::Yellow => egui::Color32::from_rgb(214, 152, 33),
                NamedColor::Blue => egui::Color32::from_rgb(69, 132, 135),
                NamedColor::Magenta => egui::Color32::from_rgb(176, 97, 133),
                NamedColor::Cyan => egui::Color32::from_rgb(104, 156, 105),
                NamedColor::White => egui::Color32::from_rgb(168, 152, 131),
                // Bright terminal reserved colors
                NamedColor::BrightBlack => egui::Color32::from_rgb(146, 130, 115),
                NamedColor::BrightRed => egui::Color32::from_rgb(250, 72, 52),
                NamedColor::BrightGreen => egui::Color32::from_rgb(184, 186, 38),
                NamedColor::BrightYellow => egui::Color32::from_rgb(249, 188, 47),
                NamedColor::BrightBlue => egui::Color32::from_rgb(131, 164, 151),
                NamedColor::BrightMagenta => egui::Color32::from_rgb(210, 133, 154),
                NamedColor::BrightCyan => egui::Color32::from_rgb(142, 191, 123),
                NamedColor::BrightWhite => egui::Color32::from_rgb(235, 218, 177),
                NamedColor::BrightForeground => egui::Color32::from_rgb(235, 218, 177),
                _ => egui::Color32::from_rgb(0, 0, 0),
            },
        }
    }
}
