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

use itertools::Itertools;
use luminol_components::UiExt;

use luminol_core::Modal;
use luminol_data::rpg::armor::Kind;
use luminol_modals::graphic_picker::actor::Modal as GraphicPicker;

pub struct Window {
    selected_actor_name: Option<String>,
    previous_actor: Option<usize>,

    graphic_picker: GraphicPicker,

    exp_view_is_total: bool,
    exp_view_is_depersisted: bool,

    view: luminol_components::DatabaseView,
}

impl Window {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        let actors = update_state.data.actors();
        let actor = &actors.data[0];
        Self {
            selected_actor_name: None,
            previous_actor: None,

            graphic_picker: GraphicPicker::new(
                update_state,
                "Graphics/Characters".into(),
                actor.character_name.as_deref(),
                actor.character_hue,
                egui::vec2(64., 96.),
                "actor_graphic_picker",
            ),

            exp_view_is_depersisted: false,
            exp_view_is_total: false,

            view: luminol_components::DatabaseView::new(),
        }
    }
}

fn draw_graph(
    ui: &mut egui::Ui,
    actor: &luminol_data::rpg::Actor,
    param: usize,
    range: std::ops::RangeInclusive<usize>,
    color: egui::Color32,
) -> egui::Response {
    egui::Frame::canvas(ui.style())
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height((ui.available_width() * 9.) / 16.);
            let rect = ui.max_rect();
            let clip_rect = ui.clip_rect().intersect(rect);
            if clip_rect.height() == 0. || clip_rect.width() == 0. {
                return;
            }
            ui.set_clip_rect(clip_rect);

            let iter = (1..actor.parameters.ysize()).map(|i| {
                rect.left_top()
                    + egui::vec2(
                        ((i - 1) as f32 / (actor.parameters.ysize() - 2) as f32) * rect.width(),
                        ((range
                            .end()
                            .saturating_sub(actor.parameters[(param, i)] as usize))
                            as f32
                            / range.end().saturating_sub(*range.start()) as f32)
                            * rect.height(),
                    )
            });

            // Draw the filled part of the graph by drawing a trapezoid for each area horizontally
            // between two points
            let ppp = ui.ctx().pixels_per_point();
            ui.painter()
                .extend(
                    iter.clone()
                        .tuple_windows()
                        .with_position()
                        .map(|(iter_pos, (p, q))| {
                            // Round the horizontal position of each point to the nearest pixel so egui doesn't
                            // try to anti-alias the vertical edges of the trapezoids
                            let p = if iter_pos == itertools::Position::First {
                                p
                            } else {
                                egui::pos2((p.x * ppp).round() / ppp, p.y)
                            };
                            let q = if iter_pos == itertools::Position::Last {
                                q
                            } else {
                                egui::pos2((q.x * ppp).round() / ppp, q.y)
                            };

                            egui::Shape::convex_polygon(
                                vec![
                                    p,
                                    q,
                                    egui::pos2(q.x, rect.bottom()),
                                    egui::pos2(p.x, rect.bottom()),
                                ],
                                color.gamma_multiply(0.25),
                                egui::Stroke::NONE,
                            )
                        }),
                );

            // Draw the border of the graph
            ui.painter().add(egui::Shape::line(
                iter.collect_vec(),
                egui::Stroke { width: 2., color },
            ));
        })
        .response
}

