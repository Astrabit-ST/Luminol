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
pub use crate::prelude::*;

#[derive(Debug)]
pub struct MapView {
    /// Toggle to display the visible region in-game.
    pub visible_display: bool,
    /// Toggle move route preview
    pub move_preview: bool,

    pub pan: egui::Vec2,
    pub inter_tile_pan: egui::Vec2,

    pub events: slab::Slab<Event>,
    pub map: Map,

    pub selected_layer: SelectedLayer,
    pub selected_event_id: Option<usize>,
    pub cursor_pos: egui::Pos2,
    pub event_enabled: bool,
    pub snap_to_grid: bool,

    pub darken_unselected_layers: bool,

    pub scale: f32,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Default)]
pub enum SelectedLayer {
    #[default]
    Events,
    Tiles(usize),
}

impl MapView {
    pub fn new(map: &rpg::Map, tileset: &rpg::Tileset) -> Result<MapView, String> {
        // Get tilesets.

        let atlas = state!().atlas_cache.load_atlas(tileset)?;
        let events = map
            .events
            .iter()
            .map(|(id, e)| Event::new(e, &atlas).map(|o| o.map(|e| (id, e))))
            .flatten_ok()
            .try_collect()?;
        let map = Map::new(map, tileset)?;

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

            scale: 100.,
        })
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        map: &rpg::Map,
        dragging_event: bool,
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

        // Handle pan
        let panning_map_view = response.dragged_by(egui::PointerButton::Middle)
            || (ui.input(|i| {
                i.modifiers.command && response.dragged_by(egui::PointerButton::Primary)
            }));
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
        let mut cursor_tile = None;
        if let Some(pos) = response.hover_pos() {
            let mut pos_tile = (pos - self.pan - canvas_center) / tile_size
                + egui::Vec2::new(map.width as f32 / 2., map.height as f32 / 2.);
            // Force the cursor to a tile instead of in-between
            pos_tile.x = pos_tile.x.floor().clamp(0., map.width as f32 - 1.);
            pos_tile.y = pos_tile.y.floor().clamp(0., map.height as f32 - 1.);
            cursor_tile = Some(pos_tile);
            // Handle input
            if matches!(self.selected_layer, SelectedLayer::Tiles(_))
                || dragging_event
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

        self.map.paint(
            ui.painter(),
            match self.selected_layer {
                SelectedLayer::Events => None,
                SelectedLayer::Tiles(selected_layer) if self.darken_unselected_layers => {
                    Some(selected_layer)
                }
                SelectedLayer::Tiles(_) => None,
            },
            map_rect,
        );

        ui.painter().rect_stroke(
            map_rect,
            5.,
            egui::Stroke::new(3., egui::Color32::DARK_GRAY),
        );

        if self.event_enabled {
            let mut selected_event = None;
            let mut selected_event_rects = None;

            for (_, event) in map.events.iter() {
                let sprite = self.events.get(event.id);
                let event_size = sprite
                    .map(|e| e.sprite_size)
                    .unwrap_or(egui::vec2(32., 32.));
                let scaled_event_size = event_size * scale;

                let tile_rect = egui::Rect::from_min_size(
                    map_rect.min
                        + egui::vec2(event.x as f32 * tile_size, event.y as f32 * tile_size),
                    egui::vec2(32., 32.) * scale,
                );
                let box_rect = egui::Rect::from_min_size(
                    map_rect.min
                        + egui::vec2(
                            (event.x as f32 * tile_size) + (tile_size - scaled_event_size.x) / 2.,
                            (event.y as f32 * tile_size) + (tile_size - scaled_event_size.y),
                        ),
                    scaled_event_size,
                );

                if let Some(sprite) = sprite {
                    sprite.paint(ui.painter(), box_rect);
                }

                ui.painter()
                    .rect_stroke(box_rect, 5., egui::Stroke::new(1., egui::Color32::WHITE));

                if ui.rect_contains_pointer(box_rect) {
                    response = response.on_hover_ui_at_pointer(|ui| {
                        ui.label(format!("Event {:0>3}: {:?}", event.id, event.name));

                        let (response, painter) = ui.allocate_painter(
                            event_size * ui.ctx().pixels_per_point(),
                            egui::Sense::click(),
                        );
                        if let Some(sprite) = sprite {
                            sprite.paint(&painter, response.rect);
                        }
                        ui.painter().rect_stroke(
                            response.rect,
                            5.,
                            egui::Stroke::new(1., egui::Color32::WHITE),
                        );
                    });

                    // Safe because rect_contains_pointer won't run unless cursor position is
                    // detected successfully
                    let cursor_tile = cursor_tile.unwrap();

                    // Handle which event should be considered selected
                    selected_event = match selected_event {
                        // If the cursor is hovering over the exact tile of an event, then that is
                        // the selected event
                        Some(e)
                            if cursor_tile.x == event.x as f32
                                && cursor_tile.y == event.y as f32 =>
                        {
                            Some(event)
                        }
                        Some(e) if cursor_tile.x == e.x as f32 && cursor_tile.y == e.y as f32 => {
                            selected_event
                        }
                        // Otherwise if the cursor is hovering over at least one event's graphic,
                        // then the one out of those with the highest ID should be the selected event
                        _ => Some(event),
                    };
                    if let Some(e) = selected_event {
                        if e.id == event.id {
                            selected_event_rects = Some((tile_rect, box_rect));
                        }
                    }
                }
            }

            self.selected_event_id = selected_event.map(|e| e.id);

            // Draw a magenta rectangle on the border of the selected event's graphic
            // and a green rectangle on the border of the selected event's tile
            if let Some((tile_rect, box_rect)) = selected_event_rects {
                ui.painter().rect_stroke(
                    tile_rect,
                    12.,
                    egui::Stroke::new(2., egui::Color32::GREEN),
                );
                ui.painter().rect_stroke(
                    box_rect,
                    5.,
                    egui::Stroke::new(2., egui::Color32::from_rgb(255, 0, 255)),
                );
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

        // Display cursor.
        let cursor_rect = egui::Rect::from_min_size(
            map_rect.min + (self.cursor_pos.to_vec2() * tile_size),
            egui::Vec2::splat(tile_size),
        );
        ui.painter().rect_stroke(
            cursor_rect,
            5.,
            egui::Stroke::new(1., egui::Color32::YELLOW),
        );

        response
    }
}
