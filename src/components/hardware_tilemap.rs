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

use egui::{Pos2, Response, Vec2};

use crate::data::rmxp_structs::rpg;

pub struct Textures {}

pub struct Tilemap {
    pub scale: f32,
    pub visible_display: bool,
    pub pan: Vec2,
}

#[allow(dead_code)]
impl Tilemap {
    pub fn new() -> Self {
        Self {
            scale: 100.,
            visible_display: false,
            pan: Vec2::ZERO,
        }
    }

    #[allow(unused_variables)]
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        cursor_pos: &mut Pos2,
        textures: &Textures,
        toggled_layers: &[bool],
        selected_layer: usize,
    ) -> Response {
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();

        let response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        response
    }
}