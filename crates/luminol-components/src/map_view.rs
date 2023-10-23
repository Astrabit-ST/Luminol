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

use itertools::Itertools;

#[derive(Debug)]
pub struct MapView {
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    pub pan: egui::Vec2,
    pub inter_tile_pan: egui::Vec2,

    /// The first sprite is for drawing on the tilemap,
    /// and the second sprite is for the hover preview.
    pub events: luminol_data::OptionVec<(luminol_graphics::Event, luminol_graphics::Event)>,
    pub map: luminol_graphics::Map,

    pub selected_layer: SelectedLayer,
    pub selected_event_id: Option<usize>,
    pub cursor_pos: egui::Pos2,
    pub event_enabled: bool,
    pub snap_to_grid: bool,

    /// The map coordinates of the tile being hovered over
    pub hover_tile: Option<egui::Pos2>,

    /// True if selected_event_id is being hovered over by the mouse
    /// (as opposed to the map cursor)
    /// and false otherwise
    pub selected_event_is_hovered: bool,

    pub darken_unselected_layers: bool,

    pub scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorState {
    Nothing,
    DraggingEvent(egui::Vec2),
    DrawingShape(egui::Pos2),
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum SelectedLayer {
    #[default]
    Events,
    Tiles(usize),
}

impl MapView {
    pub fn new(
        update_state: &luminol_core::UpdateState<'_>,
        map_id: usize,
    ) -> anyhow::Result<MapView> {
        let map = update_state
            .data
            .get_or_load_map(map_id, update_state.filesystem);
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets[map.tileset_id];

        let atlas = update_state.graphics.atlas_cache.load_atlas(
            &update_state.graphics,
            update_state.filesystem,
            tileset,
        )?;
        let events = map
            .events
            .iter()
            .map(|(id, e)| {
                let sprite = luminol_graphics::Event::new(
                    &update_state.graphics,
                    update_state.filesystem,
                    e,
                    &atlas,
                    update_state.graphics.push_constants_supported(),
                );
                let preview_sprite = luminol_graphics::Event::new(
                    &update_state.graphics,
                    update_state.filesystem,
                    e,
                    &atlas,
                    update_state.graphics.push_constants_supported(),
                );
                let Ok(sprite) = sprite else {
                    return Err(sprite.unwrap_err());
                };
                let Ok(preview_sprite) = preview_sprite else {
                    return Err(preview_sprite.unwrap_err());
                };
                Ok(if let Some(sprite) = sprite {
                    preview_sprite.map(|preview_sprite| (id, (sprite, preview_sprite)))
                } else {
                    None
                })
            })
            .flatten_ok()
            .try_collect()?;
        let map = luminol_graphics::Map::new(
            &update_state.graphics,
            update_state.filesystem,
            &map,
            tileset,
            update_state.graphics.push_constants_supported(),
        )?;

        Ok(Self {
            visible_display: false,
            move_preview: false,

            pan: egui::Vec2::ZERO,
            inter_tile_pan: egui::Vec2::ZERO,

            events,
            map,

            selected_layer: SelectedLayer::default(),
            selected_event_id: None,
            cursor_pos: egui::Pos2::ZERO,
            event_enabled: true,
            snap_to_grid: false,

            darken_unselected_layers: true,

            hover_tile: None,

            selected_event_is_hovered: false,

            scale: 100.,
        })
    }

    // FIXME
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        graphics_state: &std::sync::Arc<luminol_graphics::GraphicsState>,
        map: &luminol_data::rpg::Map,
        tilepicker: &crate::Tilepicker,
        cursor_state: CursorState,
        force_show_pattern_rect: bool,
    ) -> egui::Response {
        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = self.scale;
            let delta = ui.input(|i| i.scroll_delta.y * 5.);

            // Apply scroll and cap max zoom to 15%
            self.scale += delta / 30.;
            self.scale = self.scale.max(15.).min(300.);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * self.scale + self.inter_tile_pan;
        }

        let ctrl_drag = ui.input(|i| {
            // Handle pan
            if i.key_pressed(egui::Key::ArrowUp) && self.cursor_pos.y > 0. {
                self.cursor_pos.y -= 1.0;
            }
            if i.key_pressed(egui::Key::ArrowDown)
                && self.cursor_pos.y < map.data.ysize() as f32 - 1.
            {
                self.cursor_pos.y += 1.0;
            }
            if i.key_pressed(egui::Key::ArrowLeft) && self.cursor_pos.x > 0. {
                self.cursor_pos.x -= 1.0;
            }
            if i.key_pressed(egui::Key::ArrowRight)
                && self.cursor_pos.x < map.data.xsize() as f32 - 1.
            {
                self.cursor_pos.x += 1.0;
            }

            i.modifiers.command && response.dragged_by(egui::PointerButton::Primary)
        });

        let panning_map_view = response.dragged_by(egui::PointerButton::Middle) || ctrl_drag;

        if panning_map_view {
            self.pan += response.drag_delta();
            ui.ctx().request_repaint();
        }

        // Handle cursor icon
        if panning_map_view {
            response = response.on_hover_cursor(egui::CursorIcon::Grabbing);
        } else {
            response = response.on_hover_cursor(egui::CursorIcon::Grab);
        }

        // Determine some values which are relatively constant
        // If we don't use pixels_per_point then the map is the wrong size.
        // *don't ask me how i know this*.
        // its a *long* story
        let scale = self.scale / (ui.ctx().pixels_per_point() * 100.);
        let tile_size = 32. * scale;

        if self.snap_to_grid {
            self.inter_tile_pan = egui::vec2(self.pan.x % tile_size, self.pan.y % tile_size);
            self.pan -= self.inter_tile_pan;
        }

        let canvas_pos = canvas_center + self.pan;

        // We check here after we calculate the scale and whatnot
        self.hover_tile = None;
        if let Some(pos) = response.hover_pos() {
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0., map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0., map.height as f32 - 1.);
            self.hover_tile = Some(pos_tile.to_pos2());
            // Handle input
            if matches!(self.selected_layer, SelectedLayer::Tiles(_))
                || matches!(cursor_state, CursorState::DraggingEvent(_))
                || response.clicked()
            {
                self.cursor_pos = pos_tile.to_pos2();
            }
        }

