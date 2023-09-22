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

#![allow(unused_imports)]
use egui::Pos2;
use std::{cell::RefMut, collections::HashMap};

use crate::prelude::*;
use crate::Pencil;

pub struct Tab {
    /// ID of the map that is being edited.
    pub id: usize,
    /// The tilemap.
    pub view: MapView,
    pub tilepicker: Tilepicker,

    dragging_event: bool,
    drawing_shape: bool,
    event_windows: window::Windows,
    force_close: bool,

    /// When event dragging starts, this is set to the difference between
    /// the dragged event's tile and the cursor position
    event_drag_offset: Option<egui::Vec2>,

    layer_cache: Vec<i16>,

    /// This cache is used by the depth-first search when using the fill brush
    dfs_cache: Vec<bool>,
    /// This is used to save a copy of the current layer when using the
    /// rectangle or circle brush
    brush_layer_cache: Vec<i16>,
    /// When drawing with any brush,
    /// this is set to the position of the original tile we began drawing on
    drawing_shape_pos: Option<egui::Pos2>,
}

impl Tab {
    /// Create a new map editor.
    pub fn new(id: usize) -> Result<Self, String> {
        let map = state!().data_cache.map(id);
        let tilesets = state!().data_cache.tilesets();
        let tileset = &tilesets[map.tileset_id];

        Ok(Self {
            id,
            view: MapView::new(&map, tileset)?,
            tilepicker: Tilepicker::new(tileset)?,

            dragging_event: false,
            drawing_shape: false,
            event_windows: window::Windows::default(),
            force_close: false,

            event_drag_offset: None,

            layer_cache: vec![0; map.data.xsize() * map.data.ysize()],

            dfs_cache: vec![false; map.data.xsize() * map.data.ysize()],
            brush_layer_cache: vec![0; map.data.xsize() * map.data.ysize()],
            drawing_shape_pos: None,
        })
    }

    fn recompute_autotile(&self, map: &rpg::Map, position: (usize, usize, usize)) -> i16 {
        if map.data[position] >= 384 {
            return map.data[position];
        }

        let autotile = map.data[position] / 48;
        if autotile == 0 {
            return 0;
        }

        let x_array: [i8; 8] = [-1, 0, 1, 1, 1, 0, -1, -1];
        let y_array: [i8; 8] = [-1, -1, -1, 0, 1, 1, 1, 0];

        /*
         * 765
         * 0 4
         * 123
         */
        let mut bitfield = 0u8;

        // Loop through the 8 neighbors of this position
        for (x, y) in x_array.into_iter().zip(y_array.into_iter()) {
            bitfield <<= 1;
            // Out-of-bounds tiles always count as valid neighbors
            let is_out_of_bounds = ((x == -1 && position.0 == 0)
                || (x == 1 && position.0 + 1 == map.data.xsize()))
                || ((y == -1 && position.1 == 0) || (y == 1 && position.1 + 1 == map.data.ysize()));
            // Otherwise, we only consider neighbors that are autotiles of the same type
            let is_same_autotile = !is_out_of_bounds
                && map.data[(
                    if x == -1 {
                        position.0 - 1
                    } else {
                        position.0 + x as usize
                    },
                    if y == -1 {
                        position.1 - 1
                    } else {
                        position.1 + y as usize
                    },
                    position.2,
                )] / 48
                    == autotile;

            if is_out_of_bounds || is_same_autotile {
                bitfield |= 1
            }
        }

        // Check how many edges have valid neighbors
        autotile * 48
            + match (bitfield & 0b01010101).count_ones() {
                4 => {
                    // If the autotile is surrounded on all 4 edges,
                    // then the autotile variant is one of the first 16,
                    // depending on which corners are surrounded
                    let tl = (bitfield & 0b10000000 == 0) as u8;
                    let tr = (bitfield & 0b00100000 == 0) as u8;
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    let bl = (bitfield & 0b00000010 == 0) as u8;
                    tl | (tr << 1) | (br << 2) | (bl << 3)
                }

                3 => {
                    // Rotate the bitfield 90 degrees counterclockwise until
                    // the one edge that is not surrounded is at the left
                    let mut bitfield = bitfield;
                    let mut i = 16u8;
                    while bitfield & 0b00000001 != 0 {
                        bitfield = bitfield.rotate_left(2);
                        i += 4;
                    }
                    // Now, the variant is one of the next 16
                    let tr = (bitfield & 0b00100000 == 0) as u8;
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    i + (tr | (br << 1))
                }

                // Top and bottom edges
                2 if bitfield & 0b01000100 == 0b01000100 => 32,

                // Left and right edges
                2 if bitfield & 0b00010001 == 0b00010001 => 33,

                2 => {
                    // Rotate the bitfield 90 degrees counterclockwise until
                    // the two edges that are surrounded are at the right and bottom
                    let mut bitfield = bitfield;
                    let mut i = 34u8;
                    while bitfield & 0b00010100 != 0b00010100 {
                        bitfield = bitfield.rotate_left(2);
                        i += 2;
                    }
                    let br = (bitfield & 0b00001000 == 0) as u8;
                    i + br
                }

                1 => {
                    // Rotate the bitfield 90 degrees clockwise until
                    // the edge is at the bottom
                    let mut bitfield = bitfield;
                    let mut i = 42u8;
                    while bitfield & 0b00000100 == 0 {
                        bitfield = bitfield.rotate_right(2);
                        i += 1;
                    }
                    i
                }

                0 => 46,

                _ => unreachable!(),
            } as i16
    }

