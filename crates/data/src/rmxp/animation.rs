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
use crate::{
    id_alox, id_serde, optional_path_alox, optional_path_serde, rpg::AudioFile, Color, Path, Table2,
};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Animation")]
pub struct Animation {
    #[serde(with = "id_serde")]
    #[marshal(with = "id_alox")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path_serde")]
    #[marshal(with = "optional_path_alox")]
    pub animation_name: Path,
    pub animation_hue: i32,
    pub position: Position,
    pub frame_max: usize,
    pub frames: Vec<Frame>,
    pub timings: Vec<Timing>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Animation::Timing")]
pub struct Timing {
    pub frame: usize,
    pub se: AudioFile,
    pub flash_scope: Scope,
    pub flash_color: Color,
    pub flash_duration: usize,
    pub condition: Condition,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            frame: 0,
            se: AudioFile::default(),
            flash_scope: Scope::default(),
            flash_color: Color::default(),
            flash_duration: 1,
            condition: Condition::default(),
        }
    }
}

#[derive(Default, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::Animation::Frame")]
pub struct Frame {
    pub cell_max: usize,
    pub cell_data: Table2,
}

impl Frame {
    /// Returns one more than the maximum cell number in this frame.
    #[inline]
    pub fn len(&self) -> usize {
        self.cell_max.min(self.cell_data.xsize())
    }

    /// Returns true if there are no cells in this frame, otherwise false.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cell_max == 0 || self.cell_data.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Position {
    Top = 0,
    #[default]
    Middle = 1,
    Bottom = 2,
    Screen = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Scope {
    #[default]
    None = 0,
    Target = 1,
    Screen = 2,
    #[strum(to_string = "Hide Target")]
    HideTarget = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Condition {
    #[default]
    None = 0,
    Hit = 1,
    Miss = 2,
}
