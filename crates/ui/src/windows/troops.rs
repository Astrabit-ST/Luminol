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

use crate::components::{DatabaseView, Field, OptionalIdComboBox, TroopView, UiExt};
use luminol_graphics::troop::{TROOP_HEIGHT, TROOP_WIDTH};

const HISTORY_SIZE: usize = 50;

/// Database - Troops management window.
pub struct Window {
    selected_troop_name: Option<String>,

    previous_troop: Option<usize>,
    previous_enemy_id: usize,
    saved_selected_member_index: Option<usize>,
    drag_state: Option<DragState>,
    previous_x: Option<i32>,
    previous_y: Option<i32>,
    history: History,
    needs_update: Option<Update>,
    troop_view: TroopView,
    view: DatabaseView,
}

#[derive(Debug)]
struct Update {
    member_index: usize,
    rebuild: bool,
}

#[derive(Debug)]
struct DragState {
    member_index: usize,
    original_x: i32,
    original_y: i32,
}

#[derive(Debug, Default)]
struct History(luminol_data::OptionVec<HistoryInner>);

#[derive(Debug, Default)]
struct HistoryInner {
    undo: std::collections::VecDeque<HistoryEntry>,
    redo: Vec<HistoryEntry>,
}

impl History {
    fn inner(&mut self, troop_index: usize) -> &mut HistoryInner {
        if !self.0.contains(troop_index) {
            self.0.insert(troop_index, Default::default());
        }
        self.0.get_mut(troop_index).unwrap()
    }

    fn remove_troop(&mut self, troop_index: usize) {
        let _ = self.0.try_remove(troop_index);
    }

    fn push(&mut self, troop_index: usize, entry: HistoryEntry) {
        let inner = self.inner(troop_index);
        inner.redo.clear();
        while inner.undo.len() >= HISTORY_SIZE {
            inner.undo.pop_front();
        }
        inner.undo.push_back(entry);
    }

    fn undo(&mut self, troop: &mut luminol_data::rpg::Troop) -> Option<Update> {
        let inner = self.inner(troop.id);
        let mut entry = inner.undo.pop_back()?;
        let state = entry.apply(troop);
        inner.redo.push(entry);
        Some(state)
    }

    fn redo(&mut self, troop: &mut luminol_data::rpg::Troop) -> Option<Update> {
        let inner = self.inner(troop.id);
        let mut entry = inner.redo.pop()?;
        let state = entry.apply(troop);
        inner.undo.push_back(entry);
        Some(state)
    }
}

#[derive(Debug, Default)]
struct HistoryEntry {
    member_index: usize,
    enemy_id: Option<Option<usize>>,
    x: i32,
    y: i32,
    hidden: bool,
    immortal: bool,
}

impl HistoryEntry {
    fn apply(&mut self, troop: &mut luminol_data::rpg::Troop) -> Update {
        while troop.members.len() <= self.member_index {
            troop.members.push(Default::default());
        }
        let member = &mut troop.members[self.member_index];
        if let Some(enemy_id) = &mut self.enemy_id {
            std::mem::swap(enemy_id, &mut member.enemy_id.0);
        }
        std::mem::swap(&mut self.x, &mut member.x);
        std::mem::swap(&mut self.y, &mut member.y);
        std::mem::swap(&mut self.hidden, &mut member.hidden);
        std::mem::swap(&mut self.immortal, &mut member.immortal);
        while troop
            .members
            .last()
            .is_some_and(|member| member.enemy_id.0.is_none())
        {
            troop.members.pop();
        }
        Update {
            member_index: self.member_index,
            rebuild: self.enemy_id.is_some(),
        }
    }
}

