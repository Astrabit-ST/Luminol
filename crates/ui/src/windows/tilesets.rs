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
use luminol_core::Modal;

use crate::components::{DatabaseView, EnumComboBox, Field, Tilepicker, UiExt};
use crate::modals::graphic_picker::tileset::Modal as TilesetModal;

const SQUARE_PASSAGE_MASK: [usize; 14] = [20, 21, 22, 23, 33, 34, 35, 36, 37, 42, 43, 45, 46, 47];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Passage {
    X,
    O,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(strum::Display, strum::EnumIter)]
enum Property {
    Passage,
}

/// Database - Tilesets management window.
pub struct Window {
    selected_tileset_name: Option<String>,
    property: Property,

    previous_tileset: Option<usize>,

    tileset_modal: TilesetModal,

    tilepicker: Option<Tilepicker>,
    view: DatabaseView,
}

impl Window {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let tilesets = update_state.data.tilesets();
        let tileset = &tilesets.data[0];
        Self {
            selected_tileset_name: None,
            property: Property::Passage,
            previous_tileset: None,
            tilepicker: None,
            tileset_modal: TilesetModal::new(tileset, "tileset_graphic_picker".into()),
            view: DatabaseView::new(),
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("tileset_editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let data = std::mem::take(update_state.data); // take data to avoid borrow checker issues
        let mut tilesets = data.tilesets();

        let mut modified = false;

        self.selected_tileset_name = None;

        let name = if let Some(name) = &self.selected_tileset_name {
            format!("Editing tileset {:?}", name)
        } else {
            "Tileset Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Tilesets",
                    &mut tilesets.data,
                    |tileset| format!("{:0>4}: {}", tileset.id + 1, tileset.name),
                    |ui, tilesets, id, update_state| {
                        let tileset = &mut tilesets[id];
                        self.selected_tileset_name = Some(tileset.name.clone());
                        let mut needs_update = self.previous_tileset != Some(tileset.id);

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut tileset.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            let changed = ui
                                .add(Field::new(
                                    "Graphic",
                                    self.tileset_modal.button(tileset, update_state),
                                ))
                                .changed();
                            if changed {
                                modified = true;
                                needs_update = true;
                            }
                        });

                        if needs_update {
                            self.tileset_modal.reset(update_state, tileset);
                            self.tilepicker = Some(
                                Tilepicker::new(
                                    update_state,
                                    tileset.tileset_name.as_deref(),
                                    &tileset.autotile_names,
                                    &tileset.passages,
                                    None,
                                )
                                .hide_selection(),
                            );
                        }

                        ui.add(EnumComboBox::new(
                            (tileset.id, "property"),
                            &mut self.property,
                        ));

                        egui::ScrollArea::both().show_viewport(ui, |ui, scroll_rect| {
                            let tilepicker = self.tilepicker.as_mut().unwrap();
                            let tilepicker_response = tilepicker.ui(update_state, ui, scroll_rect);

                            let bottom = tilepicker.view.atlas.tileset_height() as usize / 32;
                            let first_row =
                                ((scroll_rect.top().max(0.) / 32.).floor() as usize).min(bottom);
                            let last_row =
                                ((scroll_rect.bottom().max(0.) / 32.).ceil() as usize).min(bottom);
                            let first_col =
                                ((scroll_rect.left().max(0.) / 32.).floor() as usize).min(7);
                            let last_col =
                                ((scroll_rect.right().max(0.) / 32.).ceil() as usize).min(7);

                            for (y, x) in
                                (first_row..=last_row).cartesian_product(first_col..=last_col)
                            {
                                let tile_id = if y == 0 {
                                    x * 48
                                } else {
                                    (y - 1) * 8 + x + 384
                                };

                                // Determine what the passage type is for this tile ID
                                let value = if tile_id >= tileset.passages.len() {
                                    0
                                } else {
                                    tileset.passages[tile_id]
                                };
                                let passage = if y == 0 && value & 0b10000 == 0b10000 {
                                    Passage::Square
                                } else if value & 0b01111 == 0b01111 {
                                    Passage::X
                                } else {
                                    Passage::O
                                };

                                // Determine the egui coordinates of this tile in the tilepicker
                                let rect = egui::Rect::from_min_size(
                                    egui::pos2(x as f32 * 32., y as f32 * 32.)
                                        + tilepicker_response.rect.min.to_vec2(),
                                    egui::Vec2::splat(32.),
                                );
                                let response = ui.allocate_rect(rect, egui::Sense::click());

                                // Handle clicking on a tile to change its passage
                                let passage = if response.clicked() {
                                    let passage = match passage {
                                        Passage::X if y == 0 => Passage::Square,
                                        Passage::X | Passage::Square => Passage::O,
                                        Passage::O => Passage::X,
                                    };
                                    let range = if y == 0 { 0..48 } else { 0..1 };
                                    if tile_id + range.end > tileset.passages.len() {
                                        tileset.passages.resize(tile_id + range.end);
                                    }
                                    for i in range {
                                        let new_value = match passage {
                                            Passage::X => 0b01111,
                                            Passage::O => 0b00000,
                                            Passage::Square => {
                                                if SQUARE_PASSAGE_MASK.binary_search(&i).is_ok() {
                                                    0b10000
                                                } else {
                                                    0b11111
                                                }
                                            }
                                        };
                                        tileset.passages[tile_id + i] =
                                            new_value | (tileset.passages[tile_id + i] & !0b11111);
                                    }
                                    modified = true;
                                    passage
                                } else {
                                    passage
                                };

                                // Draw a symbol on top of the tile depending on the passage
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    match passage {
                                        Passage::X => '\u{f00d}',
                                        Passage::O => '\u{eabc}',
                                        Passage::Square => '\u{f0a14}',
                                    },
                                    egui::FontId {
                                        size: match passage {
                                            Passage::X => 16.,
                                            Passage::O => 24.,
                                            Passage::Square => 32.,
                                        },
                                        family: egui::FontFamily::Name("Iosevka Term".into()),
                                    },
                                    egui::Color32::WHITE,
                                );
                            }
                        });

                        self.previous_tileset = Some(tileset.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            tilesets.modified = true;
        }

        drop(tilesets);

        *update_state.data = data; // restore data
    }
}
