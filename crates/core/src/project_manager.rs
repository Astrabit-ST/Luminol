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
    modal: egui_modal::Modal,
    closure: Option<
        Box<
            dyn Fn(&mut ProjectManagerState, &mut crate::ModifiedState, &mut luminol_eframe::Frame),
        >,
    >,
}

#[derive(Default)]
pub struct ProjectManagerState {
    pub load_filesystem_promise: Option<poll_promise::Promise<FileSystemPromiseResult>>,
}

type FileSystemPromiseResult = luminol_filesystem::Result<luminol_filesystem::host::FileSystem>;

impl ProjectManager {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            modal: egui_modal::Modal::new(ctx, "luminol_save_modal"),
            closure: None,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Closes the application after asking the user to save unsaved changes.
    pub fn quit(&mut self) {
        self.closure = Some(Box::new(|_projman_state, modified, frame| {
            // Disable the modified flag so `luminol_eframe::App::on_close_event` doesn't recurse
            modified.set(false);

            frame.close();
        }));
    }

    /// Opens a project picker after asking the user to save unsaved changes.
    pub fn load_project(&mut self) {
        self.closure = Some(Box::new(|projman_state, _modified, _frame| {
            // maybe worthwhile to make an extension trait to select spawn_async or spawn_local based on the target?
            #[cfg(not(target_arch = "wasm32"))]
            {
                projman_state.load_filesystem_promise = Some(poll_promise::Promise::spawn_async(
                    luminol_filesystem::host::FileSystem::from_file_picker(),
                ));
            }
            #[cfg(target_arch = "wasm32")]
            {
                projman_state.load_filesystem_promise = Some(poll_promise::Promise::spawn_local(
                    luminol_filesystem::host::FileSystem::from_folder_picker(),
                ));
            }
        }));
    }

    // FIXME maybe make a struct inside of UpdateState that contains only these fields?
    pub fn show_unsaved_changes_modal(
        &mut self,
        data: &mut crate::Data,
        filesystem: &mut luminol_filesystem::project::FileSystem,
        project_config: &Option<luminol_config::project::Config>,
        modified: &mut crate::ModifiedState,
        projman_state: &mut ProjectManagerState,
        frame: &mut luminol_eframe::Frame,
    ) {
        let mut should_close = false;
        let mut should_save = false;
        let mut should_run_closure = false;

        if self.closure.is_some() {
            if !modified.get() {
                should_close = true;
                should_run_closure = true;
            } else if !self.modal.is_open() {
                self.modal.open();
            }
        }

        self.modal.show(|ui| {
            self.modal.title(ui, "Unsaved Changes");
            self.modal.frame(ui, |ui| {
                self.modal
                    .body(ui, "Do you want to save your changes to this project?");
            });

            self.modal.buttons(ui, |ui| {
                if self.modal.button(ui, "Cancel").clicked() {
                    should_close = true;
                } else if self.modal.caution_button(ui, "Discard").clicked() {
                    should_close = true;
                    should_run_closure = true;
                } else if self.modal.suggested_button(ui, "Save").clicked() {
                    should_close = true;
                    should_save = true;
                    should_run_closure = true;
                }
            });
        });

        if should_close {
            if should_save {
                if let Err(_err) = data.save(filesystem, project_config.as_ref().unwrap()) {
                    todo!()
                }
                modified.set(false);
            }

            if should_run_closure {
                if let Some(closure) = &self.closure {
                    closure(projman_state, modified, frame);
                }
            }

            self.closure = None;
        }
    }

    pub fn handle_project_loading(update_state: &mut crate::UpdateState<'_>) {
        let mut filesystem_open_result = None;
        #[cfg(target_arch = "wasm32")]
        let mut idb_key = None;

        if let Some(p) = update_state.projman_state.load_filesystem_promise.take() {
            match p.try_take() {
                Ok(Ok(host)) => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        idb_key = host.idb_key().map(str::to_string);
                    }

                    filesystem_open_result = Some(update_state.filesystem.load_project(
                        host,
                        update_state.project_config,
                        update_state.global_config,
                    ));
                }
                Ok(Err(error)) => update_state.toasts.error(error.to_string()),
                Err(p) => update_state.projman_state.load_filesystem_promise = Some(p),
            }
        }

        match filesystem_open_result {
            Some(Ok(load_result)) => {
                for missing_rtp in load_result.missing_rtps {
                    update_state.toasts.warning(format!(
                        "Failed to find suitable path for the RTP {missing_rtp}"
                    ));
                    // FIXME we should probably load rtps from the RTP/<rtp> paths on non-wasm targets
                    #[cfg(not(target_arch = "wasm32"))]
                    update_state
                        .toasts
                        .info(format!("You may want to set an RTP path for {missing_rtp}"));
                    #[cfg(target_arch = "wasm32")]
                    update_state
                        .toasts
                        .info(format!("Please place the {missing_rtp} RTP in the 'RTP/{missing_rtp}' subdirectory in your project directory"));
                }

                if let Err(why) = update_state.data.load(
                    update_state.filesystem,
                    // TODO code jank
                    update_state.project_config.as_mut().unwrap(),
                ) {
                    update_state
                        .toasts
                        .error(format!("Error loading the project data: {why}"));

                    #[cfg(target_arch = "wasm32")]
                    idb_key.map(luminol_filesystem::host::FileSystem::idb_drop);
                } else {
                    update_state.toasts.info(format!(
                        "Successfully opened {:?}",
                        update_state
                            .filesystem
                            .project_path()
                            .expect("project not open")
                    ));
                }
            }
            Some(Err(why)) => {
                update_state
                    .toasts
                    .error(format!("Error opening the project: {why}"));

                #[cfg(target_arch = "wasm32")]
                idb_key.map(luminol_filesystem::host::FileSystem::idb_drop);
            }
            None => {}
        }
    }
}
