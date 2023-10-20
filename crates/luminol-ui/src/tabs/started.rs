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
    load_project_promise: Option<poll_promise::Promise<PromiseResult>>,
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
    fn name(&self) -> String {
        "Get Started".to_string()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_started_tab")
    }

    fn show(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
        ui.label(
            egui::RichText::new("Luminol")
                .size(40.)
                .color(egui::Color32::LIGHT_GRAY),
        );

        ui.add_space(100.);

        ui.heading("Start");

        let mut filesystem_open_result = None;

        if let Some(p) = self.load_project_promise.take() {
            match p.try_take() {
                Ok(Ok(host)) => {
                    filesystem_open_result = Some(update_state.filesystem.load_project(
                        host,
                        update_state.project_config,
                        update_state.global_config,
                    ));
                }
                Ok(Err(error)) => update_state.toasts.error(error.to_string()),
                Err(p) => self.load_project_promise = Some(p),
            }

            ui.spinner();
        }

        ui.add_enabled_ui(self.load_project_promise.is_none(), |ui| {
            if ui
                .button(egui::RichText::new("New Project").size(20.))
                .clicked()
            {
                // FIXME
                // update_state
                //     .edit_windows
                //     .add_window(crate::windows::new_project::Window::default());
            }
            if ui
                .button(egui::RichText::new("Open Project").size(20.))
                .clicked()
            {
                self.load_project_promise = Some(poll_promise::Promise::spawn_local(
                    luminol_filesystem::host::FileSystem::from_pile_picker(),
                ));
            }
        });

        ui.add_space(100.);

        ui.heading("Recent");

        // FIXME
        for path in update_state.global_config.recent_projects.clone() {
            #[cfg(target_arch = "wasm32")]
            let (path, idb_key) = path;

            if ui.button(&path).clicked() {
                let path = path.clone();
                #[cfg(target_arch = "wasm32")]
                let idb_key = idb_key.clone();

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
                    self.load_project_promise =
                        Some(poll_promise::Promise::spawn_local(async move {
                            result =
                                match filesystem::web::FileSystem::from_idb_key(idb_key.clone())
                                    .await
                                {
                                    Some(dir) => {
                                        let idb_key = dir.idb_key().map(|k| k.to_string());
                                        if let Err(e) = state.filesystem.load_project(dir) {
                                            if let Some(idb_key) = idb_key {
                                                filesystem::web::FileSystem::idb_drop(idb_key);
                                            }
                                            Err(e)
                                        } else {
                                            Ok(())
                                        }
                                    }
                                    None => Err("Could not restore project handle from IndexedDB"
                                        .to_string()),
                                };
                        }));
                }
            }
        }

        match filesystem_open_result {
            Some(Ok(_)) => {
                if let Err(why) = update_state.data.load(
                    update_state.filesystem,
                    // TODO code jank
                    update_state.project_config.as_mut().unwrap(),
                ) {
                    update_state
                        .toasts
                        .error(format!("Error loading the project data: {why}"));
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
            }
            None => {}
        }
    }
}
