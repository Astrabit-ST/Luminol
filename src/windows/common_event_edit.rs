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

use crate::prelude::*;

/// The common event editor.
pub struct Window {
    tabs: tab::Tabs<CommonEventTab>,
    selected_id: usize,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            tabs: tab::Tabs::new("common_event_tabs", vec![]),
            selected_id: 0,
        }
    }
}

impl window::Window for Window {
    fn name(&self) -> String {
        self.tabs
            .focused_name()
            .map_or("Common Events".to_string(), |name| {
                format!("Editing Common Event {name}")
            })
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Common Events")
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .default_width(500.)
            .id(egui::Id::new("common_events_edit"))
            .open(open)
            .show(ctx, |ui| {
                egui::SidePanel::left("common_events_side_panel").show_inside(ui, |ui| {
                    let common_events = state!().data_cache.commonevents();

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
                                        command_view: CommandView::new(format!(
                                            "common_event_{ele}"
                                        )),
                                    });
                                }
                            }
                        },
                    );
                });

                self.tabs.ui(ui);
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
    command_view: CommandView,
}

impl tab::Tab for CommonEventTab {
    fn name(&self) -> String {
        format!("{}: {}", self.event.name, self.event.id)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_common_event").with(self.event.id)
    }

    fn show(&mut self, ui: &mut egui::Ui) {
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
                switch::Modal::new(format!("common_event_{}_trigger_switch", self.event.id).into())
                    .button(ui, &mut self.switch_open, &mut self.event.switch_id)
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
                let mut common_events = state!().data_cache.commonevents();

                common_events[self.event.id - 1] = self.event.clone();
            }

            ui.label("Name");
            ui.text_edit_singleline(&mut self.event.name);
        });

        ui.separator();

        egui::ScrollArea::both()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                self.command_view
                    .ui(ui, &state!().data_cache.commanddb(), &mut self.event.list);
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn force_close(&mut self) -> bool {
        self.force_close
    }
}
