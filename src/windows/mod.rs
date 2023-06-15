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
macro_rules! window_enum {
    ($visibility:vis enum $name:ident {
		$($variant_name:ident($variant_type:ty)),+
	}) => {
        $(
            static_assertions::assert_impl_all!($variant_type: WindowExt);
        )+

        $visibility enum $name {
            $(
                $variant_name($variant_type),
            )+
        }

        impl WindowExt for $name {
            fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
                match self {
                    $(
                        Self::$variant_name(w) => w.show(ctx, open),
                    )+
                }
            }

            fn name(&self) -> String {
                match self {
                    $(
                        Self::$variant_name(w) => w.name(),
                    )+
                }
            }

            fn id(&self) -> egui::Id {
                match self {
                    $(
                        Self::$variant_name(w) => w.id(),
                    )+
                }
            }

            fn requires_filesystem(&self) -> bool {
                match self {
                    $(
                        Self::$variant_name(w) => w.requires_filesystem(),
                    )+
                }
            }
        }

		$(
			impl From<$variant_type> for $name {
				fn from(value: $variant_type) -> Self {
					Self::$variant_name(value)
				}
			}
		)+
	};
}

window_enum! {
    pub enum Window {
        About(about::Window),
        CommandGeneratorWindow(command_gen::CommandGeneratorWindow),
        CommonEventEdit(common_event_edit::Window),
        Config(config::Window),
        Console(console::Console),
        EventEdit(event_edit::Window),
        Items(items::Window),
        MapPicker(map_picker::Window),
        NewProject(new_project::Window),
        ScriptEdit(script_edit::Window),
        SoundTest(sound_test::Window),
        Inspection(misc::EguiInspection),
        Memory(misc::EguiMemory)
    }
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
