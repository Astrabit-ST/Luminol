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

mod frame_edit;
mod timing;
mod util;
mod window;

/// Database - Animations management window.
pub struct Window {
    selected_animation_name: Option<String>,
    previous_animation: Option<usize>,
    previous_battler_name: Option<camino::Utf8PathBuf>,
    frame_edit_state: FrameEditState,
    timing_edit_state: TimingEditState,

    collapsing_view: luminol_components::CollapsingView,
    modals: Modals,
    view: luminol_components::DatabaseView,
}

struct FrameEditState {
    animation_fps: f64,
    frame_index: usize,
    condition: luminol_data::rpg::animation::Condition,
    enable_onion_skin: bool,
    frame_view: Option<luminol_components::AnimationFrameView>,
    cellpicker: Option<luminol_components::Cellpicker>,
    animation_graphic_picker: Option<luminol_modals::graphic_picker::animation::Modal>,
    flash_maps: luminol_data::OptionVec<util::FlashMaps>,
    animation_state: Option<AnimationState>,
    saved_frame_index: Option<usize>,
    saved_selected_cell_index: Option<usize>,
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
    se_picker: luminol_modals::sound_picker::Modal,
}

struct Modals {
    copy_frames: luminol_modals::animations::copy_frames_tool::Modal,
    clear_frames: luminol_modals::animations::clear_frames_tool::Modal,
    tween: luminol_modals::animations::tween_tool::Modal,
    batch_edit: luminol_modals::animations::batch_edit_tool::Modal,
}

impl Modals {
    fn close_all(&mut self) {
        self.copy_frames.close_window();
        self.clear_frames.close_window();
        self.tween.close_window();
        self.batch_edit.close_window();
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
            },
            timing_edit_state: TimingEditState {
                previous_frame: None,
                se_picker: luminol_modals::sound_picker::Modal::new(
                    luminol_audio::Source::SE,
                    "animations_timing_se_picker",
                ),
            },
            collapsing_view: luminol_components::CollapsingView::new(),
            modals: Modals {
                copy_frames: luminol_modals::animations::copy_frames_tool::Modal::new(
                    "animations_copy_frames_tool",
                ),
                clear_frames: luminol_modals::animations::clear_frames_tool::Modal::new(
                    "animations_clear_frames_tool",
                ),
                tween: luminol_modals::animations::tween_tool::Modal::new("animations_tween_tool"),
                batch_edit: luminol_modals::animations::batch_edit_tool::Modal::new(
                    "animations_batch_edit_tool",
                ),
            },
            view: luminol_components::DatabaseView::new(),
        }
    }
}
