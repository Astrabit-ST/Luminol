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

mod theme;
pub use theme::Theme;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Config {
    pub font: egui::FontId,
    pub initial_size: (u16, u16),
    pub bell_enabled: bool,

    pub cursor_blinking: CursorBlinking,
    pub theme: Theme,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[derive(strum::EnumIter, strum::Display)]
pub enum CursorBlinking {
    #[strum(to_string = "Terminal defined")]
    Terminal,
    Always,
    Never,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: Self::default_font(),
            initial_size: (80, 24),
            bell_enabled: true,
            cursor_blinking: CursorBlinking::Always,
            theme: Theme::default(),
        }
    }
}

impl Config {
    pub fn default_font() -> egui::FontId {
        egui::FontId {
            size: 14.,
            family: egui::FontFamily::Name("Iosevka Term".into()),
        }
    }
}
