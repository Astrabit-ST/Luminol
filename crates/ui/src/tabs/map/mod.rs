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

#![allow(unused_imports)]
use egui::Pos2;
use std::{cell::RefMut, collections::HashMap, collections::VecDeque};

const HISTORY_SIZE: usize = 50;

struct EventDragInfo {
    /// ID of the event being dragged
    id: usize,
    /// Original x position of the event at the start of the drag
    x: i32,
    /// Original y position of the event at the start of the drag
    y: i32,
    /// Difference between the dragged event's tile and the cursor position, at the start of the
    /// drag
    offset: egui::Vec2,
}

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

    drawing_shape: bool,
    event_windows: luminol_core::Windows,
    force_close: bool,

    event_drag_info: Option<EventDragInfo>,

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

    /// Brush density between 0 and 1 inclusive; determines the proportion of randomly chosen tiles
    /// the brush draws on if less than 1
    brush_density: f32,
    /// Seed for the PRNG used for the brush when brush density is less than 1
    brush_seed: [u8; 16],

    /// Asynchronous task used to save the map as an image file
    save_as_image_promise: Option<poll_promise::Promise<color_eyre::Result<()>>>,
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
        sprite: Option<luminol_graphics::Event>,
    },
}

impl Tab {
    /// Create a new map editor.
    pub fn new(
        id: usize,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) -> color_eyre::Result<Self> {
        // *sigh*
        // borrow checker.
        let view = luminol_components::MapView::new(update_state, id)?;
        let tilepicker = luminol_components::Tilepicker::new(update_state, id)?;

        let map = update_state.data.get_or_load_map(
            id,
            update_state.filesystem,
            update_state.project_config.as_ref().unwrap(),
        );
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets.data[map.tileset_id];

        let mut passages = luminol_data::Table2::new(map.data.xsize(), map.data.ysize());
        luminol_graphics::Collision::calculate_passages(
            &tileset.passages,
            &tileset.priorities,
            &map.data,
            Some(&map.events),
            (0..map.data.zsize()).rev(),
            |x, y, passage| passages[(x, y)] = passage,
        );

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
        brush_seed[8..16].copy_from_slice(&(id as u64).to_le_bytes());

        Ok(Self {
            id,

            view,
            tilepicker,

            drawing_shape: false,
            event_windows: luminol_core::Windows::default(),
            force_close: false,

            event_drag_info: None,

            layer_cache: vec![0; map.data.xsize() * map.data.ysize()],

            dfs_cache: vec![false; map.data.xsize() * map.data.ysize()],
            brush_layer_cache: vec![0; map.data.xsize() * map.data.ysize()],
            drawing_shape_pos: None,

            history: VecDeque::with_capacity(HISTORY_SIZE),
            redo_history: Vec::with_capacity(HISTORY_SIZE),
            tilemap_undo_cache: vec![0; map.data.xsize() * map.data.ysize()],
            tilemap_undo_cache_layer: 0,

            passages,

            brush_density: 1.,
            brush_seed,

            save_as_image_promise: None,
        })
    }
}