    fn set_tile(&self, map: &mut rpg::Map, tile: SelectedTile, position: (usize, usize, usize)) {
        map.data[position] = tile.to_id();

        for y in -1i8..=1i8 {
            for x in -1i8..=1i8 {
                // Don't check tiles that are out of bounds
                if ((x == -1 && position.0 == 0) || (x == 1 && position.0 + 1 == map.data.xsize()))
                    || ((y == -1 && position.1 == 0)
                        || (y == 1 && position.1 + 1 == map.data.ysize()))
                {
                    continue;
                }
                let position = (
                    if x == -1 {
                        position.0 - 1
                    } else {
                        position.0 + x as usize
                    },
                    if y == -1 {
                        position.1 - 1
                    } else {
                        position.1 + y as usize
                    },
                    position.2,
                );
                let tile_id = self.recompute_autotile(map, position);
                map.data[position] = tile_id;
            }
        }
    }

    fn add_event(&self, map: &mut rpg::Map) {
        let mut first_vacant_id = 1;
        let mut max_event_id = 0;

        for (_, event) in map.events.iter() {
            if event.id == first_vacant_id {
                first_vacant_id += 1;
            }
            max_event_id = event.id;

            if event.x == self.view.cursor_pos.x as i32 && event.y == self.view.cursor_pos.y as i32
            {
                state!()
                    .toasts
                    .error("Cannot create event on an existing event's tile");
                return;
            }
        }

        // Try first to allocate the event number directly after the current highest one.
        // However, valid event number range in RPG Maker XP and VX is 1-999.
        let new_event_id = if max_event_id < 999 {
            max_event_id + 1
        }
        // Otherwise, we'll try to use a non-allocated event ID that isn't zero.
        else if first_vacant_id <= 999 {
            first_vacant_id
        } else {
            state!()
                .toasts
                .error("Event limit reached, please delete some events");
            return;
        };

        map.events.insert(
            new_event_id,
            rpg::Event::new(
                self.view.cursor_pos.x as i32,
                self.view.cursor_pos.y as i32,
                new_event_id,
            ),
        );

        self.event_windows
            .add_window(event_edit::Window::new(new_event_id, self.id));
    }
}

