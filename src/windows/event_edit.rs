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

use egui_extras::RetainedImage;
use poll_promise::Promise;

use super::window::Window;
use crate::components::command_view::CommandView;
use crate::data::commands::{MOVE_FREQS, MOVE_SPEEDS, MOVE_TYPES};
use crate::data::rmxp_structs::rpg;
use crate::modals::modal::Modal;
use crate::modals::switch::SwitchModal;
use crate::modals::variable::VariableModal;
use crate::{load_image_software, UpdateInfo};

/// The event editor window.
pub struct EventEdit {
    id: usize,
    map_id: i32,
    selected_page: usize,
    event: rpg::event::Event,
    page_graphics_promise: Promise<(Vec<Option<RetainedImage>>, RetainedImage)>,
    viewed_tab: u8,
    modals: (bool, bool, bool),
}

impl EventEdit {
    /// Create a new event editor.
    pub fn new(
        id: usize,
        map_id: i32,
        event: rpg::event::Event,
        tileset_name: String,
        info: &'static UpdateInfo,
    ) -> Self {
        let pages_graphics: Vec<_> = event.pages.iter().map(|p| p.graphic.clone()).collect();
        Self {
            id,
            map_id,
            selected_page: 0,
            event,
            page_graphics_promise: Promise::spawn_local(async move {
                let futures = pages_graphics.iter().map(|p| {
                    load_image_software(format!("Graphics/Characters/{}", p.character_name), info)
                });
                (
                    futures::future::join_all(futures)
                        .await
                        .into_iter()
                        .map(|e| e.ok())
                        .collect(),
                    load_image_software(format!("Graphics/Tilesets/{tileset_name}"), info)
                        .await
                        .unwrap(),
                )
            }),
            viewed_tab: 2,
            modals: (false, false, false),
        }
    }
}

