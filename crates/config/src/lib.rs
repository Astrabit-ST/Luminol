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

pub mod command_db;
pub mod global;
pub mod project;
#[cfg(not(target_arch = "wasm32"))]
pub mod terminal;

#[derive(Clone, Copy, Hash, PartialEq, Debug, Default)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(strum::EnumIter, strum::Display)]
pub enum DataFormat {
    #[default]
    #[strum(to_string = "Ruby Marshal")]
    Marshal,
    #[strum(to_string = "RON")]
    Ron { pretty: bool },
    #[strum(to_string = "JSON")]
    Json { pretty: bool },
}

impl DataFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Marshal => "rxdata",
            Self::Ron { .. } => "ron",
            Self::Json { .. } => "json",
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::EnumIter, strum::Display)]
#[allow(missing_docs)]
pub enum RGSSVer {
    #[strum(to_string = "ModShot")]
    ModShot,
    #[strum(to_string = "mkxp-oneshot")]
    MKXPOneShot,
    #[strum(to_string = "rsgss")]
    RSGSS,
    #[strum(to_string = "mkxp")]
    MKXP,
    #[strum(to_string = "mkxp-freebird")]
    MKXPFreebird,
    #[strum(to_string = "mkxp-z")]
    MKXPZ,
    #[default]
    #[strum(to_string = "Stock RGSS1")]
    RGSS1,
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(strum::EnumIter, strum::Display)]
#[allow(missing_docs)]
pub enum RMVer {
    #[default]
    #[strum(to_string = "RPG Maker XP")]
    XP = 1,
    #[strum(to_string = "RPG Maker VX")]
    VX = 2,
    #[strum(to_string = "RPG Maker VX Ace")]
    Ace = 3,
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct CodeTheme {
    pub dark_mode: bool,

    pub syntect_theme: SyntectTheme,
}

impl Default for CodeTheme {
    fn default() -> Self {
        Self::dark()
    }
}

impl CodeTheme {
    #[must_use]
    pub const fn dark() -> Self {
        Self {
            dark_mode: true,
            syntect_theme: SyntectTheme::Base16MochaDark,
        }
    }

    #[must_use]
    pub const fn light() -> Self {
        Self {
            dark_mode: false,
            syntect_theme: SyntectTheme::SolarizedLight,
        }
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Debug)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(strum::EnumIter, strum::Display)]
pub enum SyntectTheme {
    #[strum(to_string = "Base16 Eighties (dark)")]
    Base16EightiesDark,
    #[strum(to_string = "Base16 Mocha (dark)")]
    Base16MochaDark,
    #[strum(to_string = "Base16 Ocean (dark)")]
    Base16OceanDark,
    #[strum(to_string = "Base16 Ocean (light)")]
    Base16OceanLight,
    #[strum(to_string = "InspiredGitHub (light)")]
    InspiredGitHub,
    #[strum(to_string = "Solarized (dark)")]
    SolarizedDark,
    #[strum(to_string = "Solarized (light)")]
    SolarizedLight,
}

impl SyntectTheme {
    pub fn syntect_key_name(self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    pub fn is_dark(self) -> bool {
        match self {
            Self::Base16EightiesDark
            | Self::Base16MochaDark
            | Self::Base16OceanDark
            | Self::SolarizedDark => true,

            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
}
