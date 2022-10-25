// Copyright (C) 2022 Lily Lyons
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

use crate::{
    components::command_view::CommandView,
    data::rmxp_structs::rpg,
    modals::{modal::Modal, switch::SwitchModal},
    tabs::tab::{Tab, Tabs},
};

use super::window::Window;

/// The common event editor.
pub struct CommonEventEdit {
    tabs: Tabs,
    selected_id: usize,
}

impl Default for CommonEventEdit {
    fn default() -> Self {
        Self {
            tabs: Tabs::new("common_event_tabs"),
            selected_id: 0,
        }
    }
}

impl Window for CommonEventEdit {
    fn name(&self) -> String {
        self.tabs
            .focused_name()
            .map_or("Common Events".to_string(), |name| {
                format!("Editing Common Event {}", name)
            })
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        egui::Window::new(self.name())
            .default_width(500.)
            .id(egui::Id::new("common_events_edit"))
            .open(open)
            .show(ctx, |ui| {
                let mut common_events = info.data_cache.common_events();
                let common_events = common_events.as_mut().unwrap();

                egui::SidePanel::left("common_events_side_panel").show_inside(ui, |ui| {
                    egui::ScrollArea::both().auto_shrink([false; 2]).show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        common_events.len(),
                        |ui, rows| {
                            for (ele, event) in common_events
                                .iter()
                                .enumerate()
                                .filter(|(ele, _)| rows.contains(ele))
                            {
                                if ui
                                    .selectable_value(
                                        &mut self.selected_id,
                                        ele,
                                        format!("{}: {}", event.id, event.name),
                                    )
                                    .double_clicked()
                                {
                                    self.tabs.add_tab(CommonEventTab {
                                        event: event.clone(),
                                        force_close: false,
                                        switch_open: false,
                                    })
                                }
                            }
                        },
                    );
                });

                self.tabs.ui(ui, info);
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}

struct CommonEventTab {
    event: rpg::CommonEvent,
    force_close: bool,
    switch_open: bool,
}

impl Tab for CommonEventTab {
    fn name(&self) -> String {
        format!("{}: {}", self.event.name, self.event.id)
    }

    fn show(&mut self, ui: &mut egui::Ui, info: &'static crate::UpdateInfo) {
        ui.horizontal(|ui| {
            let trigger_types = ["None", "Autorun", "Parallel"];
            egui::ComboBox::new(format!("common_event_{}_trigger", self.event.id), "Trigger")
                .selected_text(trigger_types[self.event.trigger])
                .show_ui(ui, |ui| {
                    for (ele, trigger) in trigger_types.into_iter().enumerate() {
                        ui.selectable_value(&mut self.event.trigger, ele, trigger);
                    }
                });

            ui.add_enabled_ui(self.event.trigger > 0, |ui| {
                SwitchModal::new(format!("common_event_{}_trigger_switch", self.event.id)).button(
                    ui,
                    &mut self.switch_open,
                    &mut self.event.switch_id,
                    info,
                )
            });

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
                let mut common_events = info.data_cache.common_events();
                let common_events = common_events.as_mut().unwrap();

                common_events[self.event.id] = self.event.clone();
            }

            ui.label("Name");
            ui.text_edit_singleline(&mut self.event.name);
        });

        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                CommandView::new(
                    &mut self.event.list,
                    &format!("common_event_{}", self.event.id),
                )
                .ui(ui, info);
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    #[cfg(feature = "discord-rpc")]
    fn discord_display(&self) -> String {
        "".to_string()
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }
}
