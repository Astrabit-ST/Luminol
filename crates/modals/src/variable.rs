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

use luminol_components::UiExt;

// TODO generalize modal into id modal
pub struct Modal {
    state: State,
    id: egui::Id,
}

enum State {
    Closed,
    Open {
        search_text: String,
        variable_id: usize,
    },
}

impl Modal {
    pub fn new(id: egui::Id) -> Self {
        Self {
            state: State::Closed,
            id,
        }
    }
}

impl luminol_core::Modal for Modal {
    type Data = usize;

    fn button<'m>(
        &'m mut self,
        data: &'m mut Self::Data,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget {
        move |ui: &mut egui::Ui| {
            let button_text = if ui.is_enabled() {
                let system = update_state.data.system();
                *data = system.variables.len().min(*data);
                format!("{:0>3}: {}", *data + 1, system.variables[*data])
            } else {
                "...".to_string()
            };
            let button_response = ui.button(button_text);

            if button_response.clicked() {
                self.state = State::Open {
                    search_text: "".to_string(),
                    variable_id: *data,
                };
            }
            if ui.is_enabled() {
                self.show_window(ui.ctx(), data, update_state);
            }

            button_response
        }
    }

    fn reset(&mut self) {
        self.state = State::Closed;
    }
}

impl Modal {
    fn show_window(
        &mut self,
        ctx: &egui::Context,
        data: &mut usize,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut win_open = true;
        let mut keep_open = true;
        let mut needs_save = false;

        let State::Open {
            search_text,
            variable_id,
        } = &mut self.state
        else {
            return;
        };

        egui::Window::new("Variable Picker")
            .resizable(false)
            .open(&mut win_open)
            .id(self.id)
            .show(ctx, |ui| {
                let system = update_state.data.system();

                let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show(ui, |ui| {
                            for (id, name) in system.variables.iter().enumerate() {
                                let text = format!("{:0>3}: {name}", id + 1);
                                if matcher.fuzzy(&text, search_text, false).is_none() {
                                    continue;
                                }

                                ui.with_stripe(id % 2 == 0, |ui| {
                                    let response = ui.selectable_value(variable_id, id, text);
                                    if response.double_clicked() {
                                        keep_open = false;
                                        needs_save = true;
                                    }
                                });
                            }
                        })
                });

                ui.horizontal(|ui| {
                    luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);

                    egui::TextEdit::singleline(search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                });
            });

        if needs_save {
            *data = *variable_id;
        }

        if !win_open || !keep_open {
            self.state = State::Closed;
        }
    }
}