impl tab::Tab for Tab {
    fn name(&self) -> String {
        let mapinfos = state!().data_cache.mapinfos();
        format!("Map {}: {}", self.id, mapinfos[&self.id].name)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_map").with(self.id)
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        // Display the toolbar.
        egui::TopBottomPanel::top(format!("map_{}_toolbar", self.id)).show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.add(
                    egui::Slider::new(&mut self.view.scale, 15.0..=300.)
                        .text("Scale")
                        .fixed_decimals(0),
                );

                ui.separator();

                ui.menu_button(
                    // Format the text based on what layer is selected.
                    match self.view.selected_layer {
                        SelectedLayer::Events => "Events ⏷".to_string(),
                        SelectedLayer::Tiles(layer) => format!("Layer {} ⏷", layer + 1),
                    },
                    |ui| {
                        // TODO: Add layer enable button
                        // Display all layers.
                        ui.columns(2, |columns| {
                            columns[1].visuals_mut().button_frame = true;
                            columns[0].label(egui::RichText::new("Panorama").underline());
                            columns[1].checkbox(&mut self.view.map.pano_enabled, "👁");

                            for (index, layer) in
                                self.view.map.enabled_layers.iter_mut().enumerate()
                            {
                                columns[0].selectable_value(
                                    &mut self.view.selected_layer,
                                    SelectedLayer::Tiles(index),
                                    format!("Layer {}", index + 1),
                                );
                                columns[1].checkbox(layer, "👁");
                            }

                            // Display event layer.
                            columns[0].selectable_value(
                                &mut self.view.selected_layer,
                                SelectedLayer::Events,
                                egui::RichText::new("Events").italics(),
                            );
                            columns[1].checkbox(&mut self.view.event_enabled, "👁");

                            columns[0].label(egui::RichText::new("Fog").underline());
                            columns[1].checkbox(&mut self.view.map.fog_enabled, "👁");
                        });
                    },
                );

                ui.separator();

                ui.checkbox(&mut self.view.visible_display, "Display Visible Area")
                    .on_hover_text("Display the visible area in-game (640x480)");
                ui.checkbox(&mut self.view.move_preview, "Preview event move routes")
                    .on_hover_text("Preview event page move routes");
                ui.checkbox(&mut self.view.snap_to_grid, "Snap to grid")
                    .on_hover_text("Snap's the viewport to the tile grid");
                ui.checkbox(
                    &mut self.view.darken_unselected_layers,
                    "Darken unselected layers",
                )
                .on_disabled_hover_text("Toggles darkening unselected layers");

                /*
                if ui.button("Save map preview").clicked() {
                    self.tilemap.save_to_disk();
                }

                if map.preview_move_route.is_some()
                && ui.button("Clear move route preview").clicked()
                {
                    map.preview_move_route = None;
                }
                */
            });
        });

        // Display the tilepicker.
        let spacing = ui.spacing();
        let tilepicker_default_width = 256.
            + 3. * spacing.window_margin.left
            + spacing.scroll_bar_inner_margin
            + spacing.scroll_bar_width
            + spacing.scroll_bar_outer_margin;
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(tilepicker_default_width)
            .max_width(tilepicker_default_width)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    self.tilepicker.ui(ui);
                    ui.separator();
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Get the map.
                let mut map = state!().data_cache.map(self.id);

                // Save the state of the selected layer into the cache
                if let SelectedLayer::Tiles(tile_layer) = self.view.selected_layer {
                    for x in 0..map.data.xsize() {
                        for y in 0..map.data.ysize() {
                            self.layer_cache[x + y * map.data.xsize()] =
                                map.data[(x, y, tile_layer)];
                        }
                    }
                }

                let response = self.view.ui(
                    ui,
                    &map,
                    &self.tilepicker,
                    self.dragging_event,
                    self.drawing_shape,
                    self.drawing_shape_pos,
                    matches!(state!().toolbar.borrow().pencil, Pencil::Pen),
                );

                let layers_max = map.data.zsize();
                let map_x = self.view.cursor_pos.x as i32;
                let map_y = self.view.cursor_pos.y as i32;

                if self.dragging_event && self.view.selected_event_id.is_none() {
                    self.dragging_event = false;
                    self.event_drag_offset = None;
                }

                if self.drawing_shape && !response.dragged_by(egui::PointerButton::Primary) {
                    self.drawing_shape = false;
                    self.drawing_shape_pos = None;
                }

                if self.drawing_shape_pos.is_some()
                    && !response.dragged_by(egui::PointerButton::Primary)
                {
                    self.drawing_shape_pos = None;
                }

                if let SelectedLayer::Tiles(tile_layer) = self.view.selected_layer {
                    // Tile drawing
                    let position = (map_x as usize, map_y as usize, tile_layer);
                    let initial_tile = SelectedTile::from_id(map.data[position]);
                    let left = self.tilepicker.selected_tiles_left;
                    let right = self.tilepicker.selected_tiles_right;
                    let top = self.tilepicker.selected_tiles_top;
                    let bottom = self.tilepicker.selected_tiles_bottom;
                    let width = right - left + 1;
                    let height = bottom - top + 1;
                    if response.dragged_by(egui::PointerButton::Primary)
                        && !ui.input(|i| i.modifiers.command)
                    {
                        match state!().toolbar.borrow().pencil {
                            Pencil::Pen => {
                                let drawing_shape_pos =
                                    if let Some(drawing_shape_pos) = self.drawing_shape_pos {
                                        drawing_shape_pos
                                    } else {
                                        self.drawing_shape_pos = Some(self.view.cursor_pos);
                                        self.view.cursor_pos
                                    };
                                for y in 0..height {
                                    for x in 0..width {
                                        self.set_tile(
                                            &mut map,
                                            self.tilepicker.get_tile_from_offset(
                                                x + (self.view.cursor_pos.x - drawing_shape_pos.x)
                                                    as i16,
                                                y + (self.view.cursor_pos.y - drawing_shape_pos.y)
                                                    as i16,
                                            ),
                                            (
                                                map_x as usize + x as usize,
                                                map_y as usize + y as usize,
                                                tile_layer,
                                            ),
                                        );
                                    }
                                }
                            }

                            Pencil::Fill
                                if initial_tile == self.tilepicker.get_tile_from_offset(0, 0) => {}
                            Pencil::Fill => {
                                // Use depth-first search to find all of the orthogonally
                                // contiguous matching tiles
                                let mut stack = vec![position; 1];
                                let initial_x = position.0;
                                let initial_y = position.1;
                                while let Some(position) = stack.pop() {
                                    self.set_tile(
                                        &mut map,
                                        self.tilepicker.get_tile_from_offset(
                                            position.0 as i16 - initial_x as i16,
                                            position.1 as i16 - initial_y as i16,
                                        ),
                                        position,
                                    );
                                    self.dfs_cache[position.0 + position.1 * map.data.xsize()] =
                                        true;

                                    let x_array: [i8; 4] = [-1, 1, 0, 0];
                                    let y_array: [i8; 4] = [0, 0, -1, 1];
                                    for (x, y) in x_array.into_iter().zip(y_array.into_iter()) {
                                        // Don't search tiles that are out of bounds
                                        if ((x == -1 && position.0 == 0)
                                            || (x == 1 && position.0 + 1 == map.data.xsize()))
                                            || ((y == -1 && position.1 == 0)
                                                || (y == 1 && position.1 + 1 == map.data.ysize()))
                                        {
                                            continue;
                                        }

                                        let position = (
                                            if x == -1 {
                                                position.0 - 1
                                            } else {
                                                position.0 + x as usize
                                            },
                                            if y == -1 {
                                                position.1 - 1
                                            } else {
                                                position.1 + y as usize
                                            },
                                            position.2,
                                        );

                                        // Don't search tiles that we've already searched before
                                        // because that would cause an infinite loop
                                        if self.dfs_cache
                                            [position.0 + position.1 * map.data.xsize()]
                                        {
                                            continue;
                                        }

                                        if SelectedTile::from_id(map.data[position]) == initial_tile
                                        {
                                            stack.push(position);
                                        }
                                    }
                                }

                                for x in self.dfs_cache.iter_mut() {
                                    *x = false;
                                }
                            }

                            Pencil::Rectangle => {
                                if !self.drawing_shape {
                                    // Save the current layer
                                    for x in 0..map.data.xsize() {
                                        for y in 0..map.data.ysize() {
                                            self.brush_layer_cache[x + y * map.data.xsize()] =
                                                map.data[(x, y, tile_layer)];
                                        }
                                    }
                                    self.drawing_shape = true;
                                } else {
                                    // Restore the previously stored state of the current layer
                                    for y in 0..map.data.ysize() {
                                        for x in 0..map.data.xsize() {
                                            map.data[(x, y, tile_layer)] =
                                                self.brush_layer_cache[x + y * map.data.xsize()];
                                        }
                                    }
                                }

                                if let Some(drawing_shape_pos) = self.drawing_shape_pos {
                                    let bounding_rect = egui::Rect::from_two_pos(
                                        drawing_shape_pos,
                                        self.view.cursor_pos,
                                    );
                                    for y in (bounding_rect.min.y as usize)
                                        ..=(bounding_rect.max.y as usize)
                                    {
                                        for x in (bounding_rect.min.x as usize)
                                            ..=(bounding_rect.max.x) as usize
                                        {
                                            let position = (x, y, tile_layer);
                                            self.set_tile(
                                                &mut map,
                                                self.tilepicker.get_tile_from_offset(
                                                    x as i16 - drawing_shape_pos.x as i16,
                                                    y as i16 - drawing_shape_pos.y as i16,
                                                ),
                                                position,
                                            );
                                        }
                                    }
                                } else {
                                    self.drawing_shape_pos = Some(self.view.cursor_pos);
                                }
                            }

                            Pencil::Circle => {
                                if !self.drawing_shape {
                                    // Save the current layer
                                    for x in 0..map.data.xsize() {
                                        for y in 0..map.data.ysize() {
                                            self.brush_layer_cache[x + y * map.data.xsize()] =
                                                map.data[(x, y, tile_layer)];
                                        }
                                    }
                                    self.drawing_shape = true;
                                } else {
                                    // Restore the previously stored state of the current layer
                                    for y in 0..map.data.ysize() {
                                        for x in 0..map.data.xsize() {
                                            map.data[(x, y, tile_layer)] =
                                                self.brush_layer_cache[x + y * map.data.xsize()];
                                        }
                                    }
                                }

                                // Use Bresenham's algorithm to draw the ellipse.
                                // We consider (x, y) to be the top-left corner of the tile at
                                // (x, y).
                                if let Some(drawing_shape_pos) = self.drawing_shape_pos {
                                    let bounding_rect = egui::Rect::from_two_pos(
                                        drawing_shape_pos,
                                        self.view.cursor_pos,
                                    );
                                    // Edge case: Bresenham's algorithm breaks down when drawing a
                                    // 1x1 ellipse.
                                    if drawing_shape_pos == self.view.cursor_pos {
                                        self.set_tile(
                                            &mut map,
                                            self.tilepicker.get_tile_from_offset(
                                                map_x as i16 - drawing_shape_pos.x as i16,
                                                map_y as i16 - drawing_shape_pos.y as i16,
                                            ),
                                            (map_x as usize, map_y as usize, tile_layer),
                                        );
                                    } else {
                                        let bounding_rect =
                                            bounding_rect.translate(egui::vec2(0.5, 0.5));

                                        // Calculate where the center of the ellipse should be.
                                        let x0 = bounding_rect.center().x;
                                        let y0 = bounding_rect.center().y;

                                        // Calculate the radii of the ellipse along the
                                        // x and y directions.
                                        let rx = bounding_rect.width() / 2.;
                                        let ry = bounding_rect.height() / 2.;
                                        let rx2 = rx * rx;
                                        let ry2 = ry * ry;

                                        // Let the "ellipse function" be defined as
                                        // f(x, y) = b^2 x^2 + a^2 y^2 - a^2 b^2
                                        // where a is the x-radius of an ellipse centered at (0, 0)
                                        // and b is the y-radius.
                                        // This function is positive when (x, y) is outside the
                                        // ellipse, negative when it's inside the ellipse and zero when
                                        // it's exactly on the edge.

                                        // We'll start by drawing the part of the outer edge of the
                                        // bottom-right quadrant of the ellipse where dy/dx >= -1,
                                        // starting from the bottom of the ellipse and going to the
                                        // right.
                                        let mut x = if rx.floor() == rx { 0. } else { 0.5 };
                                        let mut y = ry;

                                        // Keep looping until dy/dx < -1.
                                        while rx2 * y >= ry2 * x {
                                            for i in ((-y).floor() as i32)..=(y.floor() as i32) {
                                                let i = if y.floor() == y {
                                                    i as f32
                                                } else {
                                                    i as f32 + 0.5
                                                };
                                                for j in [x, -x] {
                                                    let x = (x0 + j).floor();
                                                    let y = (y0 + i).floor();
                                                    self.set_tile(
                                                        &mut map,
                                                        self.tilepicker.get_tile_from_offset(
                                                            x as i16 - drawing_shape_pos.x as i16,
                                                            y as i16 - drawing_shape_pos.y as i16,
                                                        ),
                                                        (x as usize, y as usize, tile_layer),
                                                    );
                                                }
                                            }

                                            // The next tile will either be at (x + 1, y) or
                                            // (x + 1, y - 1), whichever is closest to the actual edge
                                            // of the ellipse.
                                            // To determine which is closer, we evaluate the ellipse
                                            // function at (x + 1, y - 0.5).
                                            // If it's positive, then (x + 1, y - 1) is closer.
                                            // If it's negative, then (x + 1, y) is closer.
                                            let f = ry2 * (x + 1.).powi(2)
                                                + rx2 * (y - 0.5).powi(2)
                                                - rx2 * ry2;
                                            if f > 0. {
                                                y -= 1.;
                                            }
                                            x += 1.;
                                        }

                                        // Now we draw the part of the outer edge of the
                                        // bottom-right quadrant of the ellipse where dy/dx <= -1,
                                        // starting from the right of the ellipse and going down.
                                        let mut x = rx;
                                        let mut y = if ry.floor() == ry { 0. } else { 0.5 };

                                        // Keep looping until dy/dx > -1.
                                        while rx2 * y <= ry2 * x {
                                            for i in ((-x).floor() as i32)..=(x.floor() as i32) {
                                                let i = if x.floor() == x {
                                                    i as f32
                                                } else {
                                                    i as f32 + 0.5
                                                };
                                                for j in [y, -y] {
                                                    let x = (x0 + i).floor();
                                                    let y = (y0 + j).floor();
                                                    self.set_tile(
                                                        &mut map,
                                                        self.tilepicker.get_tile_from_offset(
                                                            x as i16 - drawing_shape_pos.x as i16,
                                                            y as i16 - drawing_shape_pos.y as i16,
                                                        ),
                                                        (x as usize, y as usize, tile_layer),
                                                    );
                                                }
                                            }

                                            // The next tile will either be at (x, y + 1) or
                                            // (x - 1, y + 1), whichever is closest to the actual edge
                                            // of the ellipse.
                                            // To determine which is closer, we evaluate the ellipse
                                            // function at (x - 0.5, y + 1).
                                            // If it's positive, then (x - 1, y + 1) is closer.
                                            // If it's negative, then (x, y + 1) is closer.
                                            let f = ry2 * (x - 0.5).powi(2)
                                                + rx2 * (y + 1.).powi(2)
                                                - rx2 * ry2;
                                            if f > 0. {
                                                x -= 1.;
                                            }
                                            y += 1.;
                                        }
                                    }
                                } else {
                                    self.drawing_shape_pos = Some(self.view.cursor_pos);
                                }
                            }
                        };
                    }
                } else if let Some(selected_event_id) = self.view.selected_event_id {
                    if response.double_clicked()
                        || (response.hovered()
                            && ui.memory(|m| m.focus().is_none())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        // Double-click/press enter on events to edit them
                        if ui.input(|i| !i.modifiers.command) {
                            self.dragging_event = false;
                            self.event_drag_offset = None;
                            self.event_windows
                                .add_window(event_edit::Window::new(selected_event_id, self.id));
                        }
                    }
                    // Allow drag and drop to move events
                    else if !self.dragging_event
                        && self.view.selected_event_is_hovered
                        && response.drag_started_by(egui::PointerButton::Primary)
                    {
                        self.dragging_event = true;
                    } else if self.dragging_event
                        && !response.dragged_by(egui::PointerButton::Primary)
                    {
                        self.dragging_event = false;
                        self.event_drag_offset = None;
                    }

                    // Press delete or backspace to delete the selected event
                    if response.hovered()
                        && ui.memory(|m| m.focus().is_none())
                        && ui.input(|i| {
                            i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                        })
                    {
                        map.events.remove(selected_event_id);
                        let _ = self.view.events.try_remove(selected_event_id);
                    }

                    if let Some(hover_tile) = self.view.hover_tile {
                        if self.dragging_event {
                            if let Some(selected_event) = map.events.get(selected_event_id) {
                                // If we just started dragging an event, save the offset between the
                                // cursor and the event's tile so that the event will be dragged
                                // with that offset from the cursor
                                if self.event_drag_offset.is_none() {
                                    self.event_drag_offset = Some(
                                        egui::Pos2::new(
                                            selected_event.x as f32,
                                            selected_event.y as f32,
                                        ) - hover_tile,
                                    );
                                };
                            }

                            if let Some(offset) = self.event_drag_offset {
                                // If moving an event, move the dragged event's tile to the cursor
                                // after adjusting for drag offset, unless that would put the event
                                // on the same tile as an existing event
                                let adjusted_hover_tile = hover_tile + offset;
                                if !map.events.iter().any(|(_, e)| {
                                    adjusted_hover_tile.x == e.x as f32
                                        && adjusted_hover_tile.y == e.y as f32
                                }) {
                                    if let Some(selected_event) =
                                        map.events.get_mut(selected_event_id)
                                    {
                                        selected_event.x = adjusted_hover_tile.x as i32;
                                        selected_event.y = adjusted_hover_tile.y as i32;
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Double-click/press enter on an empty space to add an event
                    // (hold shift to prevent events from being selected)
                    if response.double_clicked()
                        || (response.hovered()
                            && ui.memory(|m| m.focus().is_none())
                            && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        self.dragging_event = false;
                        self.event_drag_offset = None;
                        self.add_event(&mut map);
                    }
                }

                for (_, event) in map.events.iter_mut() {
                    event.extra_data.is_editor_open = false;
                }

                // Write the buffered tile changes to the tilemap
                if let SelectedLayer::Tiles(tile_layer) = self.view.selected_layer {
                    for x in 0..map.data.xsize() {
                        for y in 0..map.data.ysize() {
                            let position = (x, y, tile_layer);
                            let new_tile_id = map.data[(x, y, tile_layer)];
                            if new_tile_id != self.layer_cache[x + y * map.data.xsize()] {
                                self.view.map.set_tile(new_tile_id, position);
                            }
                        }
                    }
                }
            })
        });

        self.event_windows.update(ui.ctx());
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
