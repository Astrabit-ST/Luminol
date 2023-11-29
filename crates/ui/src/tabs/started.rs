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

/// The Luminol "get started screen" similar to vscode's.
#[derive(Default)]
pub struct Tab {
    // now this is a type
    load_filesystem_promise: Option<poll_promise::Promise<PromiseResult>>,
}

type PromiseResult = luminol_filesystem::Result<luminol_filesystem::host::FileSystem>;

// FIXME
#[allow(unsafe_code)]
unsafe impl Send for Tab {}

impl Tab {
    /// Create a new starting screen.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl luminol_core::Tab for Tab {
    fn name(&self, _update_state: &luminol_core::UpdateState<'_>) -> String {
        "Get Started".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_started_tab")
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        _is_focused: bool,
    ) {
        ui.label(
            egui::RichText::new("Luminol")
                .size(40.)
                .color(egui::Color32::LIGHT_GRAY),
        );

        ui.add_space(100.);

        ui.heading("Start");

        let mut filesystem_open_result = None;
        #[cfg(target_arch = "wasm32")]
        let mut idb_key = None;

        if let Some(p) = self.load_filesystem_promise.take() {
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
                Err(p) => self.load_filesystem_promise = Some(p),
            }

            ui.spinner();
        }

        ui.add_enabled_ui(self.load_filesystem_promise.is_none(), |ui| {
            if ui
                .button(egui::RichText::new("New Project").size(20.))
                .clicked()
            {
                update_state
                    .edit_windows
                    .add_window(crate::windows::new_project::Window::default());
            }
            if ui
                .button(egui::RichText::new("Open Project").size(20.))
                .clicked()
            {
                // maybe worthwhile to make an extension trait to select spawn_async or spawn_local based on the target?
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.load_filesystem_promise = Some(poll_promise::Promise::spawn_async(
                        luminol_filesystem::host::FileSystem::from_file_picker(),
                    ));
                }
                #[cfg(target_arch = "wasm32")]
                {
                    self.load_filesystem_promise = Some(poll_promise::Promise::spawn_local(
                        luminol_filesystem::host::FileSystem::from_folder_picker(),
                    ));
                }
            }
        });

        ui.add_space(100.);

        ui.heading("Recent");

        // FIXME this logic is shared with the top bar
        // We should probably join the two
        for path in update_state.global_config.recent_projects.clone() {
            #[cfg(target_arch = "wasm32")]
            let (path, idb_key) = path;

            if ui.button(&path).clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    filesystem_open_result = Some(update_state.filesystem.load_project_from_path(
                        update_state.project_config,
                        update_state.global_config,
                        path,
                    ));
                }

                #[cfg(target_arch = "wasm32")]
                {
                    self.load_filesystem_promise = Some(poll_promise::Promise::spawn_local(
                        luminol_filesystem::host::FileSystem::from_idb_key(idb_key),
                    ));
                }
            }
        }

        match filesystem_open_result {
            Some(Ok(load_result)) => {
                for missing_rtp in load_result.missing_rtps {
                    update_state.toasts.warning(format!(
                        "Failed to find suitable path for the RTP {missing_rtp}"
                    ));
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