impl Window for EventEdit {
    fn name(&self) -> String {
        format!(
            "Event: {}, {} in Map {}",
            self.event.name, self.id, self.map_id
        )
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, info: &'static crate::UpdateInfo) {
        let mut win_open = true;

        egui::Window::new(self.name())
            .id(egui::Id::new(format!("event_{}_{}", self.id, self.map_id)))
            .open(&mut win_open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.event.name);

                    ui.button("New page").clicked();
                    ui.button("Copy page").clicked();
                    ui.button("Paste page").clicked();
                    ui.button("Clear page").clicked();
                });

                ui.separator();

                ui.horizontal(|ui| {
                    for (page, _) in self.event.pages.iter().enumerate() {
                        if ui
                            .selectable_value(&mut self.selected_page, page, page.to_string())
                            .clicked()
                        {
                            self.modals = (false, false, false)
                        }
                    }
                });

                ui.separator();

                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.viewed_tab, 0, "Configuration");
                    ui.selectable_value(&mut self.viewed_tab, 1, "Graphic");
                    ui.selectable_value(&mut self.viewed_tab, 2, "Commands");
                });

                ui.separator();

                let page = self.event.pages.get_mut(self.selected_page).unwrap();

                match self.viewed_tab {
                    0 => {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Condition");
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut page.condition.switch1_valid, "Switch");

                                        ui.add_enabled_ui(page.condition.switch1_valid, |ui| {
                                            SwitchModal::new(format!(
                                                "event_{}_{}_switch1",
                                                self.id, self.map_id
                                            ))
                                            .button(
                                                ui,
                                                &mut self.modals.0,
                                                &mut page.condition.switch1_id,
                                                info,
                                            );
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut page.condition.switch2_valid, "Switch");

                                        ui.add_enabled_ui(page.condition.switch2_valid, |ui| {
                                            SwitchModal::new(format!(
                                                "event_{}_{}_switch2",
                                                self.id, self.map_id
                                            ))
                                            .button(
                                                ui,
                                                &mut self.modals.1,
                                                &mut page.condition.switch2_id,
                                                info,
                                            );
                                        });
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut page.condition.variable_valid, "Variable");

                                        ui.add_enabled_ui(page.condition.variable_valid, |ui| {
                                            VariableModal::new(format!(
                                                "event_{}_{}_variable",
                                                self.id, self.map_id
                                            ))
                                            .button(
                                                ui,
                                                &mut self.modals.2,
                                                &mut page.condition.variable_id,
                                                info,
                                            );
                                        });

                                        ui.add_enabled(
                                            page.condition.variable_valid,
                                            egui::DragValue::new(
                                                &mut page.condition.variable_value,
                                            ),
                                        );
                                        ui.label("or above");
                                    });

                                    ui.horizontal(|ui| {
                                        ui.checkbox(
                                            &mut page.condition.self_switch_valid,
                                            "Self Switch",
                                        );
                                        ui.add_enabled_ui(page.condition.self_switch_valid, |ui| {
                                            egui::ComboBox::new(
                                                format!(
                                                    "event_{}_{}_self_switch_combo",
                                                    self.id, self.map_id
                                                ),
                                                "is on",
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

                                ui.horizontal(|ui| {
                                    ui.vertical(|ui| {
                                        ui.label("Options");
                                        ui.group(|ui| {
                                            ui.checkbox(&mut page.step_anime, "Move Animation");
                                            ui.checkbox(&mut page.walk_anime, "Stop Animation");
                                            ui.checkbox(&mut page.direction_fix, "Direction Fix");
                                            ui.checkbox(&mut page.through, "Through");
                                            ui.checkbox(&mut page.always_on_top, "Always on Top");
                                        });
                                    });

                                    ui.vertical(|ui| {
                                        ui.label("Trigger");
                                        ui.group(|ui| {
                                            ui.radio_value(&mut page.trigger, 0, "Action Button");
                                            ui.radio_value(&mut page.trigger, 1, "Player Touch");
                                            ui.radio_value(&mut page.trigger, 2, "Event Touch");
                                            ui.radio_value(&mut page.trigger, 3, "Autorun");
                                            ui.radio_value(
                                                &mut page.trigger,
                                                4,
                                                "Parallel Process",
                                            );
                                        });
                                    })
                                });
                            });
                        });
                    }
                    1 => {
                        if self.page_graphics_promise.ready().is_some() {
                            let graphics = self.page_graphics_promise.ready().unwrap();

                            let space =
                                ui.available_size_before_wrap() - ui.spacing().button_padding * 2.;

                            if if page.graphic.tile_id.is_positive() {
                                let ele = page.graphic.tile_id - 384;

                                let tile_width = 32. / graphics.1.width() as f32;
                                let tile_height = 32. / graphics.1.height() as f32;

                                let tile_x =
                                    (ele as usize % (graphics.1.width() / 32)) as f32 * tile_width;
                                let tile_y =
                                    (ele as usize / (graphics.1.width() / 32)) as f32 * tile_height;

                                let uv = egui::Rect::from_min_size(
                                    egui::pos2(tile_x, tile_y),
                                    egui::vec2(tile_width, tile_height),
                                );

                                ui.add(
                                    egui::ImageButton::new(
                                        graphics.1.texture_id(ui.ctx()),
                                        egui::vec2(space.x, space.x),
                                    )
                                    .uv(uv),
                                )
                            } else if let Some(ref tex) = graphics.0[self.selected_page] {
                                let cw = (tex.width() / 4) as f32;
                                let ch = (tex.height() / 4) as f32;

                                let cx = (page.graphic.pattern as f32 * cw) / tex.width() as f32;
                                let cy = (((page.graphic.direction - 2) / 2) as f32 * ch)
                                    / tex.height() as f32;

                                let uv = egui::Rect::from_min_size(
                                    egui::pos2(cx, cy),
                                    egui::vec2(cw / tex.width() as f32, ch / tex.height() as f32),
                                );

                                ui.add(
                                    egui::ImageButton::new(
                                        tex.texture_id(ui.ctx()),
                                        egui::vec2(space.x, ch * (space.x / cw)),
                                    )
                                    .uv(uv),
                                )
                            } else {
                                ui.button("Add image")
                            }
                            .clicked()
                            {
                                // TODO: Use modals for an image picker
                            }
                        } else {
                            ui.centered_and_justified(|ui| ui.spinner());
                        }
                    }
                    2 => {
                        ui.vertical(|ui| {
                            ui.group(|ui| {
                                egui::ScrollArea::both()
                                    .max_height(500.)
                                    .auto_shrink([false; 2])
                                    .show(ui, |ui| {
                                        CommandView::new(
                                            &mut page.list,
                                            &format!(
                                                "map_event_{}_{}_page_{}",
                                                self.id, self.map_id, self.selected_page
                                            ),
                                            Some(self.map_id),
                                        )
                                        .ui(ui, info);
                                    });
                            });
                        });
                    }
                    _ => unreachable!(),
                }

                ui.separator();

                ui.horizontal(|ui| {
                    let ok_clicked = ui.button("Ok").clicked();
                    let apply_clicked = ui.button("Apply").clicked();
                    let cancel_clicked = ui.button("Cancel").clicked();

                    if apply_clicked || ok_clicked {
                        let mut map = info.data_cache.get_map(self.map_id);
                        map.events[self.id] = self.event.clone();
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
