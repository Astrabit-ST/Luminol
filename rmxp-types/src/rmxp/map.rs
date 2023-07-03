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
use crate::rpg::{AudioFile, Event, MoveRoute};
use crate::Table3;

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "RPG::Map")]
pub struct Map {
    pub tileset_id: i32,
    pub width: usize,
    pub height: usize,
    pub autoplay_bgm: bool,
    pub bgm: AudioFile,
    pub autoplay_bgs: bool,
    pub bgs: AudioFile,
    pub encounter_list: Vec<i32>,
    pub encounter_step: i32,
    pub data: Table3,
    pub events: slab::Slab<Event>,

    #[serde(skip)]
    /// (direction: i32, start_pos: Pos2, route: MoveRoute)
    pub preview_move_route: Option<(i32, MoveRoute)>,
}