        let width2 = map.width as f32 / 2.;
        let height2 = map.height as f32 / 2.;

        let pos = egui::Vec2::new(width2 * tile_size, height2 * tile_size);
        let map_rect = egui::Rect {
            min: canvas_pos - pos,
            max: canvas_pos + pos,
        };

        let proj_center_x = width2 * 32. - self.pan.x / scale;
        let proj_center_y = height2 * 32. - self.pan.y / scale;
        let proj_width2 = canvas_rect.width() / scale / 2.;
        let proj_height2 = canvas_rect.height() / scale / 2.;

        let graphics_state = graphics_state.clone();

        self.map.set_proj(
            &graphics_state.render_state,
            glam::Mat4::orthographic_rh(
                proj_center_x - proj_width2,
                proj_center_x + proj_width2,
                proj_center_y + proj_height2,
                proj_center_y - proj_height2,
                -1.,
                1.,
            ),
        );
        self.map.paint(
            graphics_state.clone(),
            ui.painter(),
            match self.selected_layer {
                SelectedLayer::Events => None,
                SelectedLayer::Tiles(selected_layer) if self.darken_unselected_layers => {
                    Some(selected_layer)
                }
                SelectedLayer::Tiles(_) => None,
            },
            canvas_rect,
        );

        ui.painter().rect_stroke(
            map_rect,
            5.,
            egui::Stroke::new(3., egui::Color32::DARK_GRAY),
        );

        let cursor_rect = egui::Rect::from_min_size(
            map_rect.min + (self.cursor_pos.to_vec2() * tile_size),
            egui::Vec2::splat(tile_size),
        );
        let pattern_rect = egui::Rect::from_min_size(
            map_rect.min + (self.cursor_pos.to_vec2() * tile_size),
            if !force_show_pattern_rect && matches!(cursor_state, CursorState::DrawingShape(_)) {
                egui::Vec2::splat(tile_size)
            } else {
                egui::vec2(
                    tile_size
                        * (tilepicker.selected_tiles_right - tilepicker.selected_tiles_left + 1)
                            as f32,
                    tile_size
                        * (tilepicker.selected_tiles_bottom - tilepicker.selected_tiles_top + 1)
                            as f32,
                )
            },
        )
        .intersect(map_rect);

