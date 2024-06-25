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

use luminol_core::Modal;
use luminol_data::rpg;
use luminol_modals::{switch, variable};

/// The event editor window.
pub struct Window {
    map_id: usize,
    event: rpg::Event,
    selected_page: usize,

    switch_1_modal: switch::Modal,
    switch_2_modal: switch::Modal,
    variable_modal: variable::Modal,
}

impl Window {
    /// Create a new event editor.
    pub fn new(event: rpg::Event, map_id: usize) -> Self {
        let id_source = egui::Id::new("event_edit").with(event.id).with(map_id);
        Self {
            map_id,
            event,
            selected_page: 0,

            switch_1_modal: switch::Modal::new(id_source.with("switch_1_modal")),
            switch_2_modal: switch::Modal::new(id_source.with("switch_2_modal")),
            variable_modal: variable::Modal::new(id_source.with("variable_modal")),
        }
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        format!("Event '{}' ID {}", self.event.name, self.event.id)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_event_edit")
            .with(self.map_id)
            .with(self.event.id)
    }

    // This needs an overhaul
    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Page: ");
                for i in 0..self.event.pages.len() {
                    ui.selectable_value(&mut self.selected_page, i, format!("{}", i + 1));
                }

                if ui
                    .button(egui::RichText::new("Add").color(egui::Color32::LIGHT_GREEN))
                    .clicked()
                {
                    self.event.pages.push(rpg::EventPage::default());
                    self.selected_page = self.event.pages.len() - 1;
                }

                let button = egui::Button::new(
                    egui::RichText::new("Delete").color(egui::Color32::LIGHT_RED),
                );
                if ui.add_enabled(self.event.pages.len() > 1, button).clicked() {
                    self.event.pages.remove(self.selected_page);
                    self.selected_page = self.selected_page.saturating_sub(1);
                }
                if ui.button(egui::RichText::new("Clear")).clicked() {
                    self.event.pages[self.selected_page] = rpg::EventPage::default();
                }
            });
            ui.separator();

            let id_source = self.id();
            let page = &mut self.event.pages[self.selected_page];

            ui.columns(2, |columns| {
                columns[0].horizontal(|ui| {
                    ui.checkbox(&mut page.condition.switch1_valid, "Switch");
                    ui.add_enabled(
                        page.condition.switch1_valid,
                        self.switch_1_modal
                            .button(&mut page.condition.switch1_id, update_state),
                    );
                    ui.label("is ON");
                });
                columns[1].horizontal(|ui| {
                    ui.checkbox(&mut page.condition.switch2_valid, "Switch");
                    ui.add_enabled(
                        page.condition.switch2_valid,
                        self.switch_2_modal
                            .button(&mut page.condition.switch2_id, update_state),
                    );
                    ui.label("is ON");
                });
                columns[0].horizontal(|ui| {
                    ui.checkbox(&mut page.condition.variable_valid, "Variable");
                    ui.add_enabled(
                        page.condition.variable_valid,
                        self.variable_modal
                            .button(&mut page.condition.variable_id, update_state),
                    );
                    ui.label("is");
                    ui.add_enabled(
                        page.condition.variable_valid,
                        egui::DragValue::new(&mut page.condition.variable_value),
                    );
                    ui.label("or above");
                });
                columns[1].horizontal(|ui| {
                    ui.checkbox(&mut page.condition.self_switch_valid, "Self Switch");
                    ui.add_enabled(
                        page.condition.self_switch_valid,
                        luminol_components::EnumMenuButton::new(
                            &mut page.condition.self_switch_ch,
                            id_source.with("self_switch_ch"),
                        ),
                    );
                    ui.label("is ON");
                });
            });
            ui.separator();
            ui.columns(2, |columns| {
                columns[0].checkbox(&mut page.walk_anime, "Move Animation");
                columns[0].checkbox(&mut page.step_anime, "Stop Animation");
                columns[0].checkbox(&mut page.direction_fix, "Direction Fix");
                columns[0].checkbox(&mut page.through, "Through");
                columns[0].checkbox(&mut page.always_on_top, "Always On Top");

                columns[1].add(luminol_components::EnumMenuButton::new(
                    &mut page.trigger,
                    id_source.with("trigger"),
                ));
            });
        });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
