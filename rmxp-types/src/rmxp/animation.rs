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
use crate::{id, optional_path, rpg::AudioFile, Color, Path, Table2};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Animation")]
pub struct Animation {
    #[serde(with = "id")]
    pub id: usize,
    pub name: String,
    #[serde(with = "optional_path")]
    pub animation_name: Path,
    pub animation_hue: i32,
    pub position: Position,
    pub frame_max: i32,
    pub frames: Vec<Frame>,
    pub timings: Vec<Timing>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Animation::Timing")]
pub struct Timing {
    pub frame: i32,
    pub se: AudioFile,
    pub flash_scope: i32,
    pub flash_color: Color,
    pub flash_duration: i32,
    pub condition: i32,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Animation::Frame")]
pub struct Frame {
    pub cell_max: i32,
    pub cell_data: Table2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
pub enum Position {
    Top = 0,
    #[default]
    Middle = 1,
    Bottom = 2,
    Screen = 3,
}