        if !self.event_enabled || !matches!(self.selected_layer, SelectedLayer::Events) {
            self.selected_event_id = None;
        }
        self.selected_event_is_hovered = false;

        if self.event_enabled {
            let mut selected_event = None;
            let mut selected_event_rects = None;

            for (_, event) in map.events.iter() {
                let sprites = self.events.get(event.id);
                let event_size = sprites
                    .map(|e| e.0.sprite_size)
                    .unwrap_or(egui::vec2(32., 32.));
                let scaled_event_size = event_size * scale;

                // Darken the graphic if required
                if let Some((sprite, _)) = sprites {
                    sprite.sprite().graphic.set_opacity_multiplier(
                        &graphics_state.render_state,
                        if self.darken_unselected_layers
                            && !matches!(self.selected_layer, SelectedLayer::Events)
                        {
                            0.5
                        } else {
                            1.
                        },
                    );
                }

                let box_rect = egui::Rect::from_min_size(
                    map_rect.min
                        + egui::vec2(
                            (event.x as f32 * tile_size) + (tile_size - scaled_event_size.x) / 2.,
                            (event.y as f32 * tile_size) + (tile_size - scaled_event_size.y),
                        ),
                    scaled_event_size,
                );

                if let Some((sprite, _)) = sprites {
                    let x = event.x as f32 * 32. + (32. - event_size.x) / 2.;
                    let y = event.y as f32 * 32. + (32. - event_size.y);
                    sprite.set_proj(
                        &graphics_state.render_state,
                        glam::Mat4::orthographic_rh(
                            proj_center_x - proj_width2 - x,
                            proj_center_x + proj_width2 - x,
                            proj_center_y + proj_height2 - y,
                            proj_center_y - proj_height2 - y,
                            -1.,
                            1.,
                        ),
                    );
                    sprite.paint(graphics_state.clone(), ui.painter(), canvas_rect);
                }

                if matches!(self.selected_layer, SelectedLayer::Events)
                    && ui.input(|i| !i.modifiers.shift)
                {
                    ui.painter().rect_stroke(
                        box_rect,
                        5.,
                        egui::Stroke::new(1., egui::Color32::WHITE),
                    );

                    // If the mouse is not hovering over an event, then we will handle the selected
                    // tile based on where the map cursor is
                    if !self.selected_event_is_hovered
                        && !matches!(cursor_state, CursorState::DraggingEvent(_))
                    {
                        selected_event = match selected_event {
                            // If the map cursor is on the exact tile of an event, then that is the
                            // selected event
                            Some(_)
                                if self.cursor_pos.x == event.x as f32
                                    && self.cursor_pos.y == event.y as f32 =>
                            {
                                Some(event)
                            }
                            Some(e)
                                if self.cursor_pos.x == e.x as f32
                                    && self.cursor_pos.y == e.y as f32 =>
                            {
                                selected_event
                            }
                            // Otherwise if the map cursor intersects at least one event's graphic,
                            // then the one out of those with the highest ID should be the selected
                            // event
                            _ if box_rect.contains(cursor_rect.center()) => Some(event),
                            _ => selected_event,
                        };
                        if let Some(e) = selected_event {
                            if e.id == event.id {
                                selected_event_rects = Some(box_rect);
                            }
                        }
                    }

                    if ui.rect_contains_pointer(box_rect) {
                        response = response.on_hover_ui_at_pointer(|ui| {
                            ui.label(format!("Event {:0>3}: {:?}", event.id, event.name));

                            let (response, painter) = ui.allocate_painter(
                                event_size * ui.ctx().pixels_per_point(),
                                egui::Sense::click(),
                            );
                            if let Some((_, preview_sprite)) = sprites {
                                if ui.ctx().screen_rect().contains_rect(response.rect) {
                                    preview_sprite.paint(
                                        graphics_state.clone(),
                                        &painter,
                                        response.rect,
                                    );
                                }
                            }
                            match self.selected_event_id {
                                Some(id) if id == event.id => ui.painter().rect_stroke(
                                    response.rect,
                                    5.,
                                    egui::Stroke::new(2., egui::Color32::YELLOW),
                                ),
                                _ => ui.painter().rect_stroke(
                                    response.rect,
                                    5.,
                                    egui::Stroke::new(1., egui::Color32::WHITE),
                                ),
                            }
                        });

                        if let Some(hover_tile) = self.hover_tile {
                            if !matches!(cursor_state, CursorState::DraggingEvent(_)) {
                                // Handle which event should be considered selected based on the
                                // hovered tile
                                selected_event = match selected_event {
                                    // If the cursor is hovering over the exact tile of an event, then that is
                                    // the selected event
                                    Some(_)
                                        if hover_tile.x == event.x as f32
                                            && hover_tile.y == event.y as f32 =>
                                    {
                                        Some(event)
                                    }
                                    Some(e)
                                        if hover_tile.x == e.x as f32
                                            && hover_tile.y == e.y as f32 =>
                                    {
                                        selected_event
                                    }
                                    // Otherwise if the cursor is hovering over at least one event's graphic,
                                    // then the one out of those with the highest ID should be the selected event
                                    _ => Some(event),
                                };
                                if let Some(e) = selected_event {
                                    if e.id == event.id {
                                        self.selected_event_is_hovered = true;
                                        selected_event_rects = Some(box_rect);
                                    }
                                }
                            }
                        }
                    }

                    // If an event is being dragged, that should always be the selected event
                    if let Some(id) = self.selected_event_id {
                        if matches!(cursor_state, CursorState::DraggingEvent(_)) && id == event.id {
                            selected_event = Some(event);
                            selected_event_rects = Some(box_rect);
                        }
                    }
                } else {
                    ui.painter().rect_stroke(
                        box_rect,
                        5.,
                        egui::Stroke::new(1., egui::Color32::DARK_GRAY),
                    );
                }

                // Draw a magenta rectangle on the border of events that are being edited
                if event.extra_data.is_editor_open {
                    ui.painter().rect_stroke(
                        box_rect,
                        5.,
                        egui::Stroke::new(3., egui::Color32::from_rgb(255, 0, 255)),
                    );
                }
            }

            self.selected_event_id = selected_event.map(|e| e.id);

            // Draw a yellow rectangle on the border of the selected event's graphic
            if let Some(selected_event) = selected_event {
                // Make sure the event editor isn't open so we don't draw over the
                // magenta rectangle
                if !selected_event.extra_data.is_editor_open {
                    if let Some(box_rect) = selected_event_rects {
                        ui.painter().rect_stroke(
                            box_rect,
                            5.,
                            egui::Stroke::new(3., egui::Color32::YELLOW),
                        );
                    }
                }
            }
        }

