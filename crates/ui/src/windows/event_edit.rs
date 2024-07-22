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

use egui::Widget;
use luminol_core::prelude::*;
use luminol_modals::{
    database_modal::{SwitchModal, VariableModal},
    graphic_picker::event::Modal as GraphicPicker,
};

/// The event editor window.
pub struct Window {
    map_id: usize,
    event_id: usize,
    selected_page: usize,

    switch_1_modal: SwitchModal,
    switch_2_modal: SwitchModal,
    variable_modal: VariableModal,
    graphic_modal: GraphicPicker,
}

impl Window {
    /// Create a new event editor.
    pub fn new(
        update_state: &UpdateState<'_>,
        event: &rpg::Event,
        map_id: usize,
        tileset_id: usize,
    ) -> Self {
        let id_source = egui::Id::new("luminol_event_edit")
            .with(event.id)
            .with(map_id);
        let graphic_modal = GraphicPicker::new(
            update_state,
            &event.pages[0].graphic,
            tileset_id,
            id_source.with("graphic_modal"),
        );
        Self {
            map_id,
            event_id: event.id,
            selected_page: 0,

            switch_1_modal: SwitchModal::new(id_source.with("switch_1_modal")),
            switch_2_modal: SwitchModal::new(id_source.with("switch_2_modal")),
            variable_modal: VariableModal::new(id_source.with("variable_modal")),
            graphic_modal,
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_event_edit")
            .with(self.map_id)
            .with(self.event_id)
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        // to avoid borrowing issues, we temporarily remove the event from the map.
        // this is a pretty cheap operation because it's Option::take.
        let mut map = update_state.data.get_map(self.map_id);
        let Some(mut event) = map.events.option_remove(self.event_id) else {
            *open = false;
            return;
        };
        drop(map);

        let mut modified = false;
        let mut graphic_modified = false;

        egui::Window::new(format!("Event '{}' ID {}", event.name, self.event_id))
            .open(open)
            .id(self.id())
            .show(ctx, |ui| {
                let id_source = self.id();
                let previous_page = self.selected_page;

                egui::TopBottomPanel::top(id_source.with("top_panel")).show_inside(ui, |ui| {
                    ui.add_space(1.0); // pad the top of the window
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        ui.text_edit_singleline(&mut event.name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Page: ");
                        for i in 0..event.pages.len() {
                            ui.selectable_value(&mut self.selected_page, i, format!("{}", i + 1));
                        }

                        if ui
                            .button(egui::RichText::new("Add").color(egui::Color32::LIGHT_GREEN))
                            .clicked()
                        {
                            modified |= true;
                            event.pages.push(rpg::EventPage::default());
                            self.selected_page = event.pages.len() - 1;
                        }

                        let button = egui::Button::new(
                            egui::RichText::new("Delete").color(egui::Color32::LIGHT_RED),
                        );
                        if ui.add_enabled(event.pages.len() > 1, button).clicked() {
                            modified |= true;
                            event.pages.remove(self.selected_page);
                            self.selected_page = self.selected_page.saturating_sub(1);
                        }
                        if ui.button(egui::RichText::new("Clear")).clicked() {
                            modified |= true;
                            event.pages[self.selected_page] = rpg::EventPage::default();
                        }
                    });
                    ui.add_space(1.0); // pad the bottom of the window
                });

                let page = &mut event.pages[self.selected_page];
                if self.selected_page != previous_page {
                    // reset the modal if we've changed pages
                    self.graphic_modal.reset(update_state, &mut page.graphic);
                }

                egui::SidePanel::left(id_source.with("side_panel")).show_inside(ui, |ui| {
                    ui.label("Conditions");
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.switch1_valid, "Switch");
                            let res = ui.add_enabled(
                                page.condition.switch1_valid,
                                self.switch_1_modal
                                    .button(&mut page.condition.switch1_id, update_state),
                            );
                            modified |= res.changed();
                            ui.label("is ON");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.switch2_valid, "Switch");
                            let res = ui.add_enabled(
                                page.condition.switch2_valid,
                                self.switch_2_modal
                                    .button(&mut page.condition.switch2_id, update_state),
                            );
                            modified |= res.changed();
                            ui.label("is ON");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.variable_valid, "Variable");
                            let res = ui.add_enabled(
                                page.condition.variable_valid,
                                self.variable_modal
                                    .button(&mut page.condition.variable_id, update_state),
                            );
                            modified |= res.changed();
                            ui.label("is");
                            let res = ui.add_enabled(
                                page.condition.variable_valid,
                                egui::DragValue::new(&mut page.condition.variable_value),
                            );
                            modified |= res.changed();
                            ui.label("or above");
                        });
                        ui.horizontal(|ui| {
                            ui.checkbox(&mut page.condition.self_switch_valid, "Self Switch");
                            // TODO add self switch text box (config option)
                            let res = ui.add_enabled(
                                // FIXME ensure shrink
                                page.condition.self_switch_valid,
                                luminol_components::EnumMenuButton::new(
                                    &mut page.condition.self_switch_ch,
                                    id_source.with("self_switch_ch"),
                                ),
                            );
                            modified |= res.changed();
                            ui.label("is ON");
                            // ensure we expand to fit the side panel
                            ui.add_space(ui.available_width()); // cross justify doesn't seem to be able to replace this?
                        });
                    });

                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Graphic");

                            graphic_modified = self
                                .graphic_modal
                                .button(&mut page.graphic, update_state)
                                .ui(ui)
                                .changed();
                        });
                        ui.vertical(|ui| {
                            ui.label("Autonomous Movement");
                            ui.group(|ui| {
                                // FIXME these expand to fit, which is kinda annoying
                                ui.horizontal(|ui| {
                                    ui.label("Move Type");
                                    modified |= luminol_components::EnumComboBox::new(
                                        id_source.with("move_type"),
                                        &mut page.move_type,
                                    )
                                    .ui(ui)
                                    .changed();
                                });
                                ui.add_enabled(
                                    page.move_type == luminol_data::rpg::MoveType::Custom,
                                    egui::Button::new("Move Route..."),
                                ); // TODO
                                ui.horizontal(|ui| {
                                    ui.label("Move Speed");
                                    modified |= luminol_components::EnumComboBox::new(
                                        id_source.with("move_speed"),
                                        &mut page.move_speed,
                                    )
                                    .ui(ui)
                                    .changed();
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Move Frequency");
                                    modified |= luminol_components::EnumComboBox::new(
                                        id_source.with("move_frequency"),
                                        &mut page.move_frequency,
                                    )
                                    .ui(ui)
                                    .changed();
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
                            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                            modified |= ui
                                .checkbox(&mut page.walk_anime, "Move Animation")
                                .changed();
                            modified |= ui
                                .checkbox(&mut page.step_anime, "Stop Animation")
                                .changed();
                            modified |= ui
                                .checkbox(&mut page.direction_fix, "Direction Fix")
                                .changed();
                            modified |= ui.checkbox(&mut page.through, "Through").changed();
                            modified |= ui
                                .checkbox(&mut page.always_on_top, "Always on Top")
                                .changed();
                        });

                        right.label("Trigger");
                        right.group(|ui| {
                            modified |= luminol_components::EnumRadioList::new(&mut page.trigger)
                                .ui(ui)
                                .changed();
                        });
                    });
                });
            });

        if graphic_modified {
            event.extra_data.graphic_modified.set(true);
        }

        // reinsert the event into the map
        let mut map = update_state.data.get_map(self.map_id);
        map.events.insert(self.event_id, event);

        if modified {
            map.modified = true;
        }
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
