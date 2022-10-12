use egui_extras::RetainedImage;
use poll_promise::Promise;

use super::window::Window;
use crate::data::rmxp_structs::rpg;
use crate::{load_image_software, UpdateInfo};

pub struct EventEdit {
    id: i32,
    map_id: i32,
    selected_page: usize,
    event: rpg::event::Event,
    page_graphics_promise: Promise<(Vec<Option<RetainedImage>>, RetainedImage)>,
}

// TODO: Use egui-modal

impl EventEdit {
    pub fn new(
        id: i32,
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
                    load_image_software(format!("Graphics/Tilesets/{}", tileset_name), info)
                        .await
                        .unwrap(),
                )
            }),
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
        let system = info.data_cache.system();
        let system = system.as_ref().unwrap();

        egui::Window::new(self.name())
            .id(egui::Id::new(format!("event_{}_{}", self.id, self.map_id)))
            .open(&mut win_open)
            .show(ctx, |ui| {
                if self.page_graphics_promise.ready().is_some() {
                    let graphics = self.page_graphics_promise.ready().unwrap();

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
                            ui.selectable_value(&mut self.selected_page, page, page.to_string());
                        }
                    });

                    let page = self.event.pages.get_mut(self.selected_page).unwrap();
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Condition");
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    // FIXME: Stop using comboboxes and use modals
                                    ui.checkbox(&mut page.condition.switch1_valid, "Switch");

                                    ui.add_enabled_ui(page.condition.switch1_valid, |ui| {
                                        egui::ComboBox::new(
                                            format!(
                                                "event_{}_{}_switch_1_combo",
                                                self.id, self.map_id
                                            ),
                                            "is on",
                                        )
                                        .selected_text(&system.switches[page.condition.switch1_id])
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                egui::ScrollArea::vertical().show_rows(
                                                    ui,
                                                    ui.text_style_height(&egui::TextStyle::Body),
                                                    system.switches.len(),
                                                    |ui, rows| {
                                                        for id in rows {
                                                            ui.selectable_value(
                                                                &mut page.condition.switch1_id,
                                                                id,
                                                                format!(
                                                                    "'{}': {}",
                                                                    system.switches[id], id
                                                                ),
                                                            );
                                                        }
                                                    },
                                                )
                                            },
                                        )
                                    });
                                });

                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut page.condition.switch2_valid, "Switch");

                                    ui.add_enabled_ui(page.condition.switch2_valid, |ui| {
                                        egui::ComboBox::new(
                                            format!(
                                                "event_{}_{}_switch_2_combo",
                                                self.id, self.map_id
                                            ),
                                            "is on",
                                        )
                                        .selected_text(&system.switches[page.condition.switch2_id])
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                egui::ScrollArea::vertical().show_rows(
                                                    ui,
                                                    ui.text_style_height(&egui::TextStyle::Body),
                                                    system.switches.len(),
                                                    |ui, rows| {
                                                        for id in rows {
                                                            ui.selectable_value(
                                                                &mut page.condition.switch2_id,
                                                                id,
                                                                format!(
                                                                    "'{}': {}",
                                                                    system.switches[id], id
                                                                ),
                                                            );
                                                        }
                                                    },
                                                )
                                            },
                                        )
                                    });
                                });

                                ui.horizontal(|ui| {
                                    ui.checkbox(&mut page.condition.variable_valid, "Variable");

                                    ui.add_enabled_ui(page.condition.variable_valid, |ui| {
                                        egui::ComboBox::new(
                                            format!(
                                                "event_{}_{}_variable_combo",
                                                self.id, self.map_id
                                            ),
                                            "is",
                                        )
                                        .selected_text(
                                            &system.variables[page.condition.variable_id],
                                        )
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                egui::ScrollArea::vertical().show_rows(
                                                    ui,
                                                    ui.text_style_height(&egui::TextStyle::Body),
                                                    system.variables.len(),
                                                    |ui, rows| {
                                                        for id in rows {
                                                            ui.selectable_value(
                                                                &mut page.condition.variable_id,
                                                                id,
                                                                format!(
                                                                    "'{}': {}",
                                                                    system.variables[id], id
                                                                ),
                                                            );
                                                        }
                                                    },
                                                )
                                            },
                                        )
                                    });

                                    ui.add_enabled(
                                        page.condition.variable_valid,
                                        egui::DragValue::new(&mut page.condition.variable_value),
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

                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label("Graphic");
                                    if if page.graphic.tile_id.is_positive() {
                                        let ele = page.graphic.tile_id - 384;

                                        let tile_width = 32. / graphics.1.width() as f32;
                                        let tile_height = 32. / graphics.1.height() as f32;

                                        let tile_x = (ele as usize % (graphics.1.width() / 32))
                                            as f32
                                            * tile_width;
                                        let tile_y = (ele as usize / (graphics.1.width() / 32))
                                            as f32
                                            * tile_height;

                                        let uv = egui::Rect::from_min_size(
                                            egui::pos2(tile_x, tile_y),
                                            egui::vec2(tile_width, tile_height),
                                        );

                                        ui.add(
                                            egui::ImageButton::new(
                                                graphics.1.texture_id(ui.ctx()),
                                                egui::vec2(32., 32.),
                                            )
                                            .uv(uv),
                                        )
                                    } else if let Some(ref tex) = graphics.0[self.selected_page] {
                                        let cw = (tex.width() / 4) as f32;
                                        let ch = (tex.height() / 4) as f32;

                                        let cx =
                                            (page.graphic.pattern as f32 * cw) / tex.width() as f32;
                                        let cy = (((page.graphic.direction - 2) / 2) as f32 * ch)
                                            / tex.height() as f32;

                                        let uv = egui::Rect::from_min_size(
                                            egui::pos2(cx, cy),
                                            egui::vec2(
                                                cw / tex.width() as f32,
                                                ch / tex.height() as f32,
                                            ),
                                        );

                                        ui.add(
                                            egui::ImageButton::new(
                                                tex.texture_id(ui.ctx()),
                                                egui::vec2(cw, ch),
                                            )
                                            .uv(uv),
                                        )
                                    } else {
                                        ui.button("Add image")
                                    }
                                    .clicked()
                                    {
                                        // TODO: Use modals and add an image picker
                                    }
                                });

                                ui.vertical(|ui| {
                                    // FIXME: These are wrong
                                    let move_types = ["Fixed", "Random", "Approach", "Custom"];
                                    let move_speeds =
                                        ["Very Slow", "Slow", "Normal", "Fast", "Very fast"];
                                    let move_freqs =
                                        ["Very Low", "Low", "Normal", "High", "Very High"];

                                    ui.label("Autonomous Movement");
                                    ui.group(|ui| {
                                        egui::ComboBox::new(
                                            format!("event_{}_{}_move_type", self.id, self.map_id),
                                            "Type",
                                        )
                                        .selected_text(move_types[page.move_type])
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for (id, name) in move_types.iter().enumerate() {
                                                    ui.selectable_value(
                                                        &mut page.move_type,
                                                        id,
                                                        *name,
                                                    );
                                                }
                                            },
                                        );

                                        ui.add_enabled_ui(page.move_type == 3, |ui| {
                                            if ui.button("Move route").clicked() {}
                                        });

                                        egui::ComboBox::new(
                                            format!("event_{}_{}_move_speed", self.id, self.map_id),
                                            "Speed",
                                        )
                                        .selected_text(move_speeds[page.move_speed])
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for (id, name) in move_speeds.iter().enumerate() {
                                                    ui.selectable_value(
                                                        &mut page.move_speed,
                                                        id,
                                                        *name,
                                                    );
                                                }
                                            },
                                        );

                                        egui::ComboBox::new(
                                            format!("event_{}_{}_move_freq", self.id, self.map_id),
                                            "Frequency",
                                        )
                                        .selected_text(move_freqs[page.move_frequency])
                                        .show_ui(
                                            ui,
                                            |ui| {
                                                for (id, name) in move_freqs.iter().enumerate() {
                                                    ui.selectable_value(
                                                        &mut page.move_frequency,
                                                        id,
                                                        *name,
                                                    );
                                                }
                                            },
                                        );
                                    });
                                })
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
                                        ui.radio_value(&mut page.trigger, 4, "Parallel Process");
                                    });
                                })
                            });
                        });
                        ui.vertical(|ui| {
                            ui.label("Event commands");
                            ui.group(|ui| {
                                //
                                ui.allocate_rect(ui.max_rect(), egui::Sense::click())
                            });
                        })
                    });

                    ui.separator();

                    ui.horizontal(|ui| {
                        let ok_clicked = ui.button("Ok").clicked();
                        let apply_clicked = ui.button("Apply").clicked();
                        let cancel_clicked = ui.button("Cancel").clicked();

                        if apply_clicked || ok_clicked {
                            let mut map = info.data_cache.get_map(self.map_id);
                            map.events.insert(self.id, self.event.clone());
                        }

                        if cancel_clicked || ok_clicked {
                            *open = false;
                        }
                    });
                } else {
                    ui.centered_and_justified(|ui| ui.spinner());
                }
            });
        *open = *open && win_open;
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
