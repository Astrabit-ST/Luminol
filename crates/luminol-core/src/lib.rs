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

use std::sync::Arc;

mod tab;
pub use tab::{Tab, Tabs};

mod window;
pub use window::{Window, Windows};

pub mod modal;
pub use modal::Modal;

/// Toasts to be displayed for errors, information, etc.
mod toasts;
pub use toasts::Toasts;

pub struct UpdateState<'res, W, T> {
    pub audio: &'res mut luminol_audio::Audio,

    pub graphics: Arc<luminol_graphics::GraphicsState>,
    pub filesystem: &'res mut luminol_filesystem::project::FileSystem, // FIXME: this is probably wrong
    pub data: &'res luminol_data::data_cache::Cache, // FIXME: this is also probably wrong

    // TODO: look into std::any?
    // we're using generics here to allow specialization on the type of window
    // this is fucntionality not really used atm but maybe in the future..?
    pub edit_windows: &'res mut window::EditWindows<W>,
    pub edit_tabs: &'res mut tab::EditTabs<T>,
    pub toasts: &'res mut toasts::Toasts,

    pub project_config: &'res mut Option<luminol_config::project::Config>,
    pub global_config: &'res mut luminol_config::global::Config,

    pub toolbar: &'res mut ToolbarState,
}

#[allow(missing_docs)]
#[derive(Default)]
pub struct ToolbarState {
    /// The currently selected pencil.
    pub pencil: Pencil,
}

#[derive(Default, strum::EnumIter, strum::Display, PartialEq, Eq, Clone, Copy)]
#[allow(missing_docs)]
pub enum Pencil {
    #[default]
    Pen,
    Circle,
    Rectangle,
    Fill,
}

impl<'res, W, T> UpdateState<'res, W, T> {
    pub(crate) fn reborrow_with_edit_window<'this, O>(
        &'this mut self,
        edit_windows: &'this mut window::EditWindows<O>,
    ) -> UpdateState<'this, O, T> {
        UpdateState {
            audio: &mut *self.audio,
            graphics: self.graphics.clone(),
            filesystem: &mut *self.filesystem,
            data: self.data,
            edit_tabs: &mut *self.edit_tabs,
            edit_windows,
            toasts: &mut *self.toasts,
            project_config: &mut *self.project_config,
            global_config: &mut *self.global_config,
            toolbar: &mut *self.toolbar,
        }
    }

    pub(crate) fn reborrow_with_edit_tabs<'this, O>(
        &'this mut self,
        edit_tabs: &'this mut tab::EditTabs<O>,
    ) -> UpdateState<'this, W, O> {
        UpdateState {
            audio: &mut *self.audio,
            graphics: self.graphics.clone(),
            filesystem: &mut *self.filesystem,
            data: self.data,
            edit_tabs,
            edit_windows: &mut *self.edit_windows,
            toasts: &mut *self.toasts,
            project_config: &mut *self.project_config,
            global_config: &mut *self.global_config,
            toolbar: &mut *self.toolbar,
        }
    }
}
