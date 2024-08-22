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

use crate::components::CollapsingView;

mod frame_edit;
mod timing;
mod util;
mod window;

const HISTORY_SIZE: usize = 50;

/// Database - Animations management window.
pub struct Window {
    selected_animation_name: Option<String>,
    previous_animation: Option<usize>,
    previous_battler_name: Option<camino::Utf8PathBuf>,
    frame_edit_state: FrameEditState,
    timing_edit_state: TimingEditState,

    collapsing_view: crate::components::CollapsingView,
    modals: Modals,
    view: crate::components::DatabaseView,
}

struct FrameEditState {
    animation_fps: f64,
    frame_index: usize,
    condition: luminol_data::rpg::animation::Condition,
    enable_onion_skin: bool,
    frame_view: Option<crate::components::AnimationFrameView>,
    cellpicker: Option<crate::components::Cellpicker>,
    animation_graphic_picker: Option<crate::modals::graphic_picker::animation::Modal>,
    flash_maps: luminol_data::OptionVec<util::FlashMaps>,
    animation_state: Option<AnimationState>,
    saved_frame_index: Option<usize>,
    saved_selected_cell_index: Option<usize>,
    frame_needs_update: bool,
    history: History,
    drag_state: Option<DragState>,
}

#[derive(Debug)]
struct DragState {
    cell_index: usize,
    original_x: i16,
    original_y: i16,
}

#[derive(Debug)]
struct AnimationState {
    saved_frame_index: usize,
    start_time: f64,
    timing_index: usize,
    audio_data: std::collections::HashMap<String, Option<std::sync::Arc<[u8]>>>,
}

struct TimingEditState {
    previous_frame: Option<usize>,
    se_picker: crate::modals::sound_picker::Modal,
}

#[derive(Debug, Default)]
struct History(luminol_data::OptionVec<luminol_data::OptionVec<HistoryInner>>);

#[derive(Debug, Default)]
struct HistoryInner {
    undo: std::collections::VecDeque<Vec<HistoryEntry>>,
    redo: Vec<Vec<HistoryEntry>>,
}

impl History {
    fn inner(&mut self, animation_index: usize, frame_index: usize) -> &mut HistoryInner {
        if !self.0.contains(animation_index) {
            self.0.insert(animation_index, Default::default());
        }
        let map = self.0.get_mut(animation_index).unwrap();
        if !map.contains(frame_index) {
            map.insert(frame_index, Default::default());
        }
        map.get_mut(frame_index).unwrap()
    }

    fn remove_animation(&mut self, animation_index: usize) {
        let _ = self.0.try_remove(animation_index);
    }

    fn remove_frame(&mut self, animation_index: usize, frame_index: usize) {
        if let Some(map) = self.0.get_mut(animation_index) {
            let _ = map.try_remove(frame_index);
        }
    }

    fn push(&mut self, animation_index: usize, frame_index: usize, mut entries: Vec<HistoryEntry>) {
        entries.shrink_to_fit();
        let inner = self.inner(animation_index, frame_index);
        inner.redo.clear();
        while inner.undo.len() >= HISTORY_SIZE {
            inner.undo.pop_front();
        }
        inner.undo.push_back(entries);
    }

    fn undo(
        &mut self,
        animation_index: usize,
        frame_index: usize,
        frame: &mut luminol_data::rpg::animation::Frame,
    ) {
        let inner = self.inner(animation_index, frame_index);
        let Some(mut vec) = inner.undo.pop_back() else {
            return;
        };
        vec.reverse();
        for entry in vec.iter_mut() {
            entry.apply(frame);
        }
        inner.redo.push(vec);
    }

    fn redo(
        &mut self,
        animation_index: usize,
        frame_index: usize,
        frame: &mut luminol_data::rpg::animation::Frame,
    ) {
        let inner = self.inner(animation_index, frame_index);
        let Some(mut vec) = inner.redo.pop() else {
            return;
        };
        vec.reverse();
        for entry in vec.iter_mut() {
            entry.apply(frame);
        }
        inner.undo.push_back(vec);
    }
}

#[derive(Debug)]
enum HistoryEntry {
    Cell { index: usize, data: [i16; 8] },
    ResizeCells(usize),
}

impl HistoryEntry {
    fn new_cell(cell_data: &luminol_data::Table2, cell_index: usize) -> Self {
        let mut data = [0i16; 8];
        for i in 0..8 {
            data[i] = cell_data[(cell_index, i)];
        }
        Self::Cell {
            index: cell_index,
            data,
        }
    }

    fn apply(&mut self, frame: &mut luminol_data::rpg::animation::Frame) {
        match self {
            HistoryEntry::Cell { index, data } => {
                for (i, item) in data.iter_mut().enumerate() {
                    std::mem::swap(item, &mut frame.cell_data[(*index, i)]);
                }
            }
            HistoryEntry::ResizeCells(len) => {
                let old_len = frame.len();
                util::resize_frame(frame, *len);
                *len = old_len;
            }
        }
    }
}

struct Modals {
    copy_frames: crate::modals::animations::copy_frames_tool::Modal,
    clear_frames: crate::modals::animations::clear_frames_tool::Modal,
    tween: crate::modals::animations::tween_tool::Modal,
    batch_edit: crate::modals::animations::batch_edit_tool::Modal,
    change_frame_count: crate::modals::animations::change_frame_count_tool::Modal,
    change_cell_number: crate::modals::animations::change_cell_number_tool::Modal,
}

impl Modals {
    fn close_all(&mut self) {
        self.close_all_except_frame_count();
        self.change_frame_count.close_window();
    }

    fn close_all_except_frame_count(&mut self) {
        self.copy_frames.close_window();
        self.clear_frames.close_window();
        self.tween.close_window();
        self.batch_edit.close_window();
        self.change_cell_number.close_window();
    }
}

impl Default for Window {
    fn default() -> Self {
        Self {
            selected_animation_name: None,
            previous_animation: None,
            previous_battler_name: None,
            frame_edit_state: FrameEditState {
                animation_fps: 20.,
                frame_index: 0,
                condition: luminol_data::rpg::animation::Condition::Hit,
                enable_onion_skin: false,
                frame_view: None,
                cellpicker: None,
                animation_graphic_picker: None,
                flash_maps: Default::default(),
                animation_state: None,
                saved_frame_index: None,
                saved_selected_cell_index: None,
                frame_needs_update: false,
                drag_state: None,
                history: Default::default(),
            },
            timing_edit_state: TimingEditState {
                previous_frame: None,
                se_picker: crate::modals::sound_picker::Modal::new(
                    luminol_audio::Source::SE,
                    "animations_timing_se_picker",
                ),
            },
            collapsing_view: CollapsingView::new(),
            modals: Modals {
                copy_frames: crate::modals::animations::copy_frames_tool::Modal::new(
                    "animations_copy_frames_tool",
                ),
                clear_frames: crate::modals::animations::clear_frames_tool::Modal::new(
                    "animations_clear_frames_tool",
                ),
                tween: crate::modals::animations::tween_tool::Modal::new("animations_tween_tool"),
                batch_edit: crate::modals::animations::batch_edit_tool::Modal::new(
                    "animations_batch_edit_tool",
                ),
                change_frame_count: crate::modals::animations::change_frame_count_tool::Modal::new(
                    "change_frame_count_tool",
                ),
                change_cell_number: crate::modals::animations::change_cell_number_tool::Modal::new(
                    "change_cell_number_tool",
                ),
            },
            view: crate::components::DatabaseView::new(),
        }
    }
}
