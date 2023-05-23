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
macro_rules! impl_window_for_enum {
	($enum:ty, $($variant:ident),+) => {
		impl WindowExt for $enum {
			fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
				$(
					if let Self::$variant(window) = self {
						window.show(ctx, open);
						return;
					}
				)*
				unreachable!();
			}

			fn name(&self) -> String {
				$(
					if let Self::$variant(window) = self {
						return window.name();
					}
				)*
				unreachable!();
			}

			fn id(&self) -> egui::Id {
				$(
					if let Self::$variant(window) = self {
						return window.id();
					}
				)*
				unreachable!();
			}

			fn requires_filesystem(&self) -> bool {
				$(
					if let Self::$variant(window) = self {
						return window.requires_filesystem();
					}
				)*
				unreachable!();
			}
		}
	};
}

pub enum EguiWindow {
    Inspection(misc::EguiInspection),
    Memory(misc::EguiMemory),
}
impl_window_for_enum! {EguiWindow, Inspection, Memory}

pub enum Window<'win> {
    About(about::Window),
    CommandGeneratorWindow(command_gen::CommandGeneratorWindow),
    CommonEventEdit(common_event_edit::Window),
    Config(config::Window),
    Console(console::Console),
    EventEdit(event_edit::Window),
    GraphicPicker(graphic_picker::Window<'win>),
    Items(items::Window<'win>),
    MapPicker(map_picker::Window),
    NewProject(new_project::Window),
    ScriptEdit(script_edit::Window),
    SoundTest(sound_test::Window),
    Egui(EguiWindow),
}
impl_window_for_enum! {
    Window<'_>,
    About,
    CommandGeneratorWindow,
    CommonEventEdit,
    Config,
    Console,
    EventEdit,
    GraphicPicker,
    Items,
    MapPicker,
    NewProject,
    ScriptEdit,
    SoundTest,
    Egui
}

/// The about window.
pub mod about;
/// The common event editor.
pub mod common_event_edit;
/// Config window
pub mod config;
/// Playtest console
pub mod console;
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

pub use window::WindowExt;

use crate::command_gen;
