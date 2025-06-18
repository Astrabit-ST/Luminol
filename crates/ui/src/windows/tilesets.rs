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
use crate::modals::graphic_picker::{
    autotile::Modal as AutotileModal, fog::Modal as FogModal, label::Modal as LabelModal,
    panorama::Modal as PanoramaModal, tileset::Modal as TilesetModal,
};

const SQUARE_PASSAGE_MASK: [usize; 14] = [20, 21, 22, 23, 33, 34, 35, 36, 37, 42, 43, 45, 46, 47];

#[derive(Clone, Copy, Debug, PartialEq)]
enum Passage {
    X,
    O,
    Square,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    Down,
    Left,
    Right,
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Overlay {
    Passage(Passage),
    Direction(Direction),
    Character(char),
    Approx,
    Diamond,
    Dot,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[derive(strum::Display, strum::EnumIter)]
enum Property {
    Passage,
    #[strum(to_string = "Passage (4-directional)")]
    Passage4Directional,
    Priority,
    #[strum(to_string = "Bush Flag")]
    BushFlag,
    #[strum(to_string = "Counter Flag")]
    CounterFlag,
    #[strum(to_string = "Terrain Tag")]
    TerrainTag,
}

fn paint_overlay(ui: &mut egui::Ui, pos: egui::Pos2, hovered: bool, overlay: Overlay) {
    let text = match overlay {
        Overlay::Passage(Passage::X) => '\u{f00d}',
        Overlay::Passage(Passage::O) => '\u{eabc}',
        Overlay::Passage(Passage::Square) => '\u{f0a14}',
        Overlay::Direction(Direction::Down) => '\u{eb6e}',
        Overlay::Direction(Direction::Left) => '\u{eb6f}',
        Overlay::Direction(Direction::Right) => '\u{eb70}',
        Overlay::Direction(Direction::Up) => '\u{eb71}',
        Overlay::Character(c) => c,
        Overlay::Approx => '\u{2248}',
        Overlay::Diamond => '\u{25c6}',
        Overlay::Dot => '\u{b7}',
    };

    ui.painter().text(
        pos + match overlay {
            Overlay::Passage(Passage::O) => egui::vec2(1., 0.),
            Overlay::Direction(_) => egui::vec2(-1., 0.),
            _ => egui::Vec2::ZERO,
        },
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId {
            size: match overlay {
                Overlay::Passage(Passage::X) => 10.,
                Overlay::Passage(Passage::O) => 14.,
                Overlay::Passage(Passage::Square) => 18.,
                Overlay::Direction(_) => 16.,
                Overlay::Character(_) => 10.,
                Overlay::Approx => 24.,
                Overlay::Diamond => 24.,
                Overlay::Dot => 32.,
            },
            family: egui::FontFamily::Name("Iosevka Term".into()),
        },
        egui::Color32::BLACK.gamma_multiply(if hovered { 0.8 } else { 0.3 }),
    );

    ui.painter().text(
        pos + match overlay {
            Overlay::Passage(Passage::Square) => egui::vec2(0., 1.),
            Overlay::Direction(_) => egui::vec2(-1., 0.),
            Overlay::Dot => egui::vec2(0., 1.),
            _ => egui::Vec2::ZERO,
        },
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId {
            size: match overlay {
                Overlay::Passage(Passage::X) => 19.,
                Overlay::Passage(Passage::O) => 27.,
                Overlay::Passage(Passage::Square) => 38.,
                Overlay::Direction(_) => 19.,
                Overlay::Character(_) => 19.,
                Overlay::Approx => 27.,
                Overlay::Diamond => 27.,
                Overlay::Dot => 38.,
            },
            family: egui::FontFamily::Name("Iosevka Term".into()),
        },
        egui::Color32::BLACK.gamma_multiply(if hovered { 0.8 } else { 0.3 }),
    );

    ui.painter().text(
        pos + match overlay {
            Overlay::Direction(_) => egui::vec2(-1., 0.),
            _ => egui::Vec2::ZERO,
        },
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId {
            size: match overlay {
                Overlay::Passage(Passage::X) => 16.,
                Overlay::Passage(Passage::O) => 24.,
                Overlay::Passage(Passage::Square) => 32.,
                Overlay::Direction(_) => 16.,
                Overlay::Character(_) => 16.,
                Overlay::Approx => 24.,
                Overlay::Diamond => 24.,
                Overlay::Dot => 32.,
            },
            family: egui::FontFamily::Name("Iosevka Term".into()),
        },
        egui::Color32::WHITE.gamma_multiply(if hovered { 0.9 } else { 0.5 }),
    );
}

/// Database - Tilesets management window.
pub struct Window {
    selected_tileset_name: Option<String>,
    property: Property,

    previous_tileset: Option<usize>,

    autotile_modals: [AutotileModal; 7],
    tileset_modal: TilesetModal,
    panorama_modal: PanoramaModal,
    fog_modal: FogModal,
    battleback_modal: LabelModal,

    tilepicker: Option<Tilepicker>,
    view: DatabaseView,

    autotiles_view_is_depersisted: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            selected_tileset_name: None,
            property: Property::Passage,
            previous_tileset: None,
            tilepicker: None,
            autotile_modals: core::array::from_fn(|i| {
                AutotileModal::new(format!("autotile_graphic_picker_{i}").into(), i)
            }),
            tileset_modal: TilesetModal::new("tileset_graphic_picker".into()),
            panorama_modal: PanoramaModal::new("panorama_graphic_picker".into()),
            fog_modal: FogModal::new("fog_graphic_picker".into()),
            battleback_modal: LabelModal::new(
                "battleback_graphic_picker".into(),
                "Graphics/Battlebacks".into(),
            ),
            view: DatabaseView::new(),
            autotiles_view_is_depersisted: false,
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
            .min_height(600.)
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
                            ui.columns(2, |columns| {
                                let changed = columns[0]
                                    .add(Field::new(
                                        "Graphic",
                                        self.tileset_modal.button(tileset, update_state),
                                    ))
                                    .changed();
                                if changed {
                                    modified = true;
                                    needs_update = true;
                                }

                                let changed = columns[1]
                                    .add(Field::new(
                                        "Panorama",
                                        self.panorama_modal.button(tileset, update_state),
                                    ))
                                    .changed();
                                if changed {
                                    modified = true;
                                    needs_update = true;
                                }
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                let changed = columns[0]
                                    .add(Field::new(
                                        "Fog",
                                        self.fog_modal.button(tileset, update_state),
                                    ))
                                    .changed();
                                if changed {
                                    modified = true;
                                    needs_update = true;
                                }

                                let changed = columns[1]
                                    .add(Field::new(
                                        "Battleback",
                                        self.battleback_modal
                                            .button(&mut tileset.battleback_name.0, update_state),
                                    ))
                                    .changed();
                                if changed {
                                    modified = true;
                                    needs_update = true;
                                }
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            // Forget whether the collapsing header was open from the last time
                            // the editor was open
                            let ui_id = ui.make_persistent_id("autotiles_collapsing_header");
                            if !self.autotiles_view_is_depersisted {
                                self.autotiles_view_is_depersisted = true;
                                if let Some(h) =
                                    egui::collapsing_header::CollapsingState::load(ui.ctx(), ui_id)
                                {
                                    h.remove(ui.ctx());
                                }
                                ui.ctx().animate_bool_with_time(ui_id, false, 0.);
                            }

                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                ui_id,
                                false,
                            )
                            .show_header(ui, |ui| {
                                ui.with_cross_justify(|ui| {
                                    ui.label("Autotiles");
                                });
                            })
                            .body(|ui| {
                                let num_columns = 2;

                                let atlas_dirty =
                                    (0..7).chunks(num_columns).into_iter().enumerate().any(
                                        |(row_index, row)| {
                                            let mut row = row.peekable();
                                            let first_in_row = *row.peek().unwrap();
                                            ui.with_padded_stripe(row_index % 2 != 0, |ui| {
                                                ui.columns(num_columns, |columns| {
                                                    row.any(|i| {
                                                        columns[i - first_in_row]
                                                            .add(Field::new(
                                                                format!("Autotile {}", i + 1),
                                                                self.autotile_modals[i]
                                                                    .button(tileset, update_state),
                                                            ))
                                                            .changed()
                                                    })
                                                })
                                            })
                                            .inner
                                        },
                                    );

                                if atlas_dirty {
                                    modified = true;
                                    needs_update = true;
                                    update_state
                                        .graphics
                                        .atlas_loader
                                        .remove_atlas(tileset.tileset_name.0.as_deref());
                                }
                            });
                        });

                        if needs_update {
                            tileset.nonce += 1;
                            self.tileset_modal.reset(update_state, tileset);
                            self.panorama_modal.reset(update_state, tileset);
                            self.fog_modal.reset(update_state, tileset);
                            self.battleback_modal
                                .reset(update_state, &mut tileset.battleback_name.0);
                            for modal in self.autotile_modals.iter_mut() {
                                modal.reset(update_state, tileset);
                            }
                            self.tilepicker = Some(
                                Tilepicker::new(
                                    update_state,
                                    tileset.tileset_name.0.as_deref(),
                                    &tileset.autotile_names,
                                    &tileset.passages,
                                    None,
                                    false,
                                )
                                .hide_selection(),
                            );
                        }

                        ui.add(EnumComboBox::new(
                            (tileset.id, "property"),
                            &mut self.property,
                        ));

                        egui::ScrollArea::both()
                            .min_scrolled_height(256.)
                            .show_viewport(ui, |ui, scroll_rect| {
                                let tilepicker = self.tilepicker.as_mut().unwrap();
                                let tilepicker_response =
                                    tilepicker.ui(update_state, ui, scroll_rect);

                                let bottom = tilepicker.view.atlas.tileset_height() as usize / 32;
                                let first_row = ((scroll_rect.top().max(0.) / 32.).floor()
                                    as usize)
                                    .min(bottom);
                                let last_row = ((scroll_rect.bottom().max(0.) / 32.).ceil()
                                    as usize)
                                    .min(bottom);
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

                                    let tile_range = if y == 0 { 0..48 } else { 0..1 };

                                    let tile_passage_value = if tile_id >= tileset.passages.len() {
                                        0
                                    } else {
                                        tileset.passages[tile_id]
                                    };

                                    let tile_priority_value = if tile_id >= tileset.priorities.len()
                                    {
                                        0
                                    } else {
                                        tileset.priorities[tile_id]
                                    };

                                    let tile_terrain_value =
                                        if tile_id >= tileset.terrain_tags.len() {
                                            0
                                        } else {
                                            tileset.terrain_tags[tile_id]
                                        };

                                    // Determine the egui coordinates of this tile in the tilepicker
                                    let tile_rect = egui::Rect::from_min_size(
                                        egui::pos2(x as f32 * 32., y as f32 * 32.)
                                            + tilepicker_response.rect.min.to_vec2(),
                                        egui::Vec2::splat(32.),
                                    );
                                    let response =
                                        ui.allocate_rect(tile_rect, egui::Sense::click());

                                    match self.property {
                                        Property::Passage => {
                                            // Determine what the passage type is for this tile ID
                                            let passage = if y == 0
                                                && tile_passage_value & 0b10000 == 0b10000
                                            {
                                                Passage::Square
                                            } else if tile_passage_value & 0b01111 == 0b01111 {
                                                Passage::X
                                            } else {
                                                Passage::O
                                            };

                                            // Handle clicking on a tile to change its passage
                                            let passage = if response.clicked()
                                                || response.secondary_clicked()
                                            {
                                                let passage = if response.secondary_clicked() {
                                                    match passage {
                                                        Passage::O if y == 0 => Passage::Square,
                                                        Passage::O | Passage::Square => Passage::X,
                                                        Passage::X => Passage::O,
                                                    }
                                                } else {
                                                    match passage {
                                                        Passage::X if y == 0 => Passage::Square,
                                                        Passage::X | Passage::Square => Passage::O,
                                                        Passage::O => Passage::X,
                                                    }
                                                };
                                                if tile_id + tile_range.end > tileset.passages.len()
                                                {
                                                    tileset
                                                        .passages
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                for i in tile_range {
                                                    let new_tile_passage_value = match passage {
                                                        Passage::X => 0b01111,
                                                        Passage::O => 0b00000,
                                                        Passage::Square => {
                                                            if SQUARE_PASSAGE_MASK
                                                                .binary_search(&i)
                                                                .is_ok()
                                                            {
                                                                0b10000
                                                            } else {
                                                                0b11111
                                                            }
                                                        }
                                                    };
                                                    tileset.passages[tile_id + i] =
                                                        new_tile_passage_value
                                                            | (tileset.passages[tile_id + i]
                                                                & !0b11111);
                                                }
                                                modified = true;
                                                passage
                                            } else {
                                                passage
                                            };

                                            // Draw a symbol on top of the tile depending on the passage
                                            paint_overlay(
                                                ui,
                                                tile_rect.center(),
                                                response.hovered(),
                                                Overlay::Passage(passage),
                                            );
                                        }

                                        Property::Passage4Directional => {
                                            // Find the direction within the tile that the cursor is
                                            // hovering over
                                            let direction = response
                                                .hovered()
                                                .then(|| {
                                                    response.hover_pos().map(|pos| {
                                                        let (min_index, _) = [
                                                            (pos - tile_rect.center_bottom())
                                                                .length(),
                                                            (pos - tile_rect.left_center())
                                                                .length(),
                                                            (pos - tile_rect.right_center())
                                                                .length(),
                                                            (pos - tile_rect.center_top()).length(),
                                                        ]
                                                        .iter()
                                                        .enumerate()
                                                        .min_by(|(_, a), (_, b)| a.total_cmp(b))
                                                        .unwrap();
                                                        match min_index {
                                                            0 => Direction::Down,
                                                            1 => Direction::Left,
                                                            2 => Direction::Right,
                                                            3 => Direction::Up,
                                                            _ => unreachable!(),
                                                        }
                                                    })
                                                })
                                                .flatten();

                                            // Handle clicking to change passage
                                            let (tile_passage_value, tile_passage_value_changed) =
                                                match (response.clicked()
                                                    || response.secondary_clicked())
                                                .then_some(direction)
                                                .flatten()
                                                {
                                                    Some(Direction::Down) => {
                                                        (tile_passage_value ^ 0b00001, true)
                                                    }
                                                    Some(Direction::Left) => {
                                                        (tile_passage_value ^ 0b00010, true)
                                                    }
                                                    Some(Direction::Right) => {
                                                        (tile_passage_value ^ 0b00100, true)
                                                    }
                                                    Some(Direction::Up) => {
                                                        (tile_passage_value ^ 0b01000, true)
                                                    }
                                                    _ => (tile_passage_value, false),
                                                };
                                            if tile_passage_value_changed {
                                                if tile_id + tile_range.end > tileset.passages.len()
                                                {
                                                    tileset
                                                        .passages
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                for i in tile_range {
                                                    tileset.passages[tile_id + i] =
                                                        (tile_passage_value & 0b11111)
                                                            | (tileset.passages[tile_id + i]
                                                                & !0b11111);
                                                }
                                                modified = true;
                                            }

                                            paint_overlay(
                                                ui,
                                                tile_rect
                                                    .center()
                                                    .lerp(tile_rect.center_bottom(), 0.5),
                                                direction == Some(Direction::Down),
                                                if tile_passage_value & 0b00001 == 0 {
                                                    Overlay::Direction(Direction::Down)
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );

                                            paint_overlay(
                                                ui,
                                                tile_rect
                                                    .center()
                                                    .lerp(tile_rect.left_center(), 0.5),
                                                direction == Some(Direction::Left),
                                                if tile_passage_value & 0b00010 == 0 {
                                                    Overlay::Direction(Direction::Left)
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );

                                            paint_overlay(
                                                ui,
                                                tile_rect
                                                    .center()
                                                    .lerp(tile_rect.right_center(), 0.5),
                                                direction == Some(Direction::Right),
                                                if tile_passage_value & 0b00100 == 0 {
                                                    Overlay::Direction(Direction::Right)
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );

                                            paint_overlay(
                                                ui,
                                                tile_rect
                                                    .center()
                                                    .lerp(tile_rect.center_top(), 0.5),
                                                direction == Some(Direction::Up),
                                                if tile_passage_value & 0b01000 == 0 {
                                                    Overlay::Direction(Direction::Up)
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );
                                        }

                                        Property::Priority => {
                                            // Handle clicking to change priority
                                            let tile_priority_value = if response.clicked()
                                                || response.secondary_clicked()
                                            {
                                                if tile_id + tile_range.end
                                                    > tileset.priorities.len()
                                                {
                                                    tileset
                                                        .priorities
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                let new_tile_priority_value =
                                                    if (0..=6).contains(&tile_priority_value) {
                                                        tile_priority_value
                                                    } else {
                                                        0
                                                    };
                                                let new_tile_priority_value =
                                                    if response.secondary_clicked() {
                                                        new_tile_priority_value - 1
                                                    } else {
                                                        new_tile_priority_value + 1
                                                    }
                                                    .rem_euclid(6);
                                                for i in tile_range {
                                                    tileset.priorities[tile_id + i] =
                                                        new_tile_priority_value;
                                                }
                                                modified = true;
                                                new_tile_priority_value
                                            } else {
                                                tile_priority_value
                                            };

                                            // Draw a symbol on top of the tile depending on the
                                            // priority
                                            paint_overlay(
                                                ui,
                                                tile_rect.center(),
                                                response.hovered(),
                                                match tile_priority_value {
                                                    1 => Overlay::Character('1'),
                                                    2 => Overlay::Character('2'),
                                                    3 => Overlay::Character('3'),
                                                    4 => Overlay::Character('4'),
                                                    5 => Overlay::Character('5'),
                                                    _ => Overlay::Dot,
                                                },
                                            );
                                        }

                                        Property::BushFlag => {
                                            // Handle clicking to change bush flag
                                            let tile_passage_value = if response.clicked()
                                                || response.secondary_clicked()
                                            {
                                                if tile_id + tile_range.end > tileset.passages.len()
                                                {
                                                    tileset
                                                        .passages
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                for i in tile_range {
                                                    tileset.passages[tile_id + i] ^= 0b01000000;
                                                }
                                                modified = true;
                                                tile_passage_value ^ 0b01000000
                                            } else {
                                                tile_passage_value
                                            };

                                            // Draw a symbol on top of the tile depending on the
                                            // bush flag
                                            paint_overlay(
                                                ui,
                                                tile_rect.center(),
                                                response.hovered(),
                                                if tile_passage_value & 0b01000000 != 0 {
                                                    Overlay::Approx
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );
                                        }

                                        Property::CounterFlag => {
                                            // Handle clicking to change counter flag
                                            let tile_passage_value = if response.clicked()
                                                || response.secondary_clicked()
                                            {
                                                if tile_id + tile_range.end > tileset.passages.len()
                                                {
                                                    tileset
                                                        .passages
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                for i in tile_range {
                                                    tileset.passages[tile_id + i] ^= 0b10000000;
                                                }
                                                modified = true;
                                                tile_passage_value ^ 0b10000000
                                            } else {
                                                tile_passage_value
                                            };

                                            // Draw a symbol on top of the tile depending on the
                                            // counter flag
                                            paint_overlay(
                                                ui,
                                                tile_rect.center(),
                                                response.hovered(),
                                                if tile_passage_value & 0b10000000 != 0 {
                                                    Overlay::Diamond
                                                } else {
                                                    Overlay::Dot
                                                },
                                            );
                                        }

                                        Property::TerrainTag => {
                                            // Handle clicking to change terrain tag
                                            let tile_terrain_value = if response.clicked()
                                                || response.secondary_clicked()
                                            {
                                                if tile_id + tile_range.end
                                                    > tileset.terrain_tags.len()
                                                {
                                                    tileset
                                                        .terrain_tags
                                                        .resize(tile_id + tile_range.end);
                                                }
                                                let new_tile_terrain_value =
                                                    if (0..=8).contains(&tile_terrain_value) {
                                                        tile_terrain_value
                                                    } else {
                                                        0
                                                    };
                                                let new_tile_terrain_value =
                                                    if response.secondary_clicked() {
                                                        new_tile_terrain_value - 1
                                                    } else {
                                                        new_tile_terrain_value + 1
                                                    }
                                                    .rem_euclid(8);
                                                for i in tile_range {
                                                    tileset.terrain_tags[tile_id + i] =
                                                        new_tile_terrain_value;
                                                }
                                                modified = true;
                                                new_tile_terrain_value
                                            } else {
                                                tile_terrain_value
                                            };

                                            // Draw a symbol on top of the tile depending on the
                                            // terrain tag
                                            paint_overlay(
                                                ui,
                                                tile_rect.center(),
                                                response.hovered(),
                                                match tile_terrain_value & 0b111 {
                                                    1 => Overlay::Character('1'),
                                                    2 => Overlay::Character('2'),
                                                    3 => Overlay::Character('3'),
                                                    4 => Overlay::Character('4'),
                                                    5 => Overlay::Character('5'),
                                                    6 => Overlay::Character('6'),
                                                    7 => Overlay::Character('7'),
                                                    _ => Overlay::Dot,
                                                },
                                            );
                                        }
                                    };
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
