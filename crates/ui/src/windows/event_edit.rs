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

use egui::Widget;
use luminol_core::prelude::*;
use luminol_modals::{database_modal, event_graphic_picker};

/// The event editor window.
pub struct Window {
    map_id: usize,
    event: rpg::Event,
    selected_page: usize,

    switch_1_modal: database_modal::SwitchModal,
    switch_2_modal: database_modal::SwitchModal,
    variable_modal: database_modal::VariableModal,
    graphic_modal: event_graphic_picker::Modal,
}

impl Window {
    /// Create a new event editor.
    pub fn new(
        update_state: &UpdateState<'_>,
        event: rpg::Event,
        map_id: usize,
        tileset: &rpg::Tileset,
    ) -> Self {
        let id_source = egui::Id::new("luminol_event_edit")
            .with(event.id)
            .with(map_id);
        let graphic_modal = event_graphic_picker::Modal::new(
            update_state,
            &event.pages[0].graphic,
            tileset,
            id_source.with("graphic_modal"),
        );
        Self {
            map_id,
            event,
            selected_page: 0,

            switch_1_modal: database_modal::Modal::new(id_source.with("switch_1_modal")),
            switch_2_modal: database_modal::Modal::new(id_source.with("switch_2_modal")),
            variable_modal: database_modal::Modal::new(id_source.with("variable_modal")),
            graphic_modal,
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

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut win_open = true;
        let mut needs_save = false;

        egui::Window::new(self.name())
            .open(&mut win_open)
            .show(ctx, |ui| {
                let id_source = self.id();
                let previous_page = self.selected_page;
                egui::TopBottomPanel::top(id_source.with("top_panel")).show_inside(ui, |ui| {
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
                });

                let page = &mut self.event.pages[self.selected_page];
                if self.selected_page != previous_page {
                    // we need to update the modal to prevent desyncs
                    self.graphic_modal
                        .update_graphic(update_state, &page.graphic);
                }

                egui::SidePanel::left(id_source.with("side_panel")).show_inside(ui, |ui| {
                    ui.label("Conditions");
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.switch1_valid, "Switch");
                            ui.add_enabled(
                                page.condition.switch1_valid,
                                self.switch_1_modal
                                    .button(&mut page.condition.switch1_id, update_state),
                            );
                            ui.label("is ON");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.switch2_valid, "Switch");
                            ui.add_enabled(
                                page.condition.switch2_valid,
                                self.switch_2_modal
                                    .button(&mut page.condition.switch2_id, update_state),
                            );
                            ui.label("is ON");
                        });
                        ui.horizontal(|ui| {
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
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.self_switch_valid, "Self Switch");
                            // TODO add self switch text box (config option)
                            ui.add_enabled(
                                // FIXME ensure shrink
                                page.condition.self_switch_valid,
                                luminol_components::EnumMenuButton::new(
                                    &mut page.condition.self_switch_ch,
                                    id_source.with("self_switch_ch"),
                                ),
                            );
                            ui.label("is ON");
                            // ensure we expand to fit the side panel
                            ui.add_space(ui.available_width());
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Graphic");

                            self.graphic_modal
                                .button(&mut page.graphic, update_state)
                                .ui(ui);
                        });
                        ui.vertical(|ui| {
                            ui.label("Autonomous Movement");
                            ui.group(|ui| {
                                // FIXME these expand to fit, which is kinda annoying
                                ui.horizontal(|ui| {
                                    ui.label("Move Type");
                                    luminol_components::EnumComboBox::new(
                                        id_source.with("move_type"),
                                        &mut page.move_type,
                                    )
                                    .ui(ui);
                                });
                                ui.add_enabled(
                                    page.move_type == luminol_data::rpg::MoveType::Custom,
                                    egui::Button::new("Move Route..."),
                                ); // TODO
                                ui.horizontal(|ui| {
                                    ui.label("Move Speed");
                                    luminol_components::EnumComboBox::new(
                                        id_source.with("move_speed"),
                                        &mut page.move_speed,
                                    )
                                    .ui(ui);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Move Frequency");
                                    luminol_components::EnumComboBox::new(
                                        id_source.with("move_frequency"),
                                        &mut page.move_frequency,
                                    )
                                    .ui(ui);
                                });
                                ui.add_space(ui.available_height());
                            });
                        });
                    });

                    ui.columns(2, |columns| {
                        let [left, right] = columns else {
                            unreachable!()
                        };

                        left.label("Options");
                        left.group(|ui| {
                            ui.style_mut().wrap = Some(false);
                            ui.checkbox(&mut page.walk_anime, "Move Animation");
                            ui.checkbox(&mut page.step_anime, "Step Animation");
                            ui.checkbox(&mut page.direction_fix, "Direction Fix");
                            ui.checkbox(&mut page.through, "Through");
                            ui.checkbox(&mut page.always_on_top, "Always on Top");
                        });

                        right.label("Trigger");
                        right.group(|ui| {
                            luminol_components::EnumRadioList::new(&mut page.trigger).ui(ui);
                        });
                    });
                });

                egui::TopBottomPanel::bottom(id_source.with("bottom_panel")).show_inside(
                    ui,
                    |ui| {
                        ui.add_space(ui.style().spacing.item_spacing.y);
                        luminol_components::close_options_ui(ui, open, &mut needs_save)
                    },
                );
            });

        if needs_save {
            self.event.extra_data.modified.set(true);
            let mut map = update_state.data.get_map(self.map_id);
            map.events.insert(self.event.id, self.event.clone()); // don't like the extra clone, but it's necessary
        }

        *open &= win_open;
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