impl luminol_core::Tab for Tab {
    fn name(&self, update_state: &luminol_core::UpdateState<'_>) -> String {
        let map_infos = update_state.data.map_infos();
        format!(
            "{}Map {}: {}",
            if update_state.data.get_map(self.id).modified {
                "*"
            } else {
                ""
            },
            self.id,
            map_infos.data[&self.id].name,
        )
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_map").with(self.id)
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        is_focused: bool,
    ) {
        self.brush_density = update_state.toolbar.brush_density;

        // Display the toolbar.
        // FIXME: find a proper place for this toolbar! it looks very out of place right now.
        egui::TopBottomPanel::top(format!("map_{}_toolbar", self.id)).show_inside(ui, |ui| {
            egui::Frame::none()
                .outer_margin(egui::Margin {
                    bottom: ui.spacing().item_spacing.y,
                    ..egui::Margin::ZERO
                })
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui::Slider::new(&mut self.view.scale, 15.0..=300.)
                                .text("Scale")
                                .logarithmic(true)
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
                                egui::Grid::new(self.id().with("layer_select"))
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.label(egui::RichText::new("Panorama").underline());
                                        ui.checkbox(&mut self.view.map.pano_enabled, "👁");
                                        ui.end_row();

                                        for (index, layer) in self
                                            .view
                                            .map
                                            .tiles
                                            .enabled_layers
                                            .iter_mut()
                                            .enumerate()
                                        {
                                            ui.columns(1, |columns| {
                                                columns[0].selectable_value(
                                                    &mut self.view.selected_layer,
                                                    luminol_components::SelectedLayer::Tiles(index),
                                                    format!("Layer {}", index + 1),
                                                );
                                            });
                                            ui.checkbox(layer, "👁");
                                            ui.end_row();
                                        }

                                        // Display event layer.
                                        ui.columns(1, |columns| {
                                            columns[0].selectable_value(
                                                &mut self.view.selected_layer,
                                                luminol_components::SelectedLayer::Events,
                                                egui::RichText::new("Events").italics(),
                                            );
                                        });
                                        ui.checkbox(&mut self.view.map.event_enabled, "👁");
                                        ui.end_row();

                                        ui.label(egui::RichText::new("Fog").underline());
                                        ui.checkbox(&mut self.view.map.fog_enabled, "👁");
                                        ui.end_row();

                                        ui.label(egui::RichText::new("Collision").underline());
                                        ui.checkbox(&mut self.view.map.coll_enabled, "👁");
                                        ui.end_row();

                                        ui.label(egui::RichText::new("Grid").underline());
                                        ui.checkbox(&mut self.view.map.grid_enabled, "👁");
                                        ui.end_row();
                                    });
                            },
                        );

                        ui.separator();

                        ui.menu_button("Display options ⏷", |ui| {
                            ui.checkbox(&mut self.view.visible_display, "Display visible area")
                                .on_hover_text("Display the visible area in-game (640x480)");
                            ui.checkbox(&mut self.view.move_preview, "Preview event move routes")
                                .on_hover_text("Preview event page move routes");
                            ui.checkbox(&mut self.view.snap_to_grid, "Snap to grid")
                                .on_hover_text("Snaps the viewport to the tile grid");
                            ui.checkbox(
                                &mut self.view.darken_unselected_layers,
                                "Darken unselected layers",
                            )
                            .on_hover_text("Toggles darkening unselected layers");
                            ui.checkbox(&mut self.view.display_tile_ids, "Display tile IDs")
                                .on_disabled_hover_text(
                                    "Display the tile IDs of the currently selected layer",
                                );
                        });

                        ui.separator();

                        if ui.button("Save map preview").clicked()
                            && self.save_as_image_promise.is_none()
                        {
                            self.save_as_image_promise =
                                Some(luminol_core::spawn_future(self.view.save_as_image(
                                    &update_state.graphics,
                                    &update_state.data.get_map(self.id),
                                )))
                        }

                        /*
                        if map.preview_move_route.is_some()
                        && ui.button("Clear move route preview").clicked()
                        {
                            map.preview_move_route = None;
                        }
                        */
                    });
                });
        });

        // Display the tilepicker.
        let spacing = ui.spacing();
        let tilepicker_default_width = 256. + spacing.indent;
        egui::SidePanel::left(format!("map_{}_tilepicker", self.id))
            .default_width(tilepicker_default_width)
            .max_width(tilepicker_default_width)
            .show_inside(ui, |ui| {
                egui::ScrollArea::both()
                    .id_source(
                        update_state
                            .project_config
                            .as_ref()
                            .expect("project not loaded")
                            .project
                            .persistence_id,
                    )
                    .show_viewport(ui, |ui, rect| {
                        self.tilepicker.view.coll_enabled = self.view.map.coll_enabled;
                        self.tilepicker.view.grid_enabled = self.view.map.grid_enabled;
                        self.tilepicker.ui(update_state, ui, rect);
                        ui.separator();
                    });
            });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                // Get the map.
                let mut map = update_state.data.get_map(self.id);
                let tilesets = update_state.data.tilesets();
                let tileset = &tilesets.data[map.tileset_id];

                // Save the state of the selected layer into the cache
                if let luminol_components::SelectedLayer::Tiles(tile_layer) =
                    self.view.selected_layer
                {
                    self.layer_cache
                        .copy_from_slice(map.data.layer_as_slice(tile_layer));
                }

                let response = self.view.ui(
                    ui,
                    update_state,
                    &map,
                    &self.tilepicker,
                    self.event_drag_info.is_some(),
                    self.drawing_shape,
                    self.drawing_shape_pos,
                    matches!(update_state.toolbar.pencil, luminol_core::Pencil::Pen),
                    is_focused,
                );

                let _layers_max = map.data.zsize();
                let map_x = self.view.cursor_pos.x as i32;
                let map_y = self.view.cursor_pos.y as i32;

                let is_delete_pressed = is_focused
                    && ui.input(|i| {
                        i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                    });

                // If the user stopped dragging an event or the user tried to delete an event while
                // dragging it
                if self.event_drag_info.as_ref().is_some_and(|info| {
                    is_delete_pressed
                        || !response.dragged_by(egui::PointerButton::Primary)
                        || !self.view.selected_event_id.is_some_and(|id| info.id == id)
                }) {
                    let info = self.event_drag_info.take().unwrap();

                    // If the event has moved from its original position, save the original
                    // position to the history (we need to check if it has moved because otherwise
                    // it'll also be saved if the user just clicks or double-clicks the event)
                    if map
                        .events
                        .get(info.id)
                        .is_some_and(|event| event.x != info.x || event.y != info.y)
                    {
                        self.push_to_history(
                            update_state,
                            &mut map,
                            HistoryEntry::EventMoved {
                                id: info.id,
                                x: info.x,
                                y: info.y,
                            },
                        );
                    }
                }

                if !response.is_pointer_button_down_on()
                    || ui.input(|i| !i.pointer.button_down(egui::PointerButton::Primary))
                {
                    if self.drawing_shape {
                        self.drawing_shape = false;
                    }

                    if self.drawing_shape_pos.is_some() {
                        self.drawing_shape_pos = None;
                        let delta = (0..map.data.ysize())
                            .cartesian_product(0..map.data.xsize())
                            .filter_map(|(y, x)| {
                                let old_id = self.tilemap_undo_cache[x + y * map.data.xsize()];
                                (map.data[(x, y, self.tilemap_undo_cache_layer)] != old_id)
                                    .then_some((x, y, old_id))
                            })
                            .collect();
                        self.push_to_history(
                            update_state,
                            &mut map,
                            HistoryEntry::Tiles {
                                layer: self.tilemap_undo_cache_layer,
                                delta,
                            },
                        );
                    }
                }

                if let luminol_components::SelectedLayer::Tiles(tile_layer) =
                    self.view.selected_layer
                {
                    // Tile drawing
                    if response.is_pointer_button_down_on()
                        && ui.input(|i| {
                            i.pointer.button_down(egui::PointerButton::Primary)
                                && !i.modifiers.command
                        })
                    {
                        if self.drawing_shape_pos.is_none() {
                            // Before drawing tiles, save the state of the current layer so we can
                            // undo it later if we need to
                            self.tilemap_undo_cache_layer = tile_layer;
                            self.tilemap_undo_cache.copy_from_slice(&self.layer_cache);
                        }

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
                        || (is_focused && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        // Double-click/press enter on events to edit them
                        if ui.input(|i| !i.modifiers.command) {
                            let event = map.events[selected_event_id].clone();
                            self.event_windows.add_window(event_edit::Window::new(
                                update_state,
                                &event,
                                self.id,
                                map.tileset_id,
                            ));
                        }
                    }

                    // Press delete or backspace to delete the selected event
                    if is_delete_pressed {
                        let event = map.events.remove(selected_event_id);
                        let sprite = self.view.map.events.try_remove(selected_event_id).ok();
                        self.push_to_history(
                            update_state,
                            &mut map,
                            HistoryEntry::EventDeleted { event, sprite },
                        );
                    }

                    if let Some(hover_tile) = self.view.hover_tile {
                        // Allow drag and drop to move events
                        if self.event_drag_info.is_none()
                            && self.view.selected_event_is_hovered
                            && !response.double_clicked()
                            && response.drag_started_by(egui::PointerButton::Primary)
                        {
                            if let Some(selected_event) = map.events.get(selected_event_id) {
                                // If we just started dragging an event, save the offset between the
                                // cursor and the event's tile so that the event will be dragged
                                // with that offset from the cursor
                                if self.event_drag_info.is_none() {
                                    self.event_drag_info = Some(EventDragInfo {
                                        id: selected_event.id,
                                        x: selected_event.x,
                                        y: selected_event.y,
                                        offset: egui::Pos2::new(
                                            selected_event.x as f32,
                                            selected_event.y as f32,
                                        ) - hover_tile,
                                    });
                                };
                            }
                        }

                        if let Some(info) = &self.event_drag_info {
                            // If moving an event, move the dragged event's tile to the cursor
                            // after adjusting for drag offset, unless that would put the event
                            // on the same tile as an existing event
                            let adjusted_hover_tile = hover_tile + info.offset;
                            if egui::Rect::from_min_size(
                                egui::pos2(0., 0.),
                                egui::vec2(
                                    map.data.xsize() as f32 - 0.5,
                                    map.data.ysize() as f32 - 0.5,
                                ),
                            )
                            .contains(adjusted_hover_tile)
                                && !map.events.iter().any(|(_, e)| {
                                    adjusted_hover_tile.x == e.x as f32
                                        && adjusted_hover_tile.y == e.y as f32
                                })
                            {
                                if let Some(selected_event) = map.events.get_mut(selected_event_id)
                                {
                                    selected_event.x = adjusted_hover_tile.x as i32;
                                    selected_event.y = adjusted_hover_tile.y as i32;
                                }
                            }
                        }
                    }
                } else {
                    // Double-click/press enter on an empty space to add an event
                    // (hold shift to prevent events from being selected)
                    if response.double_clicked()
                        || (is_focused && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    {
                        if let Some(id) = self.add_event(update_state, &mut map) {
                            self.push_to_history(
                                update_state,
                                &mut map,
                                HistoryEntry::EventCreated(id),
                            );
                        }
                    }
                }

                // Handle undo/redo keypresses
                let is_dragged_by_primary = response.dragged_by(egui::PointerButton::Primary);
                let is_undo_pressed = is_focused
                    && ui.input(|i| {
                        i.modifiers.command && !i.modifiers.shift && i.key_pressed(egui::Key::Z)
                    });
                let is_redo_pressed = is_focused
                    && ui.input(|i| {
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
                            let sprite = self.view.map.events.try_remove(id).ok();
                            Some(HistoryEntry::EventDeleted { event, sprite })
                        }

                        Some(HistoryEntry::EventDeleted { event, sprite }) => {
                            let id = event.id;
                            map.events.insert(id, event);
                            if let Some(sprite) = sprite {
                                self.view.map.events.insert(id, sprite);
                            }
                            Some(HistoryEntry::EventCreated(id))
                        }
                    };

                    if let Some(new_entry) = new_entry {
                        update_state.modified.set(true);
                        map.modified = true;
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
                luminol_graphics::Collision::calculate_passages(
                    &tileset.passages,
                    &tileset.priorities,
                    &map.data,
                    if self.view.map.event_enabled {
                        Some(&map.events)
                    } else {
                        None
                    },
                    (0..map.data.zsize())
                        .filter(|&i| self.view.map.tiles.enabled_layers[i])
                        .rev(),
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

        if let Some(p) = self.save_as_image_promise.take() {
            match p.try_take() {
                Ok(Ok(())) => {}
                Ok(Err(error))
                    if !matches!(
                        error.root_cause().downcast_ref(),
                        Some(luminol_filesystem::Error::CancelledLoading)
                    ) =>
                {
                    luminol_core::error!(update_state.toasts, error);
                }
                Ok(Err(_)) => {}
                Err(p) => self.save_as_image_promise = Some(p),
            }
        }
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
