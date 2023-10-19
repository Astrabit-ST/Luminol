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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

pub mod tabs;

pub mod windows;

macro_rules! tab_enum {
    ($visibility:vis enum $name:ident {
        $( $variant:ident($variant_type:ty) ),* $(,)?
    }) => {
        $visibility enum $name {
            $(
                $variant($variant_type),
            )*
        }

        impl luminol_core::Tab for $name {
            fn name(&self) -> String {
                match self {
                    $(
                        Self::$variant(v) => v.name(),
                    )*
                }
            }

            fn id(&self) -> egui::Id {
                match self {
                    $(
                        Self::$variant(v) => v.id(),
                    )*
                }
            }

            fn show<W, T>(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_, W, T>)
            where
                W: luminol_core::Window,
                T: luminol_core::Tab
            {
                match self {
                    $(
                        Self::$variant(v) => v.show(ui, update_state),
                    )*
                }
            }

            fn requires_filesystem(&self) -> bool {
                match self {
                    $(
                        Self::$variant(v) => v.requires_filesystem(),
                    )*
                }
            }

            fn force_close(&mut self) -> bool {
                match self {
                    $(
                        Self::$variant(v) => v.force_close(),
                    )*
                }
            }
        }

        $(
            impl From<$variant_type> for $name {
                fn from(value: $variant_type) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

macro_rules! window_enum {
    ($visibility:vis enum $name:ident {
        $( $variant:ident($variant_type:ty) ),* $(,)?
    }) => {
        $visibility enum $name {
            $(
                $variant($variant_type),
            )*
        }

        impl luminol_core::Window for $name {
            fn show<W, T>(
                &mut self,
                ctx: &egui::Context,
                open: &mut bool,
                update_state: &mut luminol_core::UpdateState<'_, W, T>,
            ) where
                W: luminol_core::Window,
                T: luminol_core::Tab
            {
                match self {
                    $(
                        Self::$variant(v) => v.show(ctx, open, update_state),
                    )*
                }
            }

            fn name(&self) -> String {
                match self {
                    $(
                        Self::$variant(v) => v.name(),
                    )*
                }
            }

            fn id(&self) -> egui::Id {
                match self {
                    $(
                        Self::$variant(v) => v.id(),
                    )*
                }
            }

            fn requires_filesystem(&self) -> bool {
                match self {
                    $(
                        Self::$variant(v) => v.requires_filesystem(),
                    )*
                }
            }
        }

        $(
            impl From<$variant_type> for $name {
                fn from(value: $variant_type) -> Self {
                    Self::$variant(value)
                }
            }
        )*
    };
}

tab_enum! {
    pub enum Tab {
        Map(tabs::map::Tab),
        Started(tabs::started::Tab)
    }
}

window_enum! {
    pub enum Window {
        About(windows::about::Window),
        Appearance(windows::appearance::Window),
        CommonEvent(windows::common_event_edit::Window),
        ProjectConfig(windows::config_window::Window),
        Console(windows::console::Window),
        EventEdit(windows::event_edit::Window),
        GlobalConfig(windows::global_config_window::Window),
        Items(windows::items::Window),
        MapPicker(windows::map_picker::Window),
        EguiInspection(windows::misc::EguiInspection),
        EguiMemory(windows::misc::EguiMemory),
        FilesystemDebug(windows::misc::FilesystemDebug),
        NewProject(windows::new_project::Window),
        ScriptEdit(windows::script_edit::Window),
        SoundTest(windows::sound_test::Window)
    }
}
