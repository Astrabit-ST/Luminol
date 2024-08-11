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

use luminol_data::rpg::animation::{Condition, Scope, Timing};

#[derive(Debug, Default)]
pub struct FlashMaps {
    none_hide: FlashMap<HideFlash>,
    hit_hide: FlashMap<HideFlash>,
    miss_hide: FlashMap<HideFlash>,
    none_target: FlashMap<ColorFlash>,
    hit_target: FlashMap<ColorFlash>,
    miss_target: FlashMap<ColorFlash>,
    none_screen: FlashMap<ColorFlash>,
    hit_screen: FlashMap<ColorFlash>,
    miss_screen: FlashMap<ColorFlash>,
}

impl FlashMaps {
    pub fn new(timings: &[Timing]) -> Self {
        Self {
            none_hide: <FlashMap<HideFlash>>::new(timings, Condition::None, Scope::HideTarget),
            hit_hide: <FlashMap<HideFlash>>::new(timings, Condition::Hit, Scope::HideTarget),
            miss_hide: <FlashMap<HideFlash>>::new(timings, Condition::Miss, Scope::HideTarget),
            none_target: <FlashMap<ColorFlash>>::new(timings, Condition::None, Scope::Target),
            hit_target: <FlashMap<ColorFlash>>::new(timings, Condition::Hit, Scope::Target),
            miss_target: <FlashMap<ColorFlash>>::new(timings, Condition::Miss, Scope::Target),
            none_screen: <FlashMap<ColorFlash>>::new(timings, Condition::None, Scope::Screen),
            hit_screen: <FlashMap<ColorFlash>>::new(timings, Condition::Hit, Scope::Screen),
            miss_screen: <FlashMap<ColorFlash>>::new(timings, Condition::Miss, Scope::Screen),
        }
    }

    pub fn target(&self, condition: Condition) -> &FlashMap<ColorFlash> {
        match condition {
            Condition::None => &self.none_target,
            Condition::Hit => &self.hit_target,
            Condition::Miss => &self.miss_target,
        }
    }

    pub fn target_mut(&mut self, condition: Condition) -> &mut FlashMap<ColorFlash> {
        match condition {
            Condition::None => &mut self.none_target,
            Condition::Hit => &mut self.hit_target,
            Condition::Miss => &mut self.miss_target,
        }
    }

    pub fn screen(&self, condition: Condition) -> &FlashMap<ColorFlash> {
        match condition {
            Condition::None => &self.none_screen,
            Condition::Hit => &self.hit_screen,
            Condition::Miss => &self.miss_screen,
        }
    }

    pub fn screen_mut(&mut self, condition: Condition) -> &mut FlashMap<ColorFlash> {
        match condition {
            Condition::None => &mut self.none_screen,
            Condition::Hit => &mut self.hit_screen,
            Condition::Miss => &mut self.miss_screen,
        }
    }

    pub fn hide(&self, condition: Condition) -> &FlashMap<HideFlash> {
        match condition {
            Condition::None => &self.none_hide,
            Condition::Hit => &self.hit_hide,
            Condition::Miss => &self.miss_hide,
        }
    }

