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
use std::{cell::RefMut, collections::HashMap, collections::VecDeque};

const HISTORY_SIZE: usize = 50;

use crate::windows::event_edit;

use itertools::Itertools;

mod brush;
mod history;
mod util;

pub struct Tab {
    /// ID of the map that is being edited.
    pub id: usize,
    /// The tilemap.
    pub view: luminol_components::MapView,
    pub tilepicker: luminol_components::Tilepicker,

    dragging_event: bool,
    drawing_shape: bool,
    event_windows: luminol_core::Windows,
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

    /// Undo history
    history: VecDeque<HistoryEntry>,
    /// When operations are undone, they are put here so that they can be redone
    redo_history: Vec<HistoryEntry>,
    /// When starting to draw tiles, this is set to the state of the layer before
    /// any tiles are drawn in order to compute the deltas for the history
    tilemap_undo_cache: Vec<i16>,
    /// The layer tilemap_undo_cache refers to
    tilemap_undo_cache_layer: usize,

    /// This stores the passage values for every position on the map so that we can figure out
    /// which passage values have changed in the current frame
    passages: luminol_data::Table2,
}

// TODO: If we add support for changing event IDs, these need to be added as history entries
// in order to not corrupt the EventMoved and EventCreated entries.
enum HistoryEntry {
    /// Contains the (x, y, tile_id) delta for a changed map layer.
    Tiles {
        layer: usize,
        delta: Vec<(usize, usize, i16)>,
    },
    /// Contains the original map coordinates of a moved event and the ID of the event.
    EventMoved { id: usize, x: i32, y: i32 },
    /// Contains the ID of a created event.
    EventCreated(usize),
    /// Contains a deleted event and its corresponding graphic.
    EventDeleted {
        event: luminol_data::rpg::Event,
        sprites: Option<(luminol_graphics::Event, luminol_graphics::Event)>,
    },
}

impl Tab {
    /// Create a new map editor.
    pub fn new(
        id: usize,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) -> anyhow::Result<Self> {
        // *sigh*
        // borrow checker.
        let view = luminol_components::MapView::new(update_state, id)?;
        let tilepicker = luminol_components::Tilepicker::new(update_state, id)?;

        let map = update_state
            .data
            .get_or_load_map(id, update_state.filesystem);
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets[map.tileset_id];

        let mut passages = luminol_data::Table2::new(map.data.xsize(), map.data.ysize());
        luminol_graphics::collision::calculate_passages(
            &tileset.passages,
            &tileset.priorities,
            &map.data,
            &map.events,
            |x, y, passage| passages[(x, y)] = passage,
        );

        Ok(Self {
            id,

            view,
            tilepicker,

            dragging_event: false,
            drawing_shape: false,
            event_windows: luminol_core::Windows::default(),
            force_close: false,

            event_drag_offset: None,

            layer_cache: vec![0; map.data.xsize() * map.data.ysize()],

            dfs_cache: vec![false; map.data.xsize() * map.data.ysize()],
            brush_layer_cache: vec![0; map.data.xsize() * map.data.ysize()],
            drawing_shape_pos: None,

            history: VecDeque::with_capacity(HISTORY_SIZE),
            redo_history: Vec::with_capacity(HISTORY_SIZE),
            tilemap_undo_cache: vec![0; map.data.xsize() * map.data.ysize()],
            tilemap_undo_cache_layer: 0,

            passages,
        })
    }
}

impl luminol_core::Tab for Tab {
    fn name(&self, update_state: &luminol_core::UpdateState<'_>) -> String {
        let map_infos = update_state.data.map_infos();
        format!("Map {}: {}", self.id, map_infos[&self.id].name)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_map").with(self.id)
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }

