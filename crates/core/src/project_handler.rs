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

pub struct ProjectHandler {
    modal: egui_modal::Modal,
    closure: Option<Box<dyn Fn(&mut luminol_eframe::Frame, &mut crate::UpdateState<'_>)>>,
}

impl ProjectHandler {
    pub fn new(ctx: &egui::Context) -> Self {
        Self {
            modal: egui_modal::Modal::new(ctx, "luminol_save_modal"),
            closure: None,
        }
    }

    fn check_and_then(
        &mut self,
        closure: impl Fn(&mut luminol_eframe::Frame, &mut crate::UpdateState<'_>) + 'static,
    ) {
        self.closure = Some(Box::new(closure));
        self.modal.open();
    }

    #[cfg(not(target_arch = "wasm32"))]
    /// Closes the application after asking the user to save unsaved changes.
    pub fn quit(&mut self) {
        self.check_and_then(|frame, update_state| {
            // Disable the modified flag so `luminol_eframe::App::on_close_event` doesn't recurse
            update_state.modified.set(false);

            frame.close();
        });
    }

    pub fn show_unsaved_changes_modal(
        &mut self,
        frame: &mut luminol_eframe::Frame,
        update_state: &mut crate::UpdateState<'_>,
    ) {
        let mut should_close = false;
        let mut should_save = false;
        let mut should_run_closure = false;

        self.modal.show(|ui| {
            self.modal.title(ui, "Unsaved changes");
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
                if let Err(_err) = update_state.data.save(
                    update_state.filesystem,
                    update_state.project_config.as_ref().unwrap(),
                ) {
                    todo!()
                }
            }

            if should_run_closure {
                if let Some(closure) = &self.closure {
                    closure(frame, update_state);
                }
            }

            self.closure = None;
        }
    }
}
