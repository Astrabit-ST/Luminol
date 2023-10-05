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

pub mod command_db;
pub mod global;
pub mod project;

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

impl RMVer {
    pub fn detect_from_filesystem(
        filesystem: &impl luminol_core::filesystem::FileSystem,
    ) -> Option<Self> {
        if filesystem.exists("Data/Actors.rxdata").ok()? {
            return Some(RMVer::XP);
        }

        if filesystem.exists("Data/Actors.rvdata").ok()? {
            return Some(RMVer::VX);
        }

        if filesystem.exists("Data/Actors.rvdata2").ok()? {
            return Some(RMVer::Ace);
        }

        for path in filesystem.read_dir("").ok()? {
            let path = path.path();
            if path.extension() == Some("rgssad") {
                return Some(RMVer::XP);
            }

            if path.extension() == Some("rgss2a") {
                return Some(RMVer::VX);
            }

            if path.extension() == Some("rgss3a") {
                return Some(RMVer::Ace);
            }
        }

        None
    }
}
