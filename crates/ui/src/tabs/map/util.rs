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

impl super::Tab {
    pub(super) fn recompute_autotile(
        &self,
        map: &luminol_data::rpg::Map,
        position: (usize, usize, usize),
    ) -> i16 {
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

    pub(super) fn set_tile(
        &self,
        map: &mut luminol_data::rpg::Map,
        tile: luminol_components::SelectedTile,
        position: (usize, usize, usize),
    ) {
        if self.brush_density != 1. {
            if self.brush_density == 0. {
                return;
            }

            // Pick a pseudorandom normal f32 uniformly in the interval [0, 1)
            let mut preimage = [0u8; 40];
            preimage[0..16].copy_from_slice(&self.brush_seed);
            preimage[16..24].copy_from_slice(&(position.0 as u64).to_le_bytes());
            preimage[24..32].copy_from_slice(&(position.1 as u64).to_le_bytes());
            preimage[32..40].copy_from_slice(&(position.2 as u64).to_le_bytes());
            let image = (murmur3::murmur3_32(&mut std::io::Cursor::new(preimage), 1729).unwrap()
                & 16777215) as f32
                / 16777216f32;

            // Set the tile only if that's less than the brush density
            if image >= self.brush_density {
                return;
            }
        }

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

    pub(super) fn add_event(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        map: &mut luminol_data::rpg::Map,
    ) -> Option<usize> {
        let mut first_vacant_id = 1;
        let mut max_event_id = 0;

        for (_, event) in map.events.iter() {
            if event.id == first_vacant_id {
                first_vacant_id += 1;
            }
            max_event_id = event.id;

            if event.x == self.view.cursor_pos.x as i32 && event.y == self.view.cursor_pos.y as i32
            {
                return None;
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
            return None;
        };

        let event = luminol_data::rpg::Event::new(
            self.view.cursor_pos.x as i32,
            self.view.cursor_pos.y as i32,
            new_event_id,
        );

        self.event_windows
            .add_window(crate::windows::event_edit::Window::new(
                update_state,
                &event,
                self.id,
                map.tileset_id,
            ));

        map.events.insert(new_event_id, event);
        Some(new_event_id)
    }

    pub(super) fn push_to_history(
        &mut self,
        update_state: &luminol_core::UpdateState<'_>,
        map: &mut luminol_data::rpg::Map,
        entry: super::HistoryEntry,
    ) {
        update_state.modified.set(true);
        map.modified = true;
        self.redo_history.clear();
        if self.history.len() == super::HISTORY_SIZE {
            self.history.pop_front();
        }
        self.history.push_back(entry);
    }
}
