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

use crate::components::UiExt;
use luminol_core::Modal;

use crate::components::{Field, IdVecSelection};
use crate::modals::graphic_picker::label::Modal as LabelModal;
use crate::modals::sound_picker::Modal as SoundModal;

/// Database - System management window.
pub struct Window {
    windowskin_modal: LabelModal,
    title_modal: LabelModal,
    gameover_modal: LabelModal,
    battle_transition_modal: LabelModal,
    battle_bgm_modal: SoundModal,
    battle_end_me_modal: SoundModal,
    gameover_me_modal: SoundModal,
    cursor_se_modal: SoundModal,
    decision_se_modal: SoundModal,
    cancel_se_modal: SoundModal,
    buzzer_se_modal: SoundModal,
    equip_se_modal: SoundModal,
    shop_se_modal: SoundModal,
    save_se_modal: SoundModal,
    load_se_modal: SoundModal,
    battle_start_se_modal: SoundModal,
    escape_se_modal: SoundModal,
    actor_collapse_se_modal: SoundModal,
    enemy_collapse_se_modal: SoundModal,

    max_elements: Option<usize>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            windowskin_modal: LabelModal::new(
                "windowskin_modal".into(),
                "Graphics/Windowskins".into(),
            ),
            title_modal: LabelModal::new("title_modal".into(), "Graphics/Titles".into()),
            gameover_modal: LabelModal::new("gameover_modal".into(), "Graphics/Gameovers".into()),
            battle_transition_modal: LabelModal::new(
                "battle_transition_modal".into(),
                "Graphics/Transitions".into(),
            ),
            battle_bgm_modal: SoundModal::new(luminol_audio::Source::BGM, "battle_bgm_modal"),
            battle_end_me_modal: SoundModal::new(luminol_audio::Source::ME, "battle_end_me_modal"),
            gameover_me_modal: SoundModal::new(luminol_audio::Source::ME, "gameover_me_modal"),
            cursor_se_modal: SoundModal::new(luminol_audio::Source::SE, "cursor_se_modal"),
            decision_se_modal: SoundModal::new(luminol_audio::Source::SE, "decision_se_modal"),
            cancel_se_modal: SoundModal::new(luminol_audio::Source::SE, "cancel_se_modal"),
            buzzer_se_modal: SoundModal::new(luminol_audio::Source::SE, "buzzer_se_modal"),
            equip_se_modal: SoundModal::new(luminol_audio::Source::SE, "equip_se_modal"),
            shop_se_modal: SoundModal::new(luminol_audio::Source::SE, "shop_se_modal"),
            save_se_modal: SoundModal::new(luminol_audio::Source::SE, "save_se_modal"),
            load_se_modal: SoundModal::new(luminol_audio::Source::SE, "load_se_modal"),
            battle_start_se_modal: SoundModal::new(
                luminol_audio::Source::SE,
                "battle_start_se_modal",
            ),
            escape_se_modal: SoundModal::new(luminol_audio::Source::SE, "escape_se_modal"),
            actor_collapse_se_modal: SoundModal::new(
                luminol_audio::Source::SE,
                "actor_collapse_se_modal",
            ),
            enemy_collapse_se_modal: SoundModal::new(
                luminol_audio::Source::SE,
                "enemy_collapse_se_modal",
            ),

            max_elements: None,
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("system_editor")
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
        let data = std::mem::take(update_state.data); // take data to avoid borrow checker issues
        let mut system = data.system();
        let actors = data.actors();

        let mut modified = false;

        if self.max_elements.is_none() {
            self.max_elements = Some(system.elements.len().saturating_sub(1));
        }

