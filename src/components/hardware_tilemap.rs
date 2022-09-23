// Copyright (C) 2022 Lily Lyons
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

use egui_extras::RetainedImage;
use std::collections::HashMap;

use crate::data::rmxp_structs::rpg;

pub struct Tilemap {
    pub scale: f32,
    pub visible_display: bool,
}

impl Tilemap {
    pub fn new() -> Self {
        Self {
            scale: 100.,
            visible_display: false,
        }
    }

    #[allow(unused_variables)]
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &mut rpg::Map,
        map_id: i32,
        tileset_tex: &RetainedImage,
        autotile_texs: &[Option<RetainedImage>],
        event_texs: &HashMap<String, Option<RetainedImage>>,
    ) {
    }
}
