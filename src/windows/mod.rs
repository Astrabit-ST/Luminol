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
/// The about window.
pub mod about;
/// The common event editor.
pub mod common_event_edit;
/// Config window
pub mod config;
/// The event editor.
pub mod event_edit;
/// The Graphic picker.
pub mod graphic_picker;
/// The item editor.
pub mod items;
/// The map picker.
pub mod map_picker;
/// Misc windows.
pub mod misc;
/// New project window
pub mod new_project;
/// The script editor
pub mod script_edit;
/// The sound test.
pub mod sound_test;
/// Traits and structs related to windows.
pub mod window;

pub use window::Window;
