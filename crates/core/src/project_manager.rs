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

pub struct ProjectManager {
    pub(crate) modal: egui_modal::Modal,
    pub(crate) closure:
        Option<Box<dyn FnOnce(&mut crate::UpdateState<'_>, &mut luminol_eframe::Frame)>>,

    pub load_filesystem_promise: Option<poll_promise::Promise<FileSystemPromiseResult>>,
    pub filesystem_open_result: Option<FileSystemOpenResult>,
}

type FileSystemPromiseResult = luminol_filesystem::Result<luminol_filesystem::host::FileSystem>;
type FileSystemOpenResult = luminol_filesystem::Result<luminol_filesystem::project::LoadResult>;

impl ProjectManager {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            modal: egui_modal::Modal::new(ctx, "luminol_save_modal"),
            closure: None,
            load_filesystem_promise: None,
            filesystem_open_result: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Closes the application after asking the user to save unsaved changes.
    pub fn quit(&mut self) {
        self.closure = Some(Box::new(|update_state, frame| {
            // Disable the modified flag so `luminol_eframe::App::on_close_event` doesn't recurse
            update_state.modified.set(false);

            frame.close();
        }));
    }

    /// Opens a project picker after asking the user to save unsaved changes.
    pub fn open_project_picker(&mut self) {
        self.closure = Some(Box::new(|update_state, _frame| {
            // maybe worthwhile to make an extension trait to select spawn_async or spawn_local based on the target?
            #[cfg(not(target_arch = "wasm32"))]
            {
                update_state.project_manager.load_filesystem_promise =
                    Some(poll_promise::Promise::spawn_async(
                        luminol_filesystem::host::FileSystem::from_file_picker(),
                    ));
            }
            #[cfg(target_arch = "wasm32")]
            {
                update_state.project_manager.load_filesystem_promise =
                    Some(poll_promise::Promise::spawn_local(
                        luminol_filesystem::host::FileSystem::from_folder_picker(),
                    ));
            }
        }));
    }

    /// Opens a recent project after asking the user to save unsaved changes.
    ///
    /// On native, `key` should be the absolute path to the project folder.
    /// On web, `key` should be the IndexedDB key of the project folder.
    pub fn load_recent_project(&mut self, key: String) {
        self.closure = Some(Box::new(|update_state, _frame| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                update_state.project_manager.filesystem_open_result =
                    Some(update_state.filesystem.load_project_from_path(
                        update_state.project_config,
                        update_state.global_config,
                        key,
                    ));
            }

            #[cfg(target_arch = "wasm32")]
            {
                update_state.project_manager.load_filesystem_promise =
                    Some(poll_promise::Promise::spawn_local(
                        luminol_filesystem::host::FileSystem::from_idb_key(key),
                    ));
            }
        }));
    }

    pub fn close_project(&mut self) {
        self.closure = Some(Box::new(|update_state, _frame| {
            update_state
                .edit_windows
                .clean(|w| !w.requires_filesystem());
            update_state.edit_tabs.clean(|t| !t.requires_filesystem());
            update_state.audio.clear_sinks(); // audio loads files borrows from the filesystem. unloading while they are playing is a crash
            update_state.filesystem.unload_project();
            *update_state.project_config = None;
            update_state.data.unload();
            update_state.modified.set(false);
        }))
    }
}
