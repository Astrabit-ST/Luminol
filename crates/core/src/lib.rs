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
#![feature(trait_alias)]

use std::sync::Arc;

mod tab;
pub use tab::{EditTabs, Tab, Tabs};

mod window;
pub use window::{EditWindows, Window, Windows};

pub mod modal;
pub use modal::Modal;

mod data_cache;
pub use data_cache::Data;

/// Toasts to be displayed for errors, information, etc.
mod toasts;
pub use toasts::Toasts;

pub mod project_manager;
pub use project_manager::ProjectManager;

pub struct UpdateState<'res> {
    #[cfg(not(target_arch = "wasm32"))]
    pub audio: &'res mut luminol_audio::Audio,
    #[cfg(target_arch = "wasm32")]
    pub audio: &'res mut luminol_audio::AudioWrapper,

    pub graphics: Arc<luminol_graphics::GraphicsState>,
    pub filesystem: &'res mut luminol_filesystem::project::FileSystem, // FIXME: this is probably wrong
    pub data: &'res mut Data, // FIXME: this is also probably wrong
    pub bytes_loader: Arc<luminol_filesystem::egui_bytes_loader::Loader>,

    // TODO: look into std::any?
    // we're using generics here to allow specialization on the type of window
    // this is fucntionality not really used atm but maybe in the future..?
    pub edit_windows: &'res mut EditWindows,
    pub edit_tabs: &'res mut EditTabs,
    pub toasts: &'res mut Toasts,

    pub project_config: &'res mut Option<luminol_config::project::Config>,
    pub global_config: &'res mut luminol_config::global::Config,

    pub toolbar: &'res mut ToolbarState,

    pub modified: ModifiedState,
    pub project_manager: &'res mut ProjectManager,
}

/// This stores whether or not there are unsaved changes in any file in the current project and is
/// used to determine whether we should show a "you have unsaved changes" modal when the user tries
/// to close the current project or the application window.
///
/// This must be thread-safe in wasm because the `beforeunload` event handler resides on the main
/// thread but state is written to from the worker thread.
#[derive(Debug, Default, Clone)]
pub struct ModifiedState {
    #[cfg(not(target_arch = "wasm32"))]
    modified: std::rc::Rc<std::cell::Cell<bool>>,
    #[cfg(target_arch = "wasm32")]
    modified: Arc<std::sync::atomic::AtomicBool>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ModifiedState {
    pub fn get(&self) -> bool {
        self.modified.get()
    }

    pub fn set(&self, val: bool) {
        self.modified.set(val);
    }
}

#[cfg(target_arch = "wasm32")]
impl ModifiedState {
    pub fn get(&self) -> bool {
        self.modified.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn set(&self, val: bool) {
        self.modified
            .store(val, std::sync::atomic::Ordering::Relaxed);
    }
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

impl<'res> UpdateState<'res> {
    pub(crate) fn reborrow_with_edit_window<'this>(
        &'this mut self,
        edit_windows: &'this mut window::EditWindows,
    ) -> UpdateState<'this> {
        UpdateState {
            audio: self.audio,
            graphics: self.graphics.clone(),
            filesystem: self.filesystem,
            data: self.data,
            bytes_loader: self.bytes_loader.clone(),
            edit_tabs: self.edit_tabs,
            edit_windows,
            toasts: self.toasts,
            project_config: self.project_config,
            global_config: self.global_config,
            toolbar: self.toolbar,
            modified: self.modified.clone(),
            project_manager: self.project_manager,
        }
    }