    pub fn hide_mut(&mut self, condition: Condition) -> &mut FlashMap<HideFlash> {
        match condition {
            Condition::None => &mut self.none_hide,
            Condition::Hit => &mut self.hit_hide,
            Condition::Miss => &mut self.miss_hide,
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

impl<'a> From<&'a Timing> for ColorFlash {
    fn from(timing: &'a Timing) -> Self {
        Self {
            color: timing.flash_color,
            duration: timing.flash_duration,
        }
    }
}

impl<'a> From<&'a mut Timing> for ColorFlash {
    fn from(timing: &'a mut Timing) -> Self {
        (&*timing).into()
    }
}

impl From<Timing> for ColorFlash {
    fn from(timing: Timing) -> Self {
        (&timing).into()
    }
}

impl<'a> From<&'a Timing> for HideFlash {
    fn from(timing: &'a Timing) -> Self {
        Self {
            duration: timing.flash_duration,
        }
    }
}

impl<'a> From<&'a mut Timing> for HideFlash {
    fn from(timing: &'a mut Timing) -> Self {
        (&*timing).into()
    }
}

impl From<Timing> for HideFlash {
    fn from(timing: Timing) -> Self {
        (&timing).into()
    }
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
    fn new(timings: &[Timing], condition: Condition, scope: Scope) -> Self {
        timings
            .iter()
            .filter(|timing| timing.flash_scope == scope && filter_timing(timing, condition))
            .map(|timing| (timing.frame, timing.into()))
            .collect()
    }

    /// Determines what color the flash should be for a given frame number.
    pub fn compute(&self, frame: usize) -> luminol_data::Color {
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
    fn new(timings: &[Timing], condition: Condition, scope: Scope) -> Self {
        timings
            .iter()
            .filter(|timing| timing.flash_scope == scope && filter_timing(timing, condition))
            .map(|timing| (timing.frame, timing.into()))
            .collect()
    }

    /// Determines if the hide flash is active for a given frame number.
    pub fn compute(&self, frame: usize) -> bool {
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

/// If the given timing has a sound effect and the given timing should be shown based on the given
/// condition, caches the audio data for that sound effect into `animation_state.audio_data`.
pub fn load_se(
    update_state: &mut luminol_core::UpdateState<'_>,
    animation_state: &mut super::AnimationState,
    condition: Condition,
    timing: &luminol_data::rpg::animation::Timing,
) {
    // Do nothing if this timing has no sound effect
    let Some(se_name) = &timing.se.name else {
        return;
    };

    // Do nothing if the timing shouldn't be shown based on the condition currently selected in the
    // UI or if the timing's sound effect has already been loaded
    if !filter_timing(timing, condition)
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
        frame
            .cell_data
            .resize(new_capacity, frame.cell_data.ysize().max(8));
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
        frame
            .cell_data
            .resize(new_capacity * 2, frame.cell_data.ysize().max(8));
    }

    frame.cell_max = new_cell_max;
}

/// Determines whether or not a timing should be used based on the given condition.
pub fn filter_timing(timing: &Timing, condition: Condition) -> bool {
    match condition {
        Condition::None => true,
        Condition::Hit => timing.condition != Condition::Miss,
        Condition::Miss => timing.condition != Condition::Hit,
    }
}

/// Helper function for updating `FlashMaps` when a flash is updated. Given the condition of a
/// flash, this calls the given closure once for the condition of each flash map that must be
/// updated.
pub fn update_flash_maps(condition: Condition, mut closure: impl FnMut(Condition)) {
    closure(Condition::None);
    if condition != Condition::Miss {
        closure(Condition::Hit);
    }
    if condition != Condition::Hit {
        closure(Condition::Miss);
    }
}

/// Gets mutable references at two different indices of a slice. Panics if the two indices are the
/// same.
pub fn get_two_mut<T>(slice: &mut [T], index1: usize, index2: usize) -> (&mut T, &mut T) {
    if index1 >= slice.len() {
        panic!("index1 out of range");
    }
    if index2 >= slice.len() {
        panic!("index2 out of range");
    }
    if index1 == index2 {
        panic!("index1 and index2 are the same");
    }
    let slice = &mut slice[if index1 < index2 {
        index1..=index2
    } else {
        index2..=index1
    }];
    let (min, slice) = slice.split_first_mut().unwrap();
    let max = slice.last_mut().unwrap();
    if index1 < index2 {
        (min, max)
    } else {
        (max, min)
    }
}

/// Computes the list of history entries necessary to undo the transformation from `old_frame` to
/// `new_frame`.
pub fn history_entries_from_two_tables(
    old_frame: &luminol_data::rpg::animation::Frame,
    new_frame: &luminol_data::rpg::animation::Frame,
) -> Vec<super::HistoryEntry> {
    let cell_iter = (0..old_frame.len())
        .filter(|&i| {
            (0..8).any(|j| {
                i >= new_frame.len() || new_frame.cell_data[(i, j)] != old_frame.cell_data[(i, j)]
            })
        })
        .map(|i| super::HistoryEntry::new_cell(&old_frame.cell_data, i));
    let resize_iter = std::iter::once_with(|| super::HistoryEntry::ResizeCells(old_frame.len()));
    match new_frame.len().cmp(&old_frame.len()) {
        std::cmp::Ordering::Equal => cell_iter.collect(),
        std::cmp::Ordering::Less => cell_iter.chain(resize_iter).collect(),
        std::cmp::Ordering::Greater => resize_iter
            .chain(cell_iter)
            .chain(
                (old_frame.len()..new_frame.len()).map(|i| super::HistoryEntry::Cell {
                    index: i,
                    data: [-1, 0, 0, 0, 0, 0, 0, 0],
                }),
            )
            .collect(),
    }
}