    fn show(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
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
                        luminol_components::SelectedLayer::Events => "Events ⏷".to_string(),
                        luminol_components::SelectedLayer::Tiles(layer) => {
                            format!("Layer {} ⏷", layer + 1)
                        }
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
                                    luminol_components::SelectedLayer::Tiles(index),
                                    format!("Layer {}", index + 1),
                                );
                                columns[1].checkbox(layer, "👁");
                            }

                            // Display event layer.
                            columns[0].selectable_value(
                                &mut self.view.selected_layer,
                                luminol_components::SelectedLayer::Events,
                                egui::RichText::new("Events").italics(),
                            );
                            columns[1].checkbox(&mut self.view.event_enabled, "👁");

                            columns[0].label(egui::RichText::new("Fog").underline());
                            columns[1].checkbox(&mut self.view.map.fog_enabled, "👁");

                            columns[0].label(egui::RichText::new("Collision").underline());
                            columns[1].checkbox(&mut self.view.map.coll_enabled, "👁");
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
                egui::ScrollArea::both().show_viewport(ui, |ui, rect| {
                    self.tilepicker
                        .ui(update_state, ui, rect, self.view.map.coll_enabled);
                    ui.separator();
                });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Get the map.
                let mut map = update_state.data.get_map(self.id);
                let tilesets = update_state.data.tilesets();
                let tileset = &tilesets[map.tileset_id];

                // Save the state of the selected layer into the cache
                if let luminol_components::SelectedLayer::Tiles(tile_layer) =
                    self.view.selected_layer
                {
                    self.layer_cache
                        .copy_from_slice(&map.data.layer_as_slice(tile_layer));
                }

                let response = self.view.ui(
                    ui,
                    &update_state.graphics,
                    &map,
                    &self.tilepicker,
                    self.dragging_event,
                    self.drawing_shape,
                    self.drawing_shape_pos,
                    matches!(update_state.toolbar.pencil, luminol_core::Pencil::Pen),
                );

                let layers_max = map.data.zsize();
                let map_x = self.view.cursor_pos.x as i32;
                let map_y = self.view.cursor_pos.y as i32;

                if self.dragging_event && self.view.selected_event_id.is_none() {
                    self.dragging_event = false;
                    self.event_drag_offset = None;
                }

                if !response.dragged_by(egui::PointerButton::Primary) {
                    if self.drawing_shape {
                        self.drawing_shape = false;
                    }

                    if self.drawing_shape_pos.is_some() {
                        self.drawing_shape_pos = None;
                        self.redo_history.clear();
                        if self.history.len() == HISTORY_SIZE {
                            self.history.pop_front();
                        }
                        self.history.push_back(HistoryEntry::Tiles {
                            layer: self.tilemap_undo_cache_layer,
                            delta: (0..map.data.ysize())
                                .cartesian_product(0..map.data.xsize())
                                .filter_map(|(y, x)| {
                                    let old_id = self.tilemap_undo_cache[x + y * map.data.xsize()];
                                    if map.data[(x, y, self.tilemap_undo_cache_layer)] != old_id {
                                        Some((x, y, old_id))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        });
                    }
                }

                if let luminol_components::SelectedLayer::Tiles(tile_layer) =
                    self.view.selected_layer
                {
                    // Before drawing tiles, save the state of the current layer so we can undo it
                    // later if we need to
                    if response.drag_started_by(egui::PointerButton::Primary)
                        && !ui.input(|i| i.modifiers.command)
                    {
                        self.tilemap_undo_cache_layer = tile_layer;
                        for i in 0..self.layer_cache.len() {
                            self.tilemap_undo_cache[i] = self.layer_cache[i];
                        }
                    }

                    // Tile drawing
                    if response.dragged_by(egui::PointerButton::Primary)
                        && !ui.input(|i| i.modifiers.command)
                    {
                        self.handle_brush(
                            map_x as usize,
                            map_y as usize,
                            tile_layer,
                            update_state.toolbar.pencil,
                            &mut map,
                        );
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
                        let event = map.events.remove(selected_event_id);
                        let sprites = self.view.events.try_remove(selected_event_id).ok();
                        self.redo_history.clear();
                        if self.history.len() == HISTORY_SIZE {
                            self.history.pop_front();
                        }
                        self.history
                            .push_back(HistoryEntry::EventDeleted { event, sprites });
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

                                    // Also save the original position of the event to the history
                                    self.redo_history.clear();
                                    if self.history.len() == HISTORY_SIZE {
                                        self.history.pop_front();
                                    }
                                    self.history.push_back(HistoryEntry::EventMoved {
                                        id: selected_event_id,
                                        x: selected_event.x,
                                        y: selected_event.y,
                                    });
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
                        if let Some(id) = self.add_event(&mut map) {
                            self.redo_history.clear();
                            if self.history.len() == HISTORY_SIZE {
                                self.history.pop_front();
                            }
                            self.history.push_back(HistoryEntry::EventCreated(id));
                        }
                    }
                }

                // Handle undo/redo keypresses
                let is_dragged_by_primary = response.dragged_by(egui::PointerButton::Primary);
                let is_undo_pressed = ui.input(|i| {
                    i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z)
                });
                let is_redo_pressed = ui.input(|i| {
                    i.modifiers.command
                        && (i.modifiers.shift || i.key_pressed(egui::Key::Y))
                        && (!i.modifiers.shift || i.key_pressed(egui::Key::Z))
                });
                if !is_dragged_by_primary && (is_undo_pressed || is_redo_pressed) {
                    let new_entry = match if is_undo_pressed {
                        self.history.pop_back()
                    } else {
                        self.redo_history.pop()
                    } {
                        None => None,

                        Some(HistoryEntry::Tiles { layer, mut delta }) => {
                            for d in delta.iter_mut() {
                                let position = (d.0, d.1, layer);
                                let new_id = d.2;
                                *d = (d.0, d.1, map.data[position]);
                                map.data[position] = new_id;
                                self.view.map.set_tile(
                                    &update_state.graphics.render_state,
                                    new_id,
                                    position,
                                );
                            }
                            Some(HistoryEntry::Tiles { layer, delta })
                        }

                        Some(HistoryEntry::EventMoved { id, x, y }) => {
                            let event = map.events.get_mut(id).unwrap();
                            let new_entry = Some(HistoryEntry::EventMoved {
                                id,
                                x: event.x,
                                y: event.y,
                            });
                            event.x = x;
                            event.y = y;
                            new_entry
                        }

                        Some(HistoryEntry::EventCreated(id)) => {
                            let event = map.events.remove(id);
                            let sprites = self.view.events.try_remove(id).ok();
                            Some(HistoryEntry::EventDeleted { event, sprites })
                        }

                        Some(HistoryEntry::EventDeleted { event, sprites }) => {
                            let id = event.id;
                            map.events.insert(id, event);
                            if let Some(sprites) = sprites {
                                self.view.events.insert(id, sprites);
                            }
                            Some(HistoryEntry::EventCreated(id))
                        }
                    };

                    if let Some(new_entry) = new_entry {
                        if is_undo_pressed {
                            self.redo_history.push(new_entry);
                        } else {
                            self.history.push_back(new_entry);
                        }
                    }
                }

                for (_, event) in map.events.iter_mut() {
                    event.extra_data.is_editor_open = false;
                }

                if let luminol_components::SelectedLayer::Tiles(tile_layer) =
                    self.view.selected_layer
                {
                    // Write the buffered tile changes to the tilemap
                    for y in 0..map.data.ysize() {
                        for x in 0..map.data.xsize() {
                            let position = (x, y, tile_layer);
                            let new_tile_id = map.data[position];
                            if new_tile_id != self.layer_cache[x + y * map.data.xsize()] {
                                self.view.map.set_tile(
                                    &update_state.graphics.render_state,
                                    new_tile_id,
                                    position,
                                );
                            }
                        }
                    }
                }

                // Update the collision preview
                luminol_graphics::collision::calculate_passages(
                    &tileset.passages,
                    &tileset.priorities,
                    &map.data,
                    &map.events,
                    |x, y, passage| {
                        if self.passages[(x, y)] != passage {
                            self.view.map.set_passage(
                                &update_state.graphics.render_state,
                                passage,
                                (x, y),
                            );
                            self.passages[(x, y)] = passage;
                        }
                    },
                );
            })
        });

        self.event_windows.display(ui.ctx(), update_state);
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