fn draw_exp(ui: &mut egui::Ui, actor: &luminol_data::rpg::Actor, total: &mut bool) {
    let mut exp = [0f64; 99];

    let p = actor.exp_inflation as f64 / 100. + 2.4;
    for i in 1..99.min(actor.final_level as usize) {
        exp[i] = exp[i - 1] + (actor.exp_basis as f64 * (((i + 4) as f64 / 5.).powf(p))).trunc();
    }

    ui.columns(2, |columns| {
        if columns[0]
            .selectable_label(!*total, "To Next Level")
            .clicked()
        {
            *total = false;
        }
        if columns[1].selectable_label(*total, "Total").clicked() {
            *total = true;
        }
    });

    ui.group(|ui| {
        egui::ScrollArea::vertical()
            .min_scrolled_height(200.)
            .show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Body) + ui.spacing().item_spacing.y,
                (actor.final_level - actor.initial_level + 1).clamp(0, 99) as usize,
                |ui, range| {
                    ui.set_width(ui.available_width());

                    for (pos, i) in range.with_position() {
                        ui.with_padded_stripe(i % 2 != 0, |ui| {
                            let i = i + actor.initial_level.max(1) as usize - 1;

                            ui.horizontal(|ui| {
                                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);

                                ui.with_layout(
                                    egui::Layout {
                                        main_dir: egui::Direction::RightToLeft,
                                        ..*ui.layout()
                                    },
                                    |ui| {
                                        ui.add(
                                            egui::Label::new(if *total {
                                                exp[i].to_string()
                                            } else if matches!(
                                                pos,
                                                itertools::Position::Last
                                                    | itertools::Position::Only
                                            ) {
                                                "(None)".into()
                                            } else {
                                                (exp[i + 1] - exp[i]).to_string()
                                            })
                                            .truncate(),
                                        );

                                        ui.with_layout(
                                            egui::Layout {
                                                main_dir: egui::Direction::LeftToRight,
                                                ..*ui.layout()
                                            },
                                            |ui| {
                                                ui.add(
                                                    egui::Label::new(format!("Level {}", i + 1))
                                                        .truncate(),
                                                );
                                            },
                                        );
                                    },
                                );
                            });
                        });
                    }
                },
            );
    });
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("actor_editor")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        // we take data temporarily to avoid borrowing issues
        // we could probably avoid this with Rc (Data already uses RefCell) but it'd be annoying to work into the existing code
        // using Box<Data> might be a good idea as well, that's just a pointer copy rather than a full copy
        let data = std::mem::take(update_state.data);
        let mut actors = data.actors();
        let mut classes = data.classes();
        let weapons = data.weapons();
        let armors = data.armors();

        let mut modified = false;

        self.selected_actor_name = None;

        let name = if let Some(name) = &self.selected_actor_name {
            format!("Editing actor {:?}", name)
        } else {
            "Actor Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Actors",
                    &mut actors.data,
                    |actor| format!("{:0>4}: {}", actor.id + 1, actor.name),
                    |ui, actors, id, update_state| {
                        let actor = &mut actors[id];
                        self.selected_actor_name = Some(actor.name.clone());

                        ui.with_padded_stripe(false, |ui| {
                            ui.horizontal(|ui| {
                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Icon",
                                        self.graphic_picker.button(
                                            (&mut actor.character_name, &mut actor.character_hue),
                                            update_state,
                                        ),
                                    ))
                                    .changed();
                                if self.previous_actor != Some(actor.id) {
                                    // avoid desyncs by resetting the modal if the item has changed
                                    self.graphic_picker.reset(
                                        update_state,
                                        (&mut actor.character_name, &mut actor.character_hue),
                                    );
                                }

                                modified |= ui
                                    .add(luminol_components::Field::new(
                                        "Name",
                                        egui::TextEdit::singleline(&mut actor.name)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            })
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(luminol_components::Field::new(
                                    "Class",
                                    luminol_components::OptionalIdComboBox::new(
                                        update_state,
                                        (actor.id, "class"),
                                        &mut actor.class_id,
                                        0..classes.data.len(),
                                        |id| {
                                            classes.data.get(id).map_or_else(
                                                || "".into(),
                                                |c| format!("{:0>4}: {}", id + 1, c.name),
                                            )
                                        },
                                    ),
                                ))
                                .changed();
                        });

                        if let Some(class) = classes.data.get_mut(actor.class_id) {
                            if !luminol_core::slice_is_sorted(&class.weapon_set) {
                                class.weapon_set.sort_unstable();
                            }
                            if !luminol_core::slice_is_sorted(&class.armor_set) {
                                class.armor_set.sort_unstable();
                            }
                        }
                        let class = classes.data.get(actor.class_id);

                        ui.with_padded_stripe(false, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Starting Weapon",
                                |ui: &mut egui::Ui| {
                                    egui::Frame::none()
                                        .show(ui, |ui| {
                                            ui.columns(2, |columns| {
                                                modified |= columns[0]
                                                    .add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            update_state,
                                                            (actor.id, "weapon_id"),
                                                            &mut actor.weapon_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| {
                                                                        c.weapon_set.iter().copied()
                                                                    },
                                                                )
                                                                .filter(|id| {
                                                                    (0..weapons.data.len())
                                                                        .contains(id)
                                                                }),
                                                            |id| {
                                                                weapons.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |w| {
                                                                        format!(
                                                                            "{:0>4}: {}",
                                                                            id + 1,
                                                                            w.name
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    )
                                                    .changed();
                                                modified |= columns[1]
                                                    .checkbox(&mut actor.weapon_fix, "Fixed")
                                                    .changed();
                                            });
                                        })
                                        .response
                                },
                            ));
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Starting Shield",
                                |ui: &mut egui::Ui| {
                                    egui::Frame::none()
                                        .show(ui, |ui| {
                                            ui.columns(2, |columns| {
                                                modified |= columns[0]
                                                    .add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            update_state,
                                                            (actor.id, "armor1_id"),
                                                            &mut actor.armor1_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Shield
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{:0>4}: {}",
                                                                            id + 1,
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    )
                                                    .changed();
                                                modified |= columns[1]
                                                    .checkbox(&mut actor.armor1_fix, "Fixed")
                                                    .changed();
                                            });
                                        })
                                        .response
                                },
                            ));
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Starting Helmet",
                                |ui: &mut egui::Ui| {
                                    egui::Frame::none()
                                        .show(ui, |ui| {
                                            ui.columns(2, |columns| {
                                                modified |= columns[0]
                                                    .add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            update_state,
                                                            (actor.id, "armor2_id"),
                                                            &mut actor.armor2_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Helmet
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{:0>4}: {}",
                                                                            id + 1,
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    )
                                                    .changed();
                                                modified |= columns[1]
                                                    .checkbox(&mut actor.armor2_fix, "Fixed")
                                                    .changed();
                                            });
                                        })
                                        .response
                                },
                            ));
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Starting Body Armor",
                                |ui: &mut egui::Ui| {
                                    egui::Frame::none()
                                        .show(ui, |ui| {
                                            ui.columns(2, |columns| {
                                                modified |= columns[0]
                                                    .add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            update_state,
                                                            (actor.id, "armor3_id"),
                                                            &mut actor.armor3_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::BodyArmor
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{:0>4}: {}",
                                                                            id + 1,
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    )
                                                    .changed();
                                                modified |= columns[1]
                                                    .checkbox(&mut actor.armor3_fix, "Fixed")
                                                    .changed();
                                            });
                                        })
                                        .response
                                },
                            ));
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.add(luminol_components::Field::new(
                                "Starting Accessory",
                                |ui: &mut egui::Ui| {
                                    egui::Frame::none()
                                        .show(ui, |ui| {
                                            ui.columns(2, |columns| {
                                                modified |= columns[0]
                                                    .add(
                                                        luminol_components::OptionalIdComboBox::new(
                                                            update_state,
                                                            (actor.id, "armor4_id"),
                                                            &mut actor.armor4_id,
                                                            class
                                                                .map_or_else(
                                                                    Default::default,
                                                                    |c| c.armor_set.iter().copied(),
                                                                )
                                                                .filter(|id| {
                                                                    (0..armors.data.len())
                                                                        .contains(id)
                                                                        && armors
                                                                            .data
                                                                            .get(*id)
                                                                            .is_some_and(|a| {
                                                                                matches!(
                                                                                    a.kind,
                                                                                    Kind::Accessory
                                                                                )
                                                                            })
                                                                }),
                                                            |id| {
                                                                armors.data.get(id).map_or_else(
                                                                    || "".into(),
                                                                    |a| {
                                                                        format!(
                                                                            "{:0>4}: {}",
                                                                            id + 1,
                                                                            a.name,
                                                                        )
                                                                    },
                                                                )
                                                            },
                                                        ),
                                                    )
                                                    .changed();
                                                modified |= columns[1]
                                                    .checkbox(&mut actor.armor4_fix, "Fixed")
                                                    .changed();
                                            });
                                        })
                                        .response
                                },
                            ));
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "Initial Level",
                                        egui::Slider::new(&mut actor.initial_level, 1..=99),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "Final Level",
                                        egui::Slider::new(
                                            &mut actor.final_level,
                                            actor.initial_level..=99,
                                        ),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            // Forget whether the collapsing header was open from the last time
                            // the editor was open
                            let ui_id = ui.make_persistent_id("exp_collapsing_header");
                            if !self.exp_view_is_depersisted {
                                self.exp_view_is_depersisted = true;
                                if let Some(h) =
                                    egui::collapsing_header::CollapsingState::load(ui.ctx(), ui_id)
                                {
                                    h.remove(ui.ctx());
                                }
                                ui.ctx().animate_bool_with_time(ui_id, false, 0.);
                            }

                            egui::collapsing_header::CollapsingState::load_with_default_open(
                                ui.ctx(),
                                ui_id,
                                false,
                            )
                            .show_header(ui, |ui| {
                                ui.with_cross_justify(|ui| {
                                    ui.label("EXP Curve");
                                });
                            })
                            .body(|ui| {
                                draw_exp(ui, actor, &mut self.exp_view_is_total);
                                ui.add_space(ui.spacing().item_spacing.y);
                            });

                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(luminol_components::Field::new(
                                        "EXP Curve Basis",
                                        egui::Slider::new(&mut actor.exp_basis, 10..=50),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(luminol_components::Field::new(
                                        "EXP Curve Inflation",
                                        egui::Slider::new(&mut actor.exp_inflation, 10..=50),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].add(luminol_components::Field::new(
                                    "Max HP",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            0,
                                            1..=9999,
                                            egui::Color32::from_rgb(204, 0, 0),
                                        )
                                    },
                                ));

                                columns[1].add(luminol_components::Field::new(
                                    "Max SP",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            1,
                                            1..=9999,
                                            egui::Color32::from_rgb(245, 123, 0),
                                        )
                                    },
                                ));
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].add(luminol_components::Field::new(
                                    "STR",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            2,
                                            1..=999,
                                            egui::Color32::from_rgb(237, 213, 0),
                                        )
                                    },
                                ));

                                columns[1].add(luminol_components::Field::new(
                                    "DEX",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            3,
                                            1..=999,
                                            egui::Color32::from_rgb(116, 210, 22),
                                        )
                                    },
                                ));
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                columns[0].add(luminol_components::Field::new(
                                    "AGI",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            4,
                                            1..=999,
                                            egui::Color32::from_rgb(52, 101, 164),
                                        )
                                    },
                                ));

                                columns[1].add(luminol_components::Field::new(
                                    "INT",
                                    |ui: &mut egui::Ui| {
                                        draw_graph(
                                            ui,
                                            actor,
                                            5,
                                            1..=999,
                                            egui::Color32::from_rgb(117, 80, 123),
                                        )
                                    },
                                ));
                            });
                        });

                        self.previous_actor = Some(actor.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            actors.modified = true;
        }

        // we have to drop things before we can restore data, because the compiler isn't smart enough to do that for us right now
        drop(actors);
        drop(classes);
        drop(weapons);
        drop(armors);

        *update_state.data = data;
    }
}
