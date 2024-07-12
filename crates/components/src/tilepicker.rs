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

use luminol_graphics::Renderable;

pub struct Tilepicker {
    pub selected_tiles_left: i16,
    pub selected_tiles_top: i16,
    pub selected_tiles_right: i16,
    pub selected_tiles_bottom: i16,

    pub view: luminol_graphics::Tilepicker,

    drag_origin: Option<egui::Pos2>,

    /// When true, brush tile ID randomization is enabled.
    pub brush_random: bool,
    /// Seed for the PRNG used for the brush when brush tile ID randomization is enabled.
    brush_seed: [u8; 16],
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SelectedTile {
    Autotile(i16),
    Tile(i16),
}

impl SelectedTile {
    pub fn from_id(id: i16) -> Self {
        if id < 384 {
            SelectedTile::Autotile(id / 48)
        } else {
            SelectedTile::Tile(id)
        }
    }

    pub fn to_id(&self) -> i16 {
        match *self {
            Self::Autotile(tile) => tile * 48,
            Self::Tile(tile) => tile,
        }
    }
}

impl Default for SelectedTile {
    fn default() -> Self {
        SelectedTile::Autotile(0)
    }
}

impl Tilepicker {
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        map_id: usize, // FIXME
    ) -> color_eyre::Result<Tilepicker> {
        let map = update_state.data.get_or_load_map(
            map_id,
            update_state.filesystem,
            update_state.project_config.as_ref().unwrap(),
        );
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets.data[map.tileset_id];

        let view = luminol_graphics::Tilepicker::new(
            &update_state.graphics,
            tileset,
            update_state.filesystem,
            false,
        )?;

        let mut brush_seed = [0u8; 16];
        brush_seed[0..8].copy_from_slice(
            &update_state
                .project_config
                .as_ref()
                .expect("project not loaded")
                .project
                .persistence_id
                .to_le_bytes(),
        );
        brush_seed[8..16].copy_from_slice(&(map_id as u64).to_le_bytes());

        Ok(Self {
            view,

            selected_tiles_left: 0,
            selected_tiles_top: 0,
            selected_tiles_right: 0,
            selected_tiles_bottom: 0,

            drag_origin: None,
            brush_seed,
            brush_random: false,
        })
    }

    pub fn get_tile_from_offset(
        &self,
        absolute_x: i16,
        absolute_y: i16,
        absolute_z: i16,
        relative_x: i16,
        relative_y: i16,
    ) -> SelectedTile {
        let width = self.selected_tiles_right - self.selected_tiles_left + 1;
        let height = self.selected_tiles_bottom - self.selected_tiles_top + 1;

        let (x, y) = if self.brush_random {
            let mut preimage = [0u8; 40];
            preimage[0..16].copy_from_slice(&self.brush_seed);
            preimage[16..24].copy_from_slice(&(absolute_x as u64).to_le_bytes());
            preimage[24..32].copy_from_slice(&(absolute_y as u64).to_le_bytes());
            preimage[32..40].copy_from_slice(&(absolute_z as u64).to_le_bytes());
            let image = murmur3::murmur3_32(&mut std::io::Cursor::new(preimage), 5381).unwrap();
            let x = (image & 0xffff) as i16;
            let y = (image >> 16) as i16;
            (
                self.selected_tiles_left
                    + (self.selected_tiles_left + x.rem_euclid(width)).rem_euclid(width),
                self.selected_tiles_top
                    + (self.selected_tiles_top + y.rem_euclid(height)).rem_euclid(height),
            )
        } else {
            (
                self.selected_tiles_left + relative_x.rem_euclid(width),
                self.selected_tiles_top + relative_y.rem_euclid(height),
            )
        };

        match y {
            ..=0 => SelectedTile::Autotile(x),
            _ => SelectedTile::Tile(x + (y - 1) * 8 + 384),
        }
    }

    pub fn ui(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        ui: &mut egui::Ui,
        scroll_rect: egui::Rect,
    ) -> egui::Response {
        self.brush_random = update_state.toolbar.brush_random != ui.input(|i| i.modifiers.alt);

        let (canvas_rect, response) = ui.allocate_exact_size(
            egui::vec2(256., self.view.atlas.tileset_height as f32 + 32.),
            egui::Sense::click_and_drag(),
        );

        let absolute_scroll_rect = ui
            .ctx()
            .screen_rect()
            .intersect(scroll_rect.translate(canvas_rect.min.to_vec2()));
        let scroll_rect = absolute_scroll_rect.translate(-canvas_rect.min.to_vec2());

        self.view.grid.display.set_pixels_per_point(
            &update_state.graphics.render_state,
            ui.ctx().pixels_per_point(),
        );

        self.view.set_position(
            &update_state.graphics.render_state,
            glam::vec2(0.0, -scroll_rect.top()),
        );
        self.view.viewport.set(
            &update_state.graphics.render_state,
            glam::vec2(scroll_rect.width(), scroll_rect.height()),
            glam::Vec2::ZERO,
            glam::Vec2::ONE,
        );
        self.view
            .update_animation(&update_state.graphics.render_state, ui.input(|i| i.time));

        let painter = luminol_graphics::Painter::new(self.view.prepare(&update_state.graphics));
        ui.painter()
            .add(luminol_egui_wgpu::Callback::new_paint_callback(
                absolute_scroll_rect,
                painter,
            ));

        let rect = egui::Rect::from_x_y_ranges(
            (self.selected_tiles_left * 32) as f32..=((self.selected_tiles_right + 1) * 32) as f32,
            (self.selected_tiles_top * 32) as f32..=((self.selected_tiles_bottom + 1) * 32) as f32,
        )
        .translate(canvas_rect.min.to_vec2());
        ui.painter()
            .rect_stroke(rect, 5.0, egui::Stroke::new(1.0, egui::Color32::WHITE));

        let Some(pos) = response.interact_pointer_pos() else {
            return response;
        };
        let pos = ((pos - canvas_rect.min) / 32.).to_pos2();

        if response.is_pointer_button_down_on()
            && ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary))
        {
            let drag_origin = if let Some(drag_origin) = self.drag_origin {
                drag_origin
            } else {
                self.drag_origin = Some(pos);
                pos
            };
            let rect = egui::Rect::from_two_pos(drag_origin, pos);
            let bottom = self.view.atlas.tileset_height as i16 / 32;
            self.selected_tiles_left = (rect.left() as i16).clamp(0, 7);
            self.selected_tiles_right = (rect.right() as i16).clamp(0, 7);
            self.selected_tiles_top = (rect.top() as i16).clamp(0, bottom);
            self.selected_tiles_bottom = (rect.bottom() as i16).clamp(0, bottom);
        } else {
            self.drag_origin = None;
        }

        response
    }
}