        egui::Window::new("System Editor")
            .id(self.id())
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                ui.with_cross_justify(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.set_min_width(
                            2. * (ui.spacing().slider_width + ui.spacing().interact_size.x)
                                    + ui.spacing().indent
                                    + 12. // `egui::Frame::group` inner margins are hardcoded to 6
                                          // points on each side
                                    + 5. * ui.spacing().item_spacing.x,
                        );

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Windowskin",
                                    self.windowskin_modal
                                        .button(&mut system.windowskin_name, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Title Graphic",
                                    self.title_modal
                                        .button(&mut system.title_name, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Gameover Graphic",
                                    self.gameover_modal
                                        .button(&mut system.gameover_name, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Battle Transition",
                                    self.battle_transition_modal
                                        .button(&mut system.battle_transition, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Battle BGM",
                                    self.battle_bgm_modal
                                        .button(&mut system.battle_bgm, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Battle End ME",
                                    self.battle_end_me_modal
                                        .button(&mut system.battle_end_me, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Gameover ME",
                                    self.gameover_me_modal
                                        .button(&mut system.gameover_me, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Cursor SE",
                                    self.cursor_se_modal
                                        .button(&mut system.cursor_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Decision SE",
                                    self.decision_se_modal
                                        .button(&mut system.decision_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Cancel SE",
                                    self.cancel_se_modal
                                        .button(&mut system.cancel_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Buzzer SE",
                                    self.buzzer_se_modal
                                        .button(&mut system.buzzer_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Equip SE",
                                    self.equip_se_modal
                                        .button(&mut system.equip_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Shop SE",
                                    self.shop_se_modal.button(&mut system.shop_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Save SE",
                                    self.save_se_modal.button(&mut system.save_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Load SE",
                                    self.load_se_modal.button(&mut system.load_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Battle Start SE",
                                    self.battle_start_se_modal
                                        .button(&mut system.battle_start_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Escape SE",
                                    self.escape_se_modal
                                        .button(&mut system.escape_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Actor Collapse SE",
                                    self.actor_collapse_se_modal
                                        .button(&mut system.actor_collapse_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Enemy Collapse SE",
                                    self.enemy_collapse_se_modal
                                        .button(&mut system.enemy_collapse_se, update_state),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Currency",
                                        egui::TextEdit::singleline(&mut system.words.gold)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "HP",
                                        egui::TextEdit::singleline(&mut system.words.hp)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "SP",
                                        egui::TextEdit::singleline(&mut system.words.sp)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "STR",
                                        egui::TextEdit::singleline(&mut system.words.str)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "DEX",
                                        egui::TextEdit::singleline(&mut system.words.dex)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "AGI",
                                        egui::TextEdit::singleline(&mut system.words.agi)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "INT",
                                        egui::TextEdit::singleline(&mut system.words.int)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "ATK",
                                        egui::TextEdit::singleline(&mut system.words.atk)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "PDEF",
                                        egui::TextEdit::singleline(&mut system.words.pdef)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "MDEF",
                                        egui::TextEdit::singleline(&mut system.words.mdef)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Weapon",
                                        egui::TextEdit::singleline(&mut system.words.weapon)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "Shield",
                                        egui::TextEdit::singleline(&mut system.words.armor1)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Helmet",
                                        egui::TextEdit::singleline(&mut system.words.armor2)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "Body Armor",
                                        egui::TextEdit::singleline(&mut system.words.armor3)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Accessory",
                                        egui::TextEdit::singleline(&mut system.words.armor4)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "Attack",
                                        egui::TextEdit::singleline(&mut system.words.attack)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Skill",
                                        egui::TextEdit::singleline(&mut system.words.skill)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "Guard",
                                        egui::TextEdit::singleline(&mut system.words.guard)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(false, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Item",
                                        egui::TextEdit::singleline(&mut system.words.item)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();

                                modified |= columns[1]
                                    .add(Field::new(
                                        "Equipment",
                                        egui::TextEdit::singleline(&mut system.words.equip)
                                            .desired_width(f32::INFINITY),
                                    ))
                                    .changed();
                            });
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.columns(2, |columns| {
                                modified |= columns[0]
                                    .add(Field::new(
                                        "Initial Party Members",
                                        IdVecSelection::new(
                                            update_state,
                                            "party_members",
                                            &mut system.party_members,
                                            0..actors.data.len(),
                                            |id| {
                                                actors.data.get(id).map_or_else(
                                                    || "".into(),
                                                    |e| format!("{:0>4}: {}", id + 1, e.name),
                                                )
                                            },
                                        ),
                                    ))
                                    .changed();

                                columns[1].add(Field::new("Elements", |ui: &mut egui::Ui| {
                                    ui.group(|ui| {
                                        ui.with_cross_justify(|ui| {
                                            ui.set_width(ui.available_width());

                                            ui.add_space(ui.spacing().item_spacing.y);

                                            let button_height = ui.spacing().interact_size.y.max(
                                                ui.text_style_height(&egui::TextStyle::Button)
                                                    + 2. * ui.spacing().button_padding.y,
                                            );
                                            egui::ScrollArea::vertical()
                                                .min_scrolled_height(200.)
                                                .show_rows(
                                                    ui,
                                                    button_height,
                                                    system.elements.len().saturating_sub(1),
                                                    |ui, range| {
                                                        let mut is_faint = range.start % 2 != 0;

                                                        for (i, element) in system.elements
                                                            [range.start + 1..range.end + 1]
                                                            .iter_mut()
                                                            .enumerate()
                                                        {
                                                            ui.with_stripe(is_faint, |ui| {
                                                                ui.style_mut().wrap_mode = Some(
                                                                    egui::TextWrapMode::Truncate,
                                                                );

                                                                ui.horizontal(|ui| {
                                                                    ui.label(format!(
                                                                        "{:0>4}:",
                                                                        range.start + i + 1
                                                                    ));
                                                                    modified |= ui
                                                                    .add(
                                                                        egui::TextEdit::singleline(
                                                                            element,
                                                                        )
                                                                        .desired_width(
                                                                            f32::INFINITY,
                                                                        ),
                                                                    )
                                                                    .changed()
                                                                });
                                                            });
                                                            is_faint = !is_faint;
                                                        }
                                                    },
                                                );
                                        });

                                        if system.elements.len().saturating_sub(1) <= 999
                                            && self.max_elements.is_some_and(|m| m > 999)
                                        {
                                            egui::Frame::none().show(ui, |ui| {
                                                ui.style_mut()
                                                    .visuals
                                                    .widgets
                                                    .noninteractive
                                                    .bg_stroke
                                                    .color = ui.style().visuals.warn_fg_color;
                                                egui::Frame::group(ui.style())
                                                    .fill(ui.visuals().gray_out(
                                                        ui.visuals().gray_out(
                                                            ui.visuals().gray_out(
                                                                ui.style().visuals.warn_fg_color,
                                                            ),
                                                        ),
                                                    ))
                                                    .show(ui, |ui| {
                                                        ui.set_width(ui.available_width());
                                                        ui.label(
                                                            egui::RichText::new(
                                                                "Setting the maximum above 999 may introduce performance issues and instability",
                                                            )
                                                            .color(
                                                                ui.style().visuals.warn_fg_color,
                                                            )
                                                        );
                                                    });
                                            });
                                        }

                                        ui.horizontal(|ui| {
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Truncate);

                                            ui.add(
                                                egui::DragValue::new(
                                                    self.max_elements.as_mut().unwrap(),
                                                )
                                                .range(0..=usize::MAX - 1),
                                            );

                                            if ui
                                                .add_enabled(
                                                    self.max_elements
                                                        != Some(
                                                            system.elements.len().saturating_sub(1),
                                                        ),
                                                    egui::Button::new("Set Maximum"),
                                                )
                                                .clicked()
                                            {
                                                modified = true;
                                                system.elements.resize(
                                                    self.max_elements.unwrap() + 1,
                                                    Default::default(),
                                                );
                                            };
                                        });
                                    })
                                    .response
                                }));
                            });
                        });
                    });
                });
            });

        if modified {
            update_state.modified.set(true);
            system.modified = true;
        }

        drop(actors);
        drop(system);

        *update_state.data = data; // restore data
    }
}
