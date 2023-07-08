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

use crate::{fl, prelude::*};

/// The event editor window.
pub struct Window {
    id: usize,
    map_id: usize,
    selected_page: usize,
    name: String,
    viewed_tab: u8,
    modals: (bool, bool, bool),
}

impl Window {
    /// Create a new event editor.
    pub fn new(id: usize, map_id: usize) -> Self {
        Self {
            id,
            map_id,
            selected_page: 0,
            name: String::from("(unknown)"),
            viewed_tab: 2,
            modals: (false, false, false),
        }
    }
}

impl window::Window for Window {
    fn name(&self) -> String {
        format!("Event: {}, {} in Map {}", self.name, self.id, self.map_id)
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_event_edit")
            .with(self.map_id)
            .with(self.id)
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        let mut map = state!().data_cache.map(self.map_id);
        let event = match map.events.get_mut(self.id) {
            Some(e) => e,
            None => {
                *open = false;
                return;
            }
        };
        event.extra_data.is_editor_open = true;
        self.name.clone_from(&event.name);

        let mut win_open = true;

        egui::Window::new(self.name())
            .id(egui::Id::new(format!("event_{}_{}", self.id, self.map_id)))
            .open(&mut win_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut event.name);

                    ui.button(fl!("window_event_new_page_btn")).clicked();
                    ui.button(fl!("window_event_copy_page_btn")).clicked();
                    ui.button(fl!("window_event_paste_page_btn")).clicked();
                    ui.button(fl!("window_event_clear_page_btn")).clicked();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    for (page, _) in event.pages.iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.selected_page, page, page.to_string())
                            .clicked()
                        {
                            self.modals = (false, false, false);
                        }
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(
                        &mut self.viewed_tab,
                        0,
                        fl!("window_event_tab_configuration_sv"),
                    );
                    ui.selectable_value(
                        &mut self.viewed_tab,
                        1,
                        fl!("window_event_tab_graphic_sv"),
                    );
                    ui.selectable_value(
                        &mut self.viewed_tab,
                        2,
                        fl!("window_event_tab_commands_sv"),
                    );
                });

                ui.separator();

                let page = event.pages.get_mut(self.selected_page).unwrap();

                match self.viewed_tab {
                    0 => {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(fl!("window_event_conf_condition_label"));
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(
                                            &mut page.condition.switch1_valid,
                                            fl!("window_event_conf_switch_cb"),
                                        );

                                        ui.add_enabled_ui(page.condition.switch1_valid, |ui| {
                                            switch::Modal::new(
                                                format!(
                                                    "event_{}_{}_switch1",
                                                    self.id, self.map_id
                                                )
                                                .into(),
                                            )
                                            .button(
                                                ui,
                                                &mut self.modals.0,
                                                &mut page.condition.switch1_id,
                                            );
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(
                                            &mut page.condition.switch2_valid,
                                            fl!("window_event_conf_switch_cb"),
                                        );

                                        ui.add_enabled_ui(page.condition.switch2_valid, |ui| {
                                            switch::Modal::new(
                                                format!(
                                                    "event_{}_{}_switch2",
                                                    self.id, self.map_id
                                                )
                                                .into(),
                                            )
                                            .button(
                                                ui,
                                                &mut self.modals.1,
                                                &mut page.condition.switch2_id,
                                            );
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(
                                            &mut page.condition.variable_valid,
                                            fl!("window_event_conf_variable_cb"),
                                        );

                                        ui.add_enabled_ui(page.condition.variable_valid, |ui| {
                                            variable::Modal::new(
                                                format!(
                                                    "event_{}_{}_variable",
                                                    self.id, self.map_id
                                                )
                                                .into(),
                                            )
                                            .button(
                                                ui,
                                                &mut self.modals.2,
                                                &mut page.condition.variable_id,
                                            );
                                        });

                                        ui.add_enabled(
                                            page.condition.variable_valid,
                                            egui::DragValue::new(
                                                &mut page.condition.variable_value,
                                            ),
                                        );
                                        ui.label(fl!("window_event_conf_or_above_label"));
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(
                                            &mut page.condition.self_switch_valid,
                                            fl!("window_event_conf_self_switch_cb"),
                                        );
                                        ui.add_enabled_ui(page.condition.self_switch_valid, |ui| {
                                            egui::ComboBox::new(
                                                format!(
                                                    "event_{}_{}_self_switch_combo",
                                                    self.id, self.map_id
                                                ),
                                                fl!("window_event_conf_is_on_label"),
                                            )
                                            .selected_text(page.condition.self_switch_ch.clone())
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    for ch in ["A", "B", "C", "D"] {
                                                        ui.selectable_value(
                                                            &mut page.condition.self_switch_ch,
                                                            ch.to_string(),
                                                            ch,
                                                        );
                                                    }
                                                },
                                            )
                                        });
                                    });
                                });

                                /*
                                ui.label("Autonomous Movement");
                                ui.group(|ui| {
                                    egui::ComboBox::new(
                                        format!("event_{}_{}_move_type", self.id, self.map_id),
                                        "Type",
                                    )
                                    .selected_text(MOVE_TYPES[page.move_type])
                                    .show_ui(ui, |ui| {
                                        for (id, name) in MOVE_TYPES.iter().enumerate() {
                                            ui.selectable_value(&mut page.move_type, id, *name);
                                        }
                                    });

                                    ui.add_enabled_ui(page.move_type == 3, |ui| {
                                        if ui.button("Move route").clicked() {}
                                    });

                                    egui::ComboBox::new(
                                        format!("event_{}_{}_move_speed", self.id, self.map_id),
                                        "Speed",
                                    )
                                    .selected_text(MOVE_SPEEDS[page.move_speed - 1])
                                    .show_ui(ui, |ui| {
                                        for (id, name) in MOVE_SPEEDS.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut page.move_speed,
                                                id + 1,
                                                *name,
                                            );
                                        }
                                    });

                                    egui::ComboBox::new(
                                        format!("event_{}_{}_move_freq", self.id, self.map_id),
                                        "Frequency",
                                    )
                                    .selected_text(MOVE_FREQS[page.move_frequency - 1])
                                    .show_ui(ui, |ui| {
                                        for (id, name) in MOVE_FREQS.iter().enumerate() {
                                            ui.selectable_value(
                                                &mut page.move_frequency,
                                                id + 1,
                                                *name,
                                            );
                                        }
                                    });
                                });
                                */

                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label(fl!("window_event_conf_options_label"));
                                        ui.group(|ui| {
                                            ui.checkbox(
                                                &mut page.step_anime,
                                                fl!("window_event_conf_option_move_anim_cb"),
                                            );
                                            ui.checkbox(
                                                &mut page.walk_anime,
                                                fl!("window_event_conf_option_stop_anim_cb"),
                                            );
                                            ui.checkbox(
                                                &mut page.direction_fix,
                                                fl!("window_event_conf_option_direction_fix_cb"),
                                            );
                                            ui.checkbox(
                                                &mut page.through,
                                                fl!("window_event_conf_option_through_cb"),
                                            );
                                            ui.checkbox(
                                                &mut page.always_on_top,
                                                fl!("window_event_conf_option_aot_cb"),
                                            );
                                        });
                                    });

                                    ui.vertical(|ui| {
                                        ui.label(fl!("window_event_conf_trigger_label"));
                                        ui.group(|ui| {
                                            ui.radio_value(
                                                &mut page.trigger,
                                                0,
                                                fl!("window_event_conf_trigger_action_btn_rv"),
                                            );
                                            ui.radio_value(
                                                &mut page.trigger,
                                                1,
                                                fl!("window_event_conf_trigger_player_touch_rv"),
                                            );
                                            ui.radio_value(
                                                &mut page.trigger,
                                                2,
                                                fl!("window_event_conf_trigger_event_touch_rv"),
                                            );
                                            ui.radio_value(
                                                &mut page.trigger,
                                                3,
                                                fl!("window_event_conf_trigger_autorun_rv"),
                                            );
                                            ui.radio_value(
                                                &mut page.trigger,
                                                4,
                                                fl!("window_event_conf_trigger_parallel_proc_rv"),
                                            );
                                        });
                                    })
                                });
                            });
                        });
                    }

                    1 => {}

                    2 => {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                egui::ScrollArea::both()
                                    .max_height(500.)
                                    .auto_shrink([false; 2])
                                    .show(ui, |_ui| {
                                        // CommandView::new(&mut page.list)
                                        //     .ui(ui, &info.data_cache.commanddb());
                                    });
                            });
                        });
                    }
                    _ => unreachable!(),
                }

                ui.separator();

                ui.horizontal(|ui| {
                    let ok_clicked = ui.button(fl!("ok")).clicked();
                    let apply_clicked = ui.button(fl!("apply")).clicked();
                    let cancel_clicked = ui.button(fl!("cancel")).clicked();

                    if apply_clicked || ok_clicked {
                        //let mut map = state!().data_cache.map(self.map_id);
                        //map.events[self.id] = event.clone();
                    }

                    if cancel_clicked || ok_clicked {
                        *open = false;
                    }
                });
            });
        *open = *open && win_open;
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
