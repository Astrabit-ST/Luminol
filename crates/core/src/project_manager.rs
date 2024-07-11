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

pub struct ProjectManager {
    pub(crate) modal: egui_modal::Modal,
    pub(crate) closure: Option<Box<ProjectManagerClosure>>,

    pub create_project_promise: Option<poll_promise::Promise<CreateProjectPromiseResult>>,
    pub load_filesystem_promise: Option<poll_promise::Promise<FileSystemPromiseResult>>,
    pub filesystem_open_result: Option<FileSystemOpenResult>,
}

pub struct CreateProjectResult {
    pub data_cache: crate::Data,
    pub config: luminol_config::project::Config,
    pub host_fs: luminol_filesystem::host::FileSystem,
}

type ProjectManagerClosure = dyn FnOnce(&mut crate::UpdateState<'_>);
pub type CreateProjectPromiseResult = color_eyre::Result<CreateProjectResult>;
pub type FileSystemPromiseResult = luminol_filesystem::Result<luminol_filesystem::host::FileSystem>;
pub type FileSystemOpenResult = luminol_filesystem::Result<luminol_filesystem::project::LoadResult>;

#[cfg(not(target_arch = "wasm32"))]
/// Spawns a future using `poll_promise::Promise::spawn_async` on native or
/// `poll_promise::Promise::spawn_local` on web.
pub fn spawn_future<T: Send>(
    future: impl std::future::Future<Output = T> + Send + 'static,
) -> poll_promise::Promise<T> {
    poll_promise::Promise::spawn_async(future)
}

#[cfg(target_arch = "wasm32")]
/// Spawns a future using `poll_promise::Promise::spawn_async` on native or
/// `poll_promise::Promise::spawn_local` on web.
pub fn spawn_future<T: Send>(
    future: impl std::future::Future<Output = T> + 'static,
) -> poll_promise::Promise<T> {
    poll_promise::Promise::spawn_local(future)
}

impl ProjectManager {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            modal: egui_modal::Modal::new(ctx, "luminol_save_modal"),
            closure: None,
            create_project_promise: None,
            load_filesystem_promise: None,
            filesystem_open_result: None,
        }
    }

    /// Returns whether or not the unsaved changes modal is currently open.
    pub fn is_modal_open(&self) -> bool {
        self.modal.is_open()
    }

    /// Returns whether or not a file or filder picker is currently open.
    pub fn is_picker_open(&self) -> bool {
        self.filesystem_open_result.is_some()
            || self.create_project_promise.is_some()
            || self.load_filesystem_promise.is_some()
    }

    /// Runs a closure after asking the user to save unsaved changes.
    pub fn run_custom(&mut self, closure: impl FnOnce(&mut crate::UpdateState<'_>) + 'static) {
        self.closure = Some(Box::new(closure));
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Closes the application after asking the user to save unsaved changes.
    pub fn quit(&mut self) {
        self.run_custom(|update_state| {
            // Disable the modified flag so `luminol_eframe::App::on_close_event` doesn't recurse
            update_state.modified.set(false);

            update_state
                .ctx
                .send_viewport_cmd(egui::ViewportCommand::Close);
        });
    }

    /// Opens a project picker after asking the user to save unsaved changes.
    pub fn open_project_picker(&mut self) {
        self.run_custom(|update_state| {
            #[cfg(not(target_arch = "wasm32"))]
            let promise = spawn_future(luminol_filesystem::host::FileSystem::from_file_picker());
            #[cfg(target_arch = "wasm32")]
            let promise = spawn_future(luminol_filesystem::host::FileSystem::from_folder_picker());

            update_state.project_manager.load_filesystem_promise = Some(promise);
        });
    }

    /// Opens a recent project after asking the user to save unsaved changes.
    ///
    /// On native, `key` should be the absolute path to the project folder.
    /// On web, `key` should be the IndexedDB key of the project folder.
    pub fn load_recent_project(&mut self, key: String) {
        self.run_custom(|update_state| {
            #[cfg(not(target_arch = "wasm32"))]
            {
                update_state.close_project();
                update_state.project_manager.filesystem_open_result =
                    Some(update_state.filesystem.load_project_from_path(
                        update_state.project_config,
                        update_state.global_config,
                        key,
                    ));
            }

            #[cfg(target_arch = "wasm32")]
            {
                update_state.project_manager.load_filesystem_promise = Some(spawn_future(
                    luminol_filesystem::host::FileSystem::from_idb_key(key),
                ));
            }
        });
    }

    /// Closes the current project after asking the user to save unsaved changes.
    pub fn close_project(&mut self) {
        self.run_custom(|update_state| {
            update_state.close_project();
        });
    }
}
