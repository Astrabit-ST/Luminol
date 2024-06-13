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

use color_eyre::eyre::{ContextCompat, WrapErr};
use itertools::Itertools;
use std::io::Write;

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

    /// Whether to display the tile IDs on the map
    pub display_tile_ids: bool,

    pub scale: f32,
    pub previous_scale: f32,

    /// Used to store the bounding boxes of event graphics in order to render them on top of the
    /// fog and collision layers
    pub event_rects: Vec<egui::Rect>,

    pub data_id: egui::Id,
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
    ) -> color_eyre::Result<MapView> {
        let map = update_state
            .data
            .get_or_load_map(map_id, update_state.filesystem);
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets.data[map.tileset_id];

        let mut passages = luminol_data::Table2::new(map.data.xsize(), map.data.ysize());
        luminol_graphics::collision::calculate_passages(
            &tileset.passages,
            &tileset.priorities,
            &map.data,
            Some(&map.events),
            (0..map.data.zsize()).rev(),
            |x, y, passage| passages[(x, y)] = passage,
        );

        let atlas = update_state.graphics.atlas_loader.load_atlas(
            &update_state.graphics,
            update_state.filesystem,
            tileset,
        )?;
        let events = map
            .events
            .iter()
            .map(|(id, e)| -> color_eyre::Result<_> {
                let sprite = luminol_graphics::Event::new(
                    &update_state.graphics,
                    update_state.filesystem,
                    e,
                    &atlas,
                )?;
                let preview_sprite = luminol_graphics::Event::new(
                    &update_state.graphics,
                    update_state.filesystem,
                    e,
                    &atlas,
                )?;

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
            &passages,
        )?;

        let data_id = egui::Id::new("luminol_map_view")
            .with(
                update_state
                    .project_config
                    .as_ref()
                    .expect("project not loaded")
                    .project
                    .persistence_id,
            )
            .with(map_id);
        let (cursor_pos, pan, inter_tile_pan, scale) = update_state.ctx.data_mut(|d| {
            *d.get_persisted_mut_or_insert_with(data_id, || {
                (egui::Pos2::ZERO, egui::Vec2::ZERO, egui::Vec2::ZERO, 100.)
            })
        });

        Ok(Self {
            visible_display: false,
            move_preview: false,

            pan,
            inter_tile_pan,

            events,
            map,

            selected_layer: SelectedLayer::default(),
            selected_event_id: None,
            cursor_pos,
            event_enabled: true,
            snap_to_grid: false,

            darken_unselected_layers: true,

            hover_tile: None,

            selected_event_is_hovered: false,

            display_tile_ids: false,

            scale,
            previous_scale: scale,

            event_rects: Vec::new(),

            data_id,
        })
    }

    // FIXME lots of arguments
    #[allow(clippy::too_many_arguments)]
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        graphics_state: &std::sync::Arc<luminol_graphics::GraphicsState>,
        map: &luminol_data::rpg::Map,
        tilepicker: &crate::Tilepicker,
        dragging_event: bool,
        drawing_shape: bool,
        drawing_shape_pos: Option<egui::Pos2>,
        force_show_pattern_rect: bool,
        is_focused: bool,
    ) -> egui::Response {
        // Allocate the largest size we can for the tilemap
        let canvas_rect = ui.max_rect();
        let canvas_center = canvas_rect.center();
        ui.set_clip_rect(canvas_rect);

        let mut response = ui.allocate_rect(canvas_rect, egui::Sense::click_and_drag());

        let min_clip = (ui.ctx().screen_rect().min - canvas_rect.min).max(Default::default());
        let max_clip = (canvas_rect.max - ui.ctx().screen_rect().max).max(Default::default());
        let clip_offset = (max_clip - min_clip) / 2.;
        let canvas_rect = ui.ctx().screen_rect().intersect(canvas_rect);

        self.cursor_pos = self.cursor_pos.clamp(
            egui::Pos2::ZERO,
            egui::pos2(
                map.data.xsize().saturating_sub(1) as f32,
                map.data.ysize().saturating_sub(1) as f32,
            ),
        );

        // If the user changed the scale using the scale slider, pan the map so that the scale uses
        // the center of the visible part of the map as the scale center
        if self.scale != self.previous_scale {
            self.pan = self.inter_tile_pan + self.pan * self.scale / self.previous_scale;
        }

        // Handle zoom
        if let Some(pos) = response.hover_pos() {
            // We need to store the old scale before applying any transformations
            let old_scale = self.scale;
            let delta = ui.input(|i| i.smooth_scroll_delta.y);

            // Apply scroll and cap max zoom to 15%
            self.scale *= (delta / 9.0f32.exp2()).exp2();
            self.scale = self.scale.max(15.).min(300.);

            // Get the normalized cursor position relative to pan
            let pos_norm = (pos - self.pan - canvas_center) / old_scale;
            // Offset the pan to the cursor remains in the same place
            // Still not sure how the math works out, if it ain't broke don't fix it
            self.pan = pos - canvas_center - pos_norm * self.scale + self.inter_tile_pan;
        }

        self.previous_scale = self.scale;

        let grid_inner_thickness = if self.scale >= 50. { 1. } else { 0. };

        let ctrl_drag = ui.input(|i| {
            if is_focused {
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
            }

            i.modifiers.command
        }) && response.dragged_by(egui::PointerButton::Primary);

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
                || ((dragging_event || response.clicked()) && ui.input(|i| !i.modifiers.command))
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

        let proj_center_x = width2 * 32. - (self.pan.x + clip_offset.x) / scale;
        let proj_center_y = height2 * 32. - (self.pan.y + clip_offset.y) / scale;
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
            if tilepicker.brush_random || (!force_show_pattern_rect && drawing_shape_pos.is_some())
            {
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
            let mut selected_event_rect = None;

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
                    if canvas_rect.intersects(box_rect) {
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
                }

                if matches!(self.selected_layer, SelectedLayer::Events)
                    && ui.input(|i| !i.modifiers.shift)
                {
                    self.event_rects.push(box_rect);

                    // If the mouse is not hovering over an event, then we will handle the selected
                    // tile based on where the map cursor is
                    if !self.selected_event_is_hovered && !dragging_event {
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
                                selected_event_rect = Some(box_rect);
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
                                if response.rect.is_positive() {
                                    let clipped_rect =
                                        ui.ctx().screen_rect().intersect(response.rect);
                                    let proj_rect = egui::Rect::from_min_size(
                                        (ui.ctx().screen_rect().min - response.rect.min)
                                            .max(Default::default())
                                            .to_pos2(),
                                        preview_sprite.sprite_size * clipped_rect.size()
                                            / response.rect.size(),
                                    );
                                    preview_sprite.set_proj(
                                        &graphics_state.render_state,
                                        glam::Mat4::orthographic_rh(
                                            proj_rect.left(),
                                            proj_rect.right(),
                                            proj_rect.bottom(),
                                            proj_rect.top(),
                                            -1.,
                                            1.,
                                        ),
                                    );
                                    preview_sprite.paint(
                                        graphics_state.clone(),
                                        &painter,
                                        clipped_rect,
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
                            };
                        });

                        if let Some(hover_tile) = self.hover_tile {
                            if !dragging_event {
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
                                        selected_event_rect = Some(box_rect);
                                    }
                                }
                            }
                        }
                    }

                    // If an event is being dragged, that should always be the selected event
                    if let Some(id) = self.selected_event_id {
                        if dragging_event && id == event.id {
                            selected_event = Some(event);
                            selected_event_rect = Some(box_rect);
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

            // Draw the fog and collision layers
            self.map.paint_overlay(
                graphics_state.clone(),
                ui.painter(),
                grid_inner_thickness,
                canvas_rect,
            );

            // Draw white rectangles on the border of all events
            while let Some(rect) = self.event_rects.pop() {
                ui.painter()
                    .rect_stroke(rect, 5., egui::Stroke::new(1., egui::Color32::WHITE));
            }

            // Draw a yellow rectangle on the border of the selected event's graphic
            if let Some(selected_event) = selected_event {
                // Make sure the event editor isn't open so we don't draw over the
                // magenta rectangle
                if !selected_event.extra_data.is_editor_open {
                    if let Some(rect) = selected_event_rect {
                        ui.painter().rect_stroke(
                            rect,
                            5.,
                            egui::Stroke::new(3., egui::Color32::YELLOW),
                        );
                    }
                }
            }
        } else {
            // Draw the fog and collision layers
            self.map.paint_overlay(
                graphics_state.clone(),
                ui.painter(),
                grid_inner_thickness,
                canvas_rect,
            );
        }

        // FIXME: If we want to be fast, we should be rendering all the tile ids to a texture once and then just rendering that texture here
        if self.display_tile_ids {
            if let SelectedLayer::Tiles(layer) = self.selected_layer {
                for (i, id) in map.data.layer_as_slice(layer).iter().copied().enumerate() {
                    let x = i % map.data.xsize();
                    let y = i / map.data.xsize();

                    let tile_x = x as f32 * tile_size;
                    let tile_y = y as f32 * tile_size;
                    let tile_pos = egui::Pos2::new(tile_x, tile_y)
                        + egui::Vec2::splat(tile_size / 2.0)
                        + map_rect.min.to_vec2();
                    ui.painter().text(
                        tile_pos,
                        egui::Align2::CENTER_CENTER,
                        id.to_string(),
                        egui::FontId::monospace(12. * scale),
                        egui::Color32::WHITE,
                    );
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
        if drawing_shape {
            if let Some(drawing_shape_pos) = drawing_shape_pos {
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

        ui.ctx().data_mut(|d| {
            d.insert_persisted(
                self.data_id,
                (self.cursor_pos, self.pan, self.inter_tile_pan, self.scale),
            );
        });

        response
    }

    /// Saves the current state of the map to an image file of the user's choice (will prompt the
    /// user with a file picker).
    /// This function returns a future that you need to `.await` to finish saving the image, but
    /// the future doesn't borrow anything so you don't need to worry about lifetime-related issues.
    pub fn save_as_image(
        &self,
        graphics_state: &std::sync::Arc<luminol_graphics::GraphicsState>,
        map: &luminol_data::rpg::Map,
    ) -> impl std::future::Future<Output = color_eyre::Result<()>> {
        let c = "While screenshotting the map";

        let width_unpadded = (map.width * 32) as u32;
        let height = (map.height * 32) as u32;
        let width = width_unpadded.next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / 4);
        let viewport_rect =
            egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(width as f32, height as f32));

        let texture = graphics_state
            .render_state
            .device
            .create_texture(&wgpu::TextureDescriptor {
                label: Some("map editor screenshot texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: graphics_state.render_state.target_format,
                usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let buffer = graphics_state
            .render_state
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("map editor screenshot buffer"),
                size: width as u64 * height as u64 * 4,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

        self.map.set_proj(
            &graphics_state.render_state,
            glam::Mat4::orthographic_rh(0., width as f32, height as f32, 0., -1., 1.),
        );

        let mut command_encoder = graphics_state
            .render_state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let map_callback = self.map.callback(
                graphics_state.clone(),
                match self.selected_layer {
                    SelectedLayer::Events => None,
                    SelectedLayer::Tiles(selected_layer) if self.darken_unselected_layers => {
                        Some(selected_layer)
                    }
                    SelectedLayer::Tiles(_) => None,
                },
            );
            let map_overlay_callback = self.map.overlay_callback(graphics_state.clone(), 1.);

            let event_callbacks = map
                .events
                .iter()
                .filter_map(|(_, event)| {
                    let sprites = self.events.get(event.id);
                    let tile_size = 32.;
                    let event_size = sprites
                        .map(|e| e.0.sprite_size)
                        .unwrap_or(egui::vec2(32., 32.));

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

                    let rect = egui::Rect::from_min_size(
                        egui::pos2(
                            (event.x as f32 * tile_size) + (tile_size - event_size.x) / 2.,
                            (event.y as f32 * tile_size) + (tile_size - event_size.y),
                        ),
                        event_size,
                    );

                    sprites.and_then(|(sprite, _)| {
                        viewport_rect.intersects(rect).then(|| {
                            let x = event.x as f32 * 32. + (32. - event_size.x) / 2.;
                            let y = event.y as f32 * 32. + (32. - event_size.y);
                            sprite.set_proj(
                                &graphics_state.render_state,
                                glam::Mat4::orthographic_rh(
                                    -x,
                                    width as f32 - x,
                                    height as f32 - y,
                                    -y,
                                    -1.,
                                    1.,
                                ),
                            );
                            sprite.callback(graphics_state.clone())
                        })
                    })
                })
                .collect_vec();

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("map editor screenshot render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations::default(),
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            map_callback.paint(&mut render_pass);
            for event_callback in event_callbacks.iter() {
                event_callback.paint(&mut render_pass);
            }
            map_overlay_callback.paint(
                egui::PaintCallbackInfo {
                    viewport: viewport_rect,
                    clip_rect: viewport_rect,
                    pixels_per_point: 1.,
                    screen_size_px: [width, height],
                },
                &mut render_pass,
            );
        }

        command_encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        graphics_state
            .render_state
            .queue
            .submit(Some(command_encoder.finish()));

        let graphics_state = graphics_state.clone();
        let (tx, rx) = oneshot::channel();
        async move {
            buffer
                .slice(..)
                .map_async(wgpu::MapMode::Read, move |result| {
                    let _ = tx.send(result);
                });
            if !graphics_state
                .render_state
                .device
                .poll(wgpu::Maintain::Wait)
                .is_queue_empty()
            {
                return Err(color_eyre::eyre::eyre!("wgpu::Device::poll timed out").wrap_err(c));
            }
            rx.await.unwrap().wrap_err(c)?;

            let mut vec = buffer
                .slice(..)
                .get_mapped_range()
                .chunks_exact(width as usize * 4)
                .flat_map(|row| row[..width_unpadded as usize * 4].iter())
                .copied()
                .collect_vec();
            if graphics_state.render_state.target_format == wgpu::TextureFormat::Bgra8Unorm {
                for (b, _g, r, _a) in vec.iter_mut().tuples() {
                    std::mem::swap(b, r);
                }
            }

            let screenshot = image::RgbaImage::from_raw(width_unpadded, height, vec).wrap_err(c)?;
            let mut file = luminol_filesystem::host::File::new().wrap_err(c)?;
            screenshot
                .write_to(
                    &mut std::io::BufWriter::new(&mut file),
                    image::ImageOutputFormat::Png,
                )
                .wrap_err(c)?;
            file.flush().wrap_err(c)?;
            file.save("map.png", "Portable Network Graphics")
                .await
                .wrap_err(c)
        }
    }
}