    pub(crate) fn reborrow_with_edit_tabs<'this>(
        &'this mut self,
        edit_tabs: &'this mut tab::EditTabs,
    ) -> UpdateState<'this> {
        UpdateState {
            audio: self.audio,
            graphics: self.graphics.clone(),
            filesystem: self.filesystem,
            data: self.data,
            bytes_loader: self.bytes_loader.clone(),
            edit_tabs,
            edit_windows: self.edit_windows,
            toasts: self.toasts,
            project_config: self.project_config,
            global_config: self.global_config,
            toolbar: self.toolbar,
            modified: self.modified.clone(),
            project_manager: self.project_manager,
        }
    }

    pub fn manage_projects(&mut self, frame: &mut luminol_eframe::Frame, show_modal: bool) {
        let mut should_close = false;
        let mut should_save = false;
        let mut should_run_closure = false;
        let mut should_focus_save_button = false;

        if self.project_manager.closure.is_some() {
            if !self.modified.get() {
                should_close = true;
                should_run_closure = true;
            } else if show_modal && !self.project_manager.modal.is_open() {
                self.project_manager.modal.open();
                should_focus_save_button = true;
            }
        }

        if show_modal {
            self.project_manager.modal.show(|ui| {
                self.project_manager.modal.title(ui, "Unsaved Changes");
                self.project_manager.modal.frame(ui, |ui| {
                    self.project_manager
                        .modal
                        .body(ui, "Do you want to save your changes to this project?");
                });

                self.project_manager.modal.buttons(ui, |ui| {
                    let cancel_button = self.project_manager.modal.button(ui, "Cancel");
                    let discard_button = self.project_manager.modal.caution_button(ui, "Discard");
                    let save_button = self.project_manager.modal.suggested_button(ui, "Save");

                    if cancel_button.clicked() {
                        should_close = true;
                    } else if discard_button.clicked() {
                        should_close = true;
                        should_run_closure = true;
                    } else if save_button.clicked() {
                        should_close = true;
                        should_save = true;
                        should_run_closure = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        should_close = true;
                        self.project_manager.modal.close();
                    }

                    if should_focus_save_button {
                        save_button.request_focus();
                    }
                });
            });
        }

        if should_close {
            if should_save {
                if let Err(_err) = self
                    .data
                    .save(self.filesystem, self.project_config.as_ref().unwrap())
                {
                    todo!()
                }
                self.modified.set(false);
            }

            if should_run_closure {
                if let Some(closure) = self.project_manager.closure.take() {
                    closure(self, frame);
                }
            }

            self.project_manager.closure = None;
        }

        self.handle_project_loading();
    }

    fn handle_project_loading(&mut self) {
        let mut filesystem_open_result = None;
        #[cfg(target_arch = "wasm32")]
        let mut idb_key = None;

        if let Some(p) = self.project_manager.load_filesystem_promise.take() {
            match p.try_take() {
                Ok(Ok(host)) => {
                    self.close_project();

                    #[cfg(target_arch = "wasm32")]
                    {
                        idb_key = host.idb_key().map(str::to_string);
                    }

                    filesystem_open_result = Some(self.filesystem.load_project(
                        host,
                        self.project_config,
                        self.global_config,
                    ));
                }
                Ok(Err(error)) => self.toasts.error(error.to_string()),
                Err(p) => self.project_manager.load_filesystem_promise = Some(p),
            }
        }

        if let Some(r) = self.project_manager.filesystem_open_result.take() {
            filesystem_open_result = Some(r);
        }

        match filesystem_open_result {
            Some(Ok(load_result)) => {
                for missing_rtp in load_result.missing_rtps {
                    self.toasts.warning(format!(
                        "Failed to find suitable path for the RTP {missing_rtp}"
                    ));
                    // FIXME we should probably load rtps from the RTP/<rtp> paths on non-wasm targets
                    #[cfg(not(target_arch = "wasm32"))]
                    self.toasts
                        .info(format!("You may want to set an RTP path for {missing_rtp}"));
                    #[cfg(target_arch = "wasm32")]
                    self
                        .toasts
                        .info(format!("Please place the {missing_rtp} RTP in the 'RTP/{missing_rtp}' subdirectory in your project directory"));
                }

                if let Err(why) = self.data.load(
                    self.filesystem,
                    // TODO code jank
                    self.project_config.as_mut().unwrap(),
                ) {
                    self.toasts
                        .error(format!("Error loading the project data: {why}"));

                    #[cfg(target_arch = "wasm32")]
                    idb_key.map(luminol_filesystem::host::FileSystem::idb_drop);
                } else {
                    self.toasts.info(format!(
                        "Successfully opened {:?}",
                        self.filesystem.project_path().expect("project not open")
                    ));
                }
            }
            Some(Err(why)) => {
                self.toasts
                    .error(format!("Error opening the project: {why}"));

                #[cfg(target_arch = "wasm32")]
                idb_key.map(luminol_filesystem::host::FileSystem::idb_drop);
            }
            None => {}
        }

        if let Some(p) = self.project_manager.create_project_promise.take() {
            match p.try_take() {
                Ok(Ok(project_manager::CreateProjectResult {
                    data_cache,
                    config,
                    host_fs,
                })) => {
                    let result = self.filesystem.load_partially_loaded_project(
                        host_fs,
                        &config,
                        self.global_config,
                    );

                    match result {
                        Ok(_) => {
                            self.close_project();
                            *self.data = data_cache;
                            self.project_config.replace(config);
                        }
                        Err(error) => self.toasts.error(format!("{error:#}")),
                    }
                }
                Ok(Err(error)) => self.toasts.error(format!("{error:#}")),
                Err(p) => self.project_manager.create_project_promise = Some(p),
            }
        }
    }

    fn close_project(&mut self) {
        self.edit_windows.clean(|w| !w.requires_filesystem());
        self.edit_tabs.clean(|t| !t.requires_filesystem());
        self.audio.clear_sinks(); // audio loads files borrows from the filesystem. unloading while they are playing is a crash
        self.filesystem.unload_project();
        *self.project_config = None;
        self.data.unload();
        self.modified.set(false);
    }
}