impl Window {
    pub fn new(update_state: &luminol_core::UpdateState<'_>) -> Self {
        Self {
            selected_troop_name: None,
            previous_troop: None,
            previous_enemy_id: 0,
            saved_selected_member_index: None,
            drag_state: None,
            previous_x: None,
            previous_y: None,
            history: Default::default(),
            needs_update: None,
            troop_view: TroopView::new(update_state),
            view: DatabaseView::new(),
        }
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("troop_editor")
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
        let mut troops = data.troops();
        let troops_len = troops.data.len();
        let enemies = data.enemies();

        let mut modified = false;

        self.selected_troop_name = None;

        let name = if let Some(name) = &self.selected_troop_name {
            format!("Editing troop {:?}", name)
        } else {
            "Troop Editor".into()
        };

        let response = egui::Window::new(name)
            .id(self.id())
            .default_width(720.)
            .open(open)
            .show(ctx, |ui| {
                self.view.show(
                    ui,
                    update_state,
                    "Troops",
                    &mut troops.data,
                    |troop| format!("{:0>4}: {}", troop.id + 1, troop.name),
                    |ui, troops, id, update_state| {
                        for i in troops.len()..troops_len {
                            self.history.remove_troop(i);
                        }

                        let troop = &mut troops[id];
                        self.selected_troop_name = Some(troop.name.clone());

                        let clip_rect = ui.clip_rect();

                        ui.with_padded_stripe(false, |ui| {
                            modified |= ui
                                .add(Field::new(
                                    "Name",
                                    egui::TextEdit::singleline(&mut troop.name)
                                        .desired_width(f32::INFINITY),
                                ))
                                .changed();
                        });

                        ui.with_padded_stripe(true, |ui| {
                            ui.add(Field::new(
                                "Editor Scale",
                                egui::Slider::new(&mut self.troop_view.scale, 15.0..=300.0)
                                    .suffix("%")
                                    .logarithmic(true)
                                    .fixed_decimals(0),
                            ));

                            let canvas_rect = egui::Resize::default()
                                .resizable([false, true])
                                .min_width(ui.available_width())
                                .max_width(ui.available_width())
                                .default_height(240.)
                                .show(ui, |ui| {
                                    egui::Frame::dark_canvas(ui.style())
                                        .show(ui, |ui| {
                                            let (_, rect) = ui.allocate_space(ui.available_size());
                                            rect
                                        })
                                        .inner
                                });

                            if self.previous_troop != Some(troop.id) {
                                self.troop_view.troop.rebuild_all_members(
                                    &update_state.graphics,
                                    update_state.filesystem,
                                    &enemies,
                                    troop,
                                );
                            }

                            if self.troop_view.selected_member_index.is_some_and(|i| {
                                i >= troop.members.len() || troop.members[i].enemy_id.0.is_none()
                            }) {
                                self.troop_view.selected_member_index = None;
                            }

                            if self.troop_view.selected_member_index.is_none()
                                && self.saved_selected_member_index.is_some_and(|i| {
                                    i < troop.members.len() && troop.members[i].enemy_id.0.is_some()
                                })
                            {
                                self.troop_view.selected_member_index =
                                    self.saved_selected_member_index;
                            }

                            if self.troop_view.hovered_member_index.is_some_and(|i| {
                                i >= troop.members.len() || troop.members[i].enemy_id.0.is_none()
                            }) {
                                self.troop_view.hovered_member_index = None;
                                self.troop_view.hovered_member_drag_pos = None;
                                self.troop_view.hovered_member_drag_offset = None;
                            }

                            // Handle dragging of members to move them
                            if let (Some(i), Some(drag_pos)) = (
                                self.troop_view.hovered_member_index,
                                self.troop_view.hovered_member_drag_pos,
                            ) {
                                if (troop.members[i].x, troop.members[i].y) != drag_pos {
                                    if self
                                        .drag_state
                                        .as_ref()
                                        .is_none_or(|drag_state| drag_state.member_index != i)
                                    {
                                        self.drag_state = Some(DragState {
                                            member_index: i,
                                            original_x: troop.members[i].x,
                                            original_y: troop.members[i].y,
                                        });
                                    }
                                    (troop.members[i].x, troop.members[i].y) = drag_pos;
                                    self.troop_view.troop.update_member(
                                        &update_state.graphics,
                                        troop,
                                        i,
                                    );
                                    modified = true;
                                }
                            } else if let Some(drag_state) = self.drag_state.take() {
                                self.history.push(
                                    troop.id,
                                    HistoryEntry {
                                        member_index: drag_state.member_index,
                                        enemy_id: None,
                                        x: drag_state.original_x,
                                        y: drag_state.original_y,
                                        hidden: troop.members[drag_state.member_index].hidden,
                                        immortal: troop.members[drag_state.member_index].immortal,
                                    },
                                );
                            }

                            egui::Frame::NONE.show(ui, |ui| {
                                if let Some(i) = self.troop_view.selected_member_index {
                                    let mut properties_modified = false;
                                    let mut properties_need_update = false;

                                    ui.label(format!("Member {}", i + 1));

                                    let original_x =
                                        self.previous_x.unwrap_or_else(|| troop.members[i].x);
                                    let original_y =
                                        self.previous_y.unwrap_or_else(|| troop.members[i].y);
                                    let history_entry = HistoryEntry {
                                        member_index: i,
                                        enemy_id: None,
                                        x: original_x,
                                        y: original_y,
                                        hidden: troop.members[i].hidden,
                                        immortal: troop.members[i].hidden,
                                    };

                                    let old_enemy_id = troop.members[i].enemy_id.0;
                                    let changed = ui
                                        .add(Field::new(
                                            "Enemy Type",
                                            OptionalIdComboBox::new(
                                                update_state,
                                                (troop.id, i, "enemy_id"),
                                                &mut troop.members[i].enemy_id.0,
                                                0..enemies.data.len(),
                                                |id| {
                                                    enemies.data.get(id).map_or_else(
                                                        || "".into(),
                                                        |e| format!("{:0>4}: {}", id + 1, e.name),
                                                    )
                                                },
                                            )
                                            .allow_none(false),
                                        ))
                                        .changed();
                                    if changed {
                                        self.history.push(
                                            troop.id,
                                            HistoryEntry {
                                                enemy_id: Some(old_enemy_id),
                                                ..history_entry
                                            },
                                        );
                                        self.troop_view.troop.rebuild_member(
                                            &update_state.graphics,
                                            update_state.filesystem,
                                            &enemies,
                                            troop,
                                            i,
                                        );
                                    }

                                    ui.columns(4, |columns| {
                                        columns[0].add(Field::new("X", |ui: &mut egui::Ui| {
                                            let response =
                                                egui::DragValue::new(&mut troop.members[i].x)
                                                    .range(0..=TROOP_WIDTH)
                                                    .update_while_editing(false)
                                                    .ui(ui);
                                            let mut changed = response.changed();
                                            if response.dragged() {
                                                changed = false;
                                                if self.previous_x.is_none() {
                                                    self.previous_x = Some(original_x);
                                                }
                                                properties_need_update = true;
                                            } else if self.previous_x.is_some() {
                                                self.previous_x = None;
                                                changed = true;
                                            }
                                            properties_modified |= changed;
                                            response
                                        }));

                                        columns[1].add(Field::new("Y", |ui: &mut egui::Ui| {
                                            let response =
                                                egui::DragValue::new(&mut troop.members[i].y)
                                                    .range(0..=TROOP_HEIGHT)
                                                    .update_while_editing(false)
                                                    .ui(ui);
                                            let mut changed = response.changed();
                                            if response.dragged() {
                                                changed = false;
                                                if self.previous_y.is_none() {
                                                    self.previous_y = Some(original_y);
                                                }
                                                properties_need_update = true;
                                            } else if self.previous_y.is_some() {
                                                self.previous_y = None;
                                                changed = true;
                                            }
                                            properties_modified |= changed;
                                            response
                                        }));

                                        properties_modified |= columns[2]
                                            .add(Field::new(
                                                "Hidden",
                                                egui::Checkbox::without_text(
                                                    &mut troop.members[i].hidden,
                                                ),
                                            ))
                                            .changed();

                                        modified |= columns[3]
                                            .add(Field::new(
                                                "Immortal",
                                                egui::Checkbox::without_text(
                                                    &mut troop.members[i].immortal,
                                                ),
                                            ))
                                            .changed();
                                    });

                                    if properties_modified || properties_need_update {
                                        self.troop_view.troop.update_member(
                                            &update_state.graphics,
                                            troop,
                                            i,
                                        );
                                    }
                                    if properties_modified {
                                        self.history.push(troop.id, history_entry);
                                        modified = true;
                                    }
                                }
                            });

                            if let Some(enemy_id) = self
                                .troop_view
                                .selected_member_index
                                .and_then(|i| troop.members.get(i))
                                .and_then(|member| member.enemy_id.0)
                            {
                                self.previous_enemy_id = enemy_id;
                            }

                            if let Some(update) = self.needs_update.take() {
                                if update.rebuild {
                                    self.troop_view.troop.rebuild_member(
                                        &update_state.graphics,
                                        update_state.filesystem,
                                        &enemies,
                                        troop,
                                        update.member_index,
                                    );
                                } else {
                                    self.troop_view.troop.update_member(
                                        &update_state.graphics,
                                        troop,
                                        update.member_index,
                                    );
                                }
                            }

                            ui.scope_builder(
                                egui::UiBuilder {
                                    max_rect: Some(canvas_rect),
                                    ..Default::default()
                                },
                                |ui| {
                                    let egui::InnerResponse {
                                        inner: hover_pos,
                                        response,
                                    } = self.troop_view.ui(ui, update_state, clip_rect);
                                    if response.clicked() {
                                        self.saved_selected_member_index =
                                            self.troop_view.selected_member_index;
                                    }

                                    // If the pointer is hovering over the troop view, prevent parent widgets
                                    // from receiving scroll events so that scaling the troop view with the
                                    // scroll wheel doesn't also scroll the scroll area that the troop view is
                                    // in
                                    if response.hovered() {
                                        ui.ctx().input_mut(|i| {
                                            i.smooth_scroll_delta = egui::Vec2::ZERO
                                        });
                                    }

                                    // Create new member on double click
                                    if let Some((x, y)) = hover_pos {
                                        if response.double_clicked() && !enemies.data.is_empty() {
                                            while troop
                                                .members
                                                .last()
                                                .is_some_and(|member| member.enemy_id.0.is_none())
                                            {
                                                troop.members.pop();
                                            }
                                            let next_member_index = troop.members.len();
                                            self.history.push(
                                                troop.id,
                                                HistoryEntry {
                                                    member_index: next_member_index,
                                                    enemy_id: Some(None),
                                                    ..Default::default()
                                                },
                                            );
                                            troop.members.push(luminol_data::rpg::troop::Member {
                                                enemy_id: Some(
                                                    self.previous_enemy_id
                                                        .min(enemies.data.len() - 1),
                                                )
                                                .into(),
                                                x,
                                                y,
                                                hidden: false,
                                                immortal: false,
                                            });
                                            self.needs_update = Some(Update {
                                                member_index: next_member_index,
                                                rebuild: true,
                                            });
                                            self.troop_view.selected_member_index =
                                                Some(next_member_index);
                                            modified = true;
                                        }
                                    }

                                    // Handle pressing delete or backspace to delete troops
                                    if let Some(i) = self.troop_view.selected_member_index {
                                        if i < troop.members.len()
                                            && troop.members[i].enemy_id.0.is_some()
                                            && response.has_focus()
                                            && ui.input(|i| {
                                                i.key_pressed(egui::Key::Delete)
                                                    || i.key_pressed(egui::Key::Backspace)
                                            })
                                        {
                                            let member = std::mem::take(&mut troop.members[i]);
                                            self.history.push(
                                                troop.id,
                                                HistoryEntry {
                                                    member_index: i,
                                                    enemy_id: Some(member.enemy_id.0),
                                                    x: member.x,
                                                    y: member.y,
                                                    hidden: member.hidden,
                                                    immortal: member.immortal,
                                                },
                                            );
                                            while troop
                                                .members
                                                .last()
                                                .is_some_and(|member| member.enemy_id.0.is_none())
                                            {
                                                troop.members.pop();
                                            }
                                            self.needs_update = Some(Update {
                                                member_index: i,
                                                rebuild: true,
                                            });
                                            self.troop_view.selected_member_index = None;
                                            modified = true;
                                        }
                                    }

                                    if response.has_focus() {
                                        // Ctrl+Z for undo
                                        if ui.input(|i| {
                                            i.modifiers.command
                                                && !i.modifiers.shift
                                                && i.key_pressed(egui::Key::Z)
                                        }) {
                                            self.needs_update = self.history.undo(troop);
                                        }

                                        // Ctrl+Y or Ctrl+Shift+Z for redo
                                        if ui.input(|i| {
                                            i.modifiers.command
                                                && (i.key_pressed(egui::Key::Y)
                                                    || (i.modifiers.shift
                                                        && i.key_pressed(egui::Key::Z)))
                                        }) {
                                            self.needs_update = self.history.redo(troop);
                                        }
                                    }
                                },
                            );
                        });

                        self.previous_troop = Some(troop.id);
                    },
                )
            });

        if response.is_some_and(|ir| ir.inner.is_some_and(|ir| ir.inner.modified)) {
            modified = true;
        }

        if modified {
            update_state.modified.set(true);
            troops.modified = true;
        }

        drop(troops);
        drop(enemies);

        *update_state.data = data; // restore data
    }
}
