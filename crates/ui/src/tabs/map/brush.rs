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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use itertools::Itertools;

impl super::Tab {
    pub(super) fn handle_brush(
        &mut self,
        map_x: usize,
        map_y: usize,
        tile_layer: usize,
        pencil: luminol_core::Pencil,
        map: &mut luminol_data::rpg::Map,
    ) {
        let map_pos = egui::pos2(map_x as f32, map_y as f32);
        let initial_tile =
            luminol_components::SelectedTile::from_id(map.data[(map_x, map_y, tile_layer)]);
        let left = self.tilepicker.selected_tiles_left;
        let right = self.tilepicker.selected_tiles_right;
        let top = self.tilepicker.selected_tiles_top;
        let bottom = self.tilepicker.selected_tiles_bottom;
        let width = right - left + 1;
        let height = bottom - top + 1;

        match pencil {
            luminol_core::Pencil::Pen => {
                let (rect_width, rect_height) = if self.tilepicker.brush_random {
                    (1, 1)
                } else {
                    (width, height)
                };

                let drawing_shape_pos = if let Some(drawing_shape_pos) = self.drawing_shape_pos {
                    drawing_shape_pos
                } else {
                    self.drawing_shape_pos = Some(map_pos);
                    map_pos
                };
                for (y, x) in (0..rect_height).cartesian_product(0..rect_width) {
                    let absolute_x = map_x + x as usize;
                    let absolute_y = map_y + y as usize;

                    // Skip out-of-bounds tiles
                    if absolute_x >= map.data.xsize() || absolute_y >= map.data.ysize() {
                        continue;
                    }

                    self.set_tile(
                        map,
                        self.tilepicker.get_tile_from_offset(
                            absolute_x as i16,
                            absolute_y as i16,
                            tile_layer as i16,
                            x + (map_x as f32 - drawing_shape_pos.x) as i16,
                            y + (map_y as f32 - drawing_shape_pos.y) as i16,
                        ),
                        (absolute_x, absolute_y, tile_layer),
                    );
                }
            }

            luminol_core::Pencil::Fill => {
                let drawing_shape_pos = if let Some(drawing_shape_pos) = self.drawing_shape_pos {
                    drawing_shape_pos
                } else {
                    self.drawing_shape_pos = Some(map_pos);
                    map_pos
                };

                // Use depth-first search to find all of the orthogonally
                // contiguous matching tiles
                let mut stack = vec![(map_x, map_y, tile_layer); 1];
                while let Some(position) = stack.pop() {
                    self.set_tile(
                        map,
                        self.tilepicker.get_tile_from_offset(
                            position.0 as i16,
                            position.1 as i16,
                            tile_layer as i16,
                            position.0 as i16 - drawing_shape_pos.x as i16,
                            position.1 as i16 - drawing_shape_pos.y as i16,
                        ),
                        position,
                    );
                    self.dfs_cache[position.0 + position.1 * map.data.xsize()] = true;

                    let x_array: [isize; 4] = [-1, 1, 0, 0];
                    let y_array: [isize; 4] = [0, 0, -1, 1];
                    for (x, y) in x_array.into_iter().zip(y_array.into_iter()) {
                        // Don't search tiles that are out of bounds
                        if (x == -1 && position.0 == 0)
                            || (x == 1 && position.0 + 1 == map.data.xsize())
                            || (y == -1 && position.1 == 0)
                            || (y == 1 && position.1 + 1 == map.data.ysize())
                        {
                            continue;
                        }

                        let position = (
                            position.0.saturating_add_signed(x),
                            position.1.saturating_add_signed(y),
                            position.2,
                        );

                        // Don't search tiles that we've already searched before
                        // because that would cause an infinite loop
                        if self.dfs_cache[position.0 + position.1 * map.data.xsize()] {
                            continue;
                        }

                        if luminol_components::SelectedTile::from_id(map.data[position])
                            == initial_tile
                        {
                            stack.push(position);
                        }
                    }
                }

                for x in self.dfs_cache.iter_mut() {
                    *x = false;
                }
            }

            luminol_core::Pencil::Rectangle => {
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
                    let bounding_rect = egui::Rect::from_two_pos(drawing_shape_pos, map_pos);
                    for y in (bounding_rect.min.y as usize)..=(bounding_rect.max.y as usize) {
                        for x in (bounding_rect.min.x as usize)..=(bounding_rect.max.x) as usize {
                            let position = (x, y, tile_layer);
                            self.set_tile(
                                map,
                                self.tilepicker.get_tile_from_offset(
                                    x as i16,
                                    y as i16,
                                    tile_layer as i16,
                                    x as i16 - drawing_shape_pos.x as i16,
                                    y as i16 - drawing_shape_pos.y as i16,
                                ),
                                position,
                            );
                        }
                    }
                } else {
                    self.drawing_shape_pos = Some(map_pos);
                }
            }

            luminol_core::Pencil::Circle => {
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
                    let bounding_rect = egui::Rect::from_two_pos(drawing_shape_pos, map_pos);
                    // Edge case: Bresenham's algorithm breaks down when drawing a
                    // 1x1 ellipse.
                    if drawing_shape_pos == map_pos {
                        self.set_tile(
                            map,
                            self.tilepicker.get_tile_from_offset(
                                map_x as i16,
                                map_y as i16,
                                tile_layer as i16,
                                map_x as i16 - drawing_shape_pos.x as i16,
                                map_y as i16 - drawing_shape_pos.y as i16,
                            ),
                            (map_x, map_y, tile_layer),
                        );
                    } else {
                        let bounding_rect = bounding_rect.translate(egui::vec2(0.5, 0.5));

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
                                        map,
                                        self.tilepicker.get_tile_from_offset(
                                            x as i16,
                                            y as i16,
                                            tile_layer as i16,
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
                            let f = ry2 * (x + 1.).powi(2) + rx2 * (y - 0.5).powi(2) - rx2 * ry2;
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
                                        map,
                                        self.tilepicker.get_tile_from_offset(
                                            x as i16,
                                            y as i16,
                                            tile_layer as i16,
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
                            let f = ry2 * (x - 0.5).powi(2) + rx2 * (y + 1.).powi(2) - rx2 * ry2;
                            if f > 0. {
                                x -= 1.;
                            }
                            y += 1.;
                        }
                    }
                } else {
                    self.drawing_shape_pos = Some(map_pos);
                }
            }
        };
    }
}
