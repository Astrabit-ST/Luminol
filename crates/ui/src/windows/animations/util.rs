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

use luminol_filesystem::FileSystem;

use luminol_data::rpg::animation::Condition;

#[derive(Debug, Default)]
pub struct FlashMaps {
    pub none_hide: FlashMap<HideFlash>,
    pub hit_hide: FlashMap<HideFlash>,
    pub miss_hide: FlashMap<HideFlash>,
    pub none_target: FlashMap<ColorFlash>,
    pub hit_target: FlashMap<ColorFlash>,
    pub miss_target: FlashMap<ColorFlash>,
    pub none_screen: FlashMap<ColorFlash>,
    pub hit_screen: FlashMap<ColorFlash>,
    pub miss_screen: FlashMap<ColorFlash>,
}

impl FlashMaps {
    /// Determines what color the target flash should be for a given frame number and condition.
    pub fn compute_target(&self, frame: usize, condition: Condition) -> luminol_data::Color {
        match condition {
            Condition::None => self.none_target.compute(frame),
            Condition::Hit => self.hit_target.compute(frame),
            Condition::Miss => self.miss_target.compute(frame),
        }
    }

    /// Determines what color the screen flash should be for a given frame number and condition.
    pub fn compute_screen(&self, frame: usize, condition: Condition) -> luminol_data::Color {
        match condition {
            Condition::None => self.none_screen.compute(frame),
            Condition::Hit => self.hit_screen.compute(frame),
            Condition::Miss => self.miss_screen.compute(frame),
        }
    }

