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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use crate::prelude::*;

mod state;

#[allow(dead_code)]
pub struct Lumi {
    speak: egui_extras::RetainedImage,
    idle: egui_extras::RetainedImage,
    enabled: bool,
}

impl Lumi {
    pub fn new() -> Result<Self, String> {
        let idle = egui_extras::RetainedImage::from_svg_bytes(
            "lumi_idle",
            include_bytes!("assets/lumi-idle.svg"),
        )?
        .with_options(epaint::textures::TextureOptions::LINEAR);
        let speak = egui_extras::RetainedImage::from_svg_bytes(
            "lumi_speak",
            include_bytes!("assets/lumi-speak.svg"),
        )?
        .with_options(epaint::textures::TextureOptions::LINEAR);

        Ok(Lumi {
            idle,
            speak,
            enabled: false,
        })
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.enabled {
            return;
        }

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            "lumi_layer".into(),
        ));

        let size = self.idle.size_vec2() / 4.0;
        let pos = ctx.screen_rect().max - size;

        painter.image(
            self.idle.texture_id(ctx),
            egui::Rect::from_min_size(pos, size),
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
    }
}
