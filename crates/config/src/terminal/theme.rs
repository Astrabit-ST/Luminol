// Copyright (C) 2024 Melody Madeline Lyons
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
    pub color_pallette: [egui::Color32; 16],
    pub background_color: egui::Color32,
    pub cursor_color: egui::Color32,
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

        Self {
            ansi_colors,
            background_color: egui::Color32::from_rgb(15, 15, 15),
            cursor_color: egui::Color32::WHITE,
            color_pallette: Self::default_configurable_colors(),
        }
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
    const fn default_configurable_colors() -> [egui::Color32; 16] {
        [
            // Default terminal reserved colors
            egui::Color32::from_rgb(40, 39, 39),
            egui::Color32::from_rgb(203, 35, 29),
            egui::Color32::from_rgb(152, 150, 26),
            egui::Color32::from_rgb(214, 152, 33),
            egui::Color32::from_rgb(69, 132, 135),
            egui::Color32::from_rgb(176, 97, 133),
            egui::Color32::from_rgb(104, 156, 105),
            egui::Color32::from_rgb(168, 152, 131),
            // Bright terminal reserved colors
            egui::Color32::from_rgb(146, 130, 115),
            egui::Color32::from_rgb(250, 72, 52),
            egui::Color32::from_rgb(184, 186, 38),
            egui::Color32::from_rgb(249, 188, 47),
            egui::Color32::from_rgb(131, 164, 151),
            egui::Color32::from_rgb(210, 133, 154),
            egui::Color32::from_rgb(142, 191, 123),
            egui::Color32::from_rgb(235, 218, 177),
        ]
    }

    pub fn get_ansi_color(&self, color: AnsiColor) -> egui::Color32 {
        match color {
            AnsiColor::Spec(rgb) => egui::Color32::from_rgb(rgb.r, rgb.g, rgb.b),
            AnsiColor::Indexed(index) => {
                // maybe allow editing of colors?
                if index <= 15 {
                    return self.color_pallette[index as usize];
                }

                // Other colors
                match self.ansi_colors.get(&index) {
                    Some(color) => *color,
                    None => egui::Color32::from_rgb(0, 0, 0),
                }
            }
            AnsiColor::Named(c) => match c {
                NamedColor::Background => self.background_color,
                NamedColor::Foreground => egui::Color32::from_rgb(235, 218, 177),
                NamedColor::BrightForeground => egui::Color32::from_rgb(235, 218, 177),
                // Default terminal reserved colors
                NamedColor::Black |
                NamedColor::Red |
                NamedColor::Green |
                NamedColor::Yellow |
                NamedColor::Blue |
                NamedColor::Magenta |
                NamedColor::Cyan |
                NamedColor::White |
                // Bright terminal reserved colors
                NamedColor::BrightBlack |
                NamedColor::BrightRed |
                NamedColor::BrightGreen |
                NamedColor::BrightYellow |
                NamedColor::BrightBlue |
                NamedColor::BrightMagenta |
                NamedColor::BrightCyan |
                NamedColor::BrightWhite => self.color_pallette[c as usize],
                // FIXME what do?
                _ => egui::Color32::from_rgb(0, 0, 0),
            },
        }
    }
}
