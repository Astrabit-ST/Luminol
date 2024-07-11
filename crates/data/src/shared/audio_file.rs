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
use crate::{optional_path_alox, optional_path_serde, Path};

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::AudioFile")]
pub struct AudioFile {
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub name: Path,
    pub volume: u8,
    pub pitch: u8,
}

impl Default for AudioFile {
    fn default() -> Self {
        Self {
            name: None,
            volume: 100,
            pitch: 100,
        }
    }
}