        // Do we display the visible region?
        if self.visible_display {
            // Determine the visible region.
            let width2: f32 = (640. / 2.) * scale;
            let height2: f32 = (480. / 2.) * scale;

            let pos = egui::Vec2::new(width2, height2);
            let visible_rect = egui::Rect {
                min: canvas_center - pos,
                max: canvas_center + pos,
            };

            // Show the region.
            ui.painter().rect_stroke(
                visible_rect,
                5.,
                egui::Stroke::new(1., egui::Color32::YELLOW),
            );
        }

        // Draw the origin tile for the rectangle and circle brushes
        if let CursorState::DrawingShape(drawing_shape_pos) = cursor_state {
            let drawing_shape_rect = egui::Rect::from_min_size(
                map_rect.min + (drawing_shape_pos.to_vec2() * tile_size),
                egui::Vec2::splat(tile_size),
            );
            ui.painter().rect_stroke(
                drawing_shape_rect,
                5.,
                egui::Stroke::new(1., egui::Color32::WHITE),
            );
        }

        // Display cursor.
        if matches!(self.selected_layer, SelectedLayer::Tiles(_)) {
            ui.painter().rect_stroke(
                pattern_rect,
                5.,
                egui::Stroke::new(1., egui::Color32::WHITE),
            );
        }
        ui.painter().rect_stroke(
            cursor_rect,
            5.,
            egui::Stroke::new(1., egui::Color32::YELLOW),
        );

        response
    }
}
