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

use luminol_core::Modal;
use luminol_modals::database_modal;

/// The common event editor.
pub struct Window {
    tabs: luminol_core::Tabs,
    _selected_id: usize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            tabs: luminol_core::Tabs::new("common_event_tabs", false),
            _selected_id: 0,
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("Common Events")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        _update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let name = self
            .tabs
            .focused_name()
            .map_or("Common Events".to_string(), |name| {
                format!("Editing Common Event {name}")
            });
        egui::Window::new(name)
            .default_width(500.)
            .id(egui::Id::new("common_events_edit"))
            .open(open)
            .show(ctx, |_| {
                // TODO
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

pub struct CommonEventTab {
    event: luminol_data::rpg::CommonEvent,
    force_close: bool,
    switch_modal: database_modal::SwitchModal,
    command_view: luminol_components::CommandView,
}

impl luminol_core::Tab for CommonEventTab {
    fn name(&self, _update_state: &luminol_core::UpdateState<'_>) -> String {
        format!("{}: {}", self.event.name, self.event.id)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_common_event").with(self.event.id)
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut luminol_core::UpdateState<'_>,
        _is_focused: bool,
    ) {
        ui.horizontal(|ui| {
            let trigger_types = ["None", "Autorun", "Parallel"];
            egui::ComboBox::new(format!("common_event_{}_trigger", self.event.id), "Trigger")
                .selected_text(trigger_types[self.event.trigger])
                .show_ui(ui, |ui| {
                    for (ele, trigger) in trigger_types.into_iter().enumerate() {
                        ui.selectable_value(&mut self.event.trigger, ele, trigger);
                    }
                });

            ui.add_enabled(
                self.event.trigger > 0,
                self.switch_modal
                    .button(&mut self.event.switch_id, update_state),
            );

            let mut save_event = false;

            if ui.button("Ok").clicked() {
                save_event = true;
                self.force_close = true;
            }

            if ui.button("Cancel").clicked() {
                self.force_close = true;
            }

            if ui.button("Apply").clicked() {
                save_event = true;
            }

            if save_event {
                let mut common_events = update_state.data.common_events();

                common_events.data[self.event.id - 1] = self.event.clone();
            }

            ui.label("Name");
            ui.text_edit_singleline(&mut self.event.name);
        });

        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.command_view.ui(
                    ui,
                    &update_state.project_config.as_ref().unwrap().command_db,
                    &mut self.event.list,
                );
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }
}
