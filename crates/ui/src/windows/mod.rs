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

/// The about window.
pub mod about;
/// The actor editor.
pub mod actors;
/// The archive manager for creating and extracting RGSSAD archives.
pub mod archive_manager;
/// The armor editor.
pub mod armor;
/// The class editor.
pub mod classes;
/// The common event editor.
pub mod common_event_edit;
/// Config window
pub mod config_window;
/// Playtest console
#[cfg(not(target_arch = "wasm32"))]
pub mod console;
/// The enemy editor.
pub mod enemies;
/// The event editor.
pub mod event_edit;
/// The item editor.
pub mod items;
/// The map picker.
pub mod map_picker;
/// Misc windows.
pub mod misc;
/// New project window
pub mod new_project;
pub mod preferences;
/// The crash reporter.
pub mod reporter;
/// The script editor
pub mod script_edit;
/// The script manager for creating and extracting Scripts.rxdata.
pub mod script_manager;
/// The skill editor.
pub mod skills;
/// The sound test.
pub mod sound_test;
/// The state editor.
pub mod states;
/// The weapon editor.
pub mod weapons;