    /// Determines if the hide target flash is active for a given frame number and condition.
    pub fn compute_hide(&self, frame: usize, condition: Condition) -> bool {
        match condition {
            Condition::None => self.none_hide.compute(frame),
            Condition::Hit => self.hit_hide.compute(frame),
            Condition::Miss => self.miss_hide.compute(frame),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorFlash {
    pub color: luminol_data::Color,
    pub duration: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct HideFlash {
    pub duration: usize,
}

#[derive(Debug)]
pub struct FlashMap<T> {
    map: std::collections::BTreeMap<usize, std::collections::VecDeque<T>>,
}

impl<T> Default for FlashMap<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}

impl<T> FromIterator<(usize, T)> for FlashMap<T>
where
    T: Copy,
{
    fn from_iter<I: IntoIterator<Item = (usize, T)>>(iterable: I) -> Self {
        let mut map = Self::default();
        for (frame, flash) in iterable.into_iter() {
            map.append(frame, flash);
        }
        map
    }
}

impl<T> FlashMap<T>
where
    T: Copy,
{
    /// Adds a new flash into the map at the maximum rank.
    pub fn append(&mut self, frame: usize, flash: T) {
        self.map
            .entry(frame)
            .and_modify(|e| e.push_back(flash))
            .or_insert_with(|| [flash].into());
    }

    /// Adds a new flash into the map at the given rank.
    pub fn insert(&mut self, frame: usize, rank: usize, flash: T) {
        self.map
            .entry(frame)
            .and_modify(|e| e.insert(rank, flash))
            .or_insert_with(|| [flash].into());
    }

    /// Removes a flash from the map.
    pub fn remove(&mut self, frame: usize, rank: usize) -> T {
        let deque = self
            .map
            .get_mut(&frame)
            .expect("no flashes found for the given frame");
        let flash = deque.remove(rank).expect("rank out of bounds");
        if deque.is_empty() {
            self.map.remove(&frame).unwrap();
        }
        flash
    }

    /// Modifies the frame number for a flash.
    pub fn set_frame(&mut self, frame: usize, rank: usize, new_frame: usize) {
        if frame == new_frame {
            return;
        }
        let flash = self.remove(frame, rank);
        self.map
            .entry(new_frame)
            .and_modify(|e| {
                if new_frame > frame {
                    e.push_front(flash)
                } else {
                    e.push_back(flash)
                }
            })
            .or_insert_with(|| [flash].into());
    }

    pub fn get_mut(&mut self, frame: usize, rank: usize) -> Option<&mut T> {
        self.map
            .get_mut(&frame)
            .and_then(|deque| deque.get_mut(rank))
    }
}

impl FlashMap<ColorFlash> {
    /// Determines what color the flash should be for a given frame number.
    fn compute(&self, frame: usize) -> luminol_data::Color {
        let Some((&start_frame, deque)) = self.map.range(..=frame).next_back() else {
            return luminol_data::Color {
                red: 255.,
                green: 255.,
                blue: 255.,
                alpha: 0.,
            };
        };
        let flash = deque.back().unwrap();

        let diff = frame - start_frame;
        if diff < flash.duration {
            let progression = diff as f64 / flash.duration as f64;
            luminol_data::Color {
                alpha: flash.color.alpha * (1. - progression),
                ..flash.color
            }
        } else {
            luminol_data::Color {
                red: 255.,
                green: 255.,
                blue: 255.,
                alpha: 0.,
            }
        }
    }
}

impl FlashMap<HideFlash> {
    /// Determines if the hide flash is active for a given frame number.
    fn compute(&self, frame: usize) -> bool {
        let Some((&start_frame, deque)) = self.map.range(..=frame).next_back() else {
            return false;
        };
        let flash = deque.back().unwrap();

        let diff = frame - start_frame;
        diff < flash.duration
    }
}

pub fn log_battler_error(
    update_state: &mut luminol_core::UpdateState<'_>,
    system: &luminol_data::rpg::System,
    animation: &luminol_data::rpg::Animation,
    e: color_eyre::Report,
) {
    luminol_core::error!(
        update_state.toasts,
        e.wrap_err(format!(
            "While loading texture {:?} for animation {:0>4} {:?}",
            system.battler_name,
            animation.id + 1,
            animation.name,
        )),
    );
}

pub fn log_atlas_error(
    update_state: &mut luminol_core::UpdateState<'_>,
    animation: &luminol_data::rpg::Animation,
    e: color_eyre::Report,
) {
    luminol_core::error!(
        update_state.toasts,
        e.wrap_err(format!(
            "While loading texture {:?} for animation {:0>4} {:?}",
            animation.animation_name,
            animation.id + 1,
            animation.name,
        )),
    );
}

pub fn load_se(
    update_state: &mut luminol_core::UpdateState<'_>,
    animation_state: &mut super::AnimationState,
    condition: Condition,
    timing: &luminol_data::rpg::animation::Timing,
) {
    let Some(se_name) = &timing.se.name else {
        return;
    };
    if (condition != timing.condition
        && condition != Condition::None
        && timing.condition != Condition::None)
        || animation_state.audio_data.contains_key(se_name.as_str())
    {
        return;
    }
    match update_state.filesystem.read(format!("Audio/SE/{se_name}")) {
        Ok(data) => {
            animation_state
                .audio_data
                .insert(se_name.to_string(), Some(data.into()));
        }
        Err(e) => {
            luminol_core::error!(
                update_state.toasts,
                e.wrap_err(format!("Error loading animation sound effect {se_name}"))
            );
            animation_state.audio_data.insert(se_name.to_string(), None);
        }
    }
}

pub fn resize_frame(frame: &mut luminol_data::rpg::animation::Frame, new_cell_max: usize) {
    let old_capacity = frame.cell_data.xsize();
    let new_capacity = if new_cell_max == 0 {
        0
    } else {
        new_cell_max.next_power_of_two()
    };

    // Instead of resizing `frame.cell_data` every time we call this function, we increase the
    // size of `frame.cell_data` only it's too small and we decrease the size of
    // `frame.cell_data` only if it's at <= 25% capacity for better efficiency
    let capacity_too_low = old_capacity < new_capacity;
    let capacity_too_high = old_capacity >= new_capacity * 4;

    if capacity_too_low {
        frame.cell_data.resize(new_capacity, 8);
        for i in old_capacity..new_capacity {
            frame.cell_data[(i, 0)] = -1;
            frame.cell_data[(i, 1)] = 0;
            frame.cell_data[(i, 2)] = 0;
            frame.cell_data[(i, 3)] = 100;
            frame.cell_data[(i, 4)] = 0;
            frame.cell_data[(i, 5)] = 0;
            frame.cell_data[(i, 6)] = 255;
            frame.cell_data[(i, 7)] = 1;
        }
    } else if capacity_too_high {
        frame.cell_data.resize(new_capacity * 2, 8);
    }

    frame.cell_max = new_cell_max;
}
