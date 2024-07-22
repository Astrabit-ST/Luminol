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

use std::marker::PhantomData;

use luminol_components::UiExt;

mod variable;
pub use variable::Variable;
mod switch;
pub use switch::Switch;

pub struct Modal<M> {
    state: State,
    id: egui::Id,
    _phantom: PhantomData<M>, // so that M is constrained
}

pub type SwitchModal = Modal<Switch>;
pub type VariableModal = Modal<Variable>;

enum State {
    Closed,
    Open {
        search_text: String,
        selected_id: usize,
        new_size: Option<usize>,
    },
}

#[allow(unused_variables)]
pub trait DatabaseModalHandler {
    fn window_title() -> &'static str;

    fn button_format(id: &mut usize, update_state: &mut luminol_core::UpdateState<'_>) -> String;

    fn iter(
        update_state: &mut luminol_core::UpdateState<'_>,
        f: impl FnOnce(&mut dyn Iterator<Item = (usize, String)>), // can't figure out how to avoid the dyn
    );

    fn current_size(update_state: &luminol_core::UpdateState<'_>) -> Option<usize> {
        None
    }
    fn resize(update_state: &mut luminol_core::UpdateState<'_>, new_size: usize) {}
}

impl<M> Modal<M>
where
    M: DatabaseModalHandler,
{
    pub fn new(id: egui::Id) -> Self {
        Self {
            state: State::Closed,
            id,
            _phantom: PhantomData,
        }
    }
}

impl<M> luminol_core::Modal for Modal<M>
where
    M: DatabaseModalHandler,
{
    type Data<'m> = &'m mut usize;

    fn button<'m>(
        &'m mut self,
        data: Self::Data<'m>,
        update_state: &'m mut luminol_core::UpdateState<'_>,
    ) -> impl egui::Widget + 'm {
        move |ui: &mut egui::Ui| {
            let button_text = if ui.is_enabled() {
                M::button_format(data, update_state)
            } else {
                "...".to_string()
            };
            let button_response = ui.button(button_text);

            if button_response.clicked() {
                self.state = State::Open {
                    search_text: String::new(),
                    selected_id: *data,
                    new_size: M::current_size(update_state),
                };
            }
            if ui.is_enabled() {
                self.show_window(ui.ctx(), data, update_state);
            }

            button_response
        }
    }

    fn reset(&mut self, _update_state: &mut luminol_core::UpdateState<'_>, _data: Self::Data<'_>) {
        // not much internal state, so we dont need to do much here
        self.state = State::Closed;
    }
}

impl<M> Modal<M>
where
    M: DatabaseModalHandler,
{
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
            selected_id,
            new_size,
        } = &mut self.state
        else {
            return;
        };

        egui::Window::new(M::window_title())
            .resizable(true)
            .open(&mut win_open)
            .id(self.id)
            .show(ctx, |ui| {
                let matcher = fuzzy_matcher::skim::SkimMatcherV2::default();

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show(ui, |ui| {
                            let mut is_faint = false;
                            M::iter(update_state, |iter| {
                                for (id, text) in iter {
                                    if matcher.fuzzy(&text, search_text, false).is_none() {
                                        continue;
                                    }
                                    is_faint = !is_faint;

                                    ui.with_stripe(is_faint, |ui| {
                                        ui.horizontal(|ui| {
                                            let response =
                                                ui.selectable_value(selected_id, id, text);
                                            ui.add_space(ui.available_width());
                                            if response.double_clicked() {
                                                keep_open = false;
                                                needs_save = true;
                                            }
                                        });
                                    });
                                }
                            })
                        })
                });

                if M::current_size(update_state).is_some_and(|size| size <= 999) && new_size.is_some_and(|size| size > 999) {
                    egui::Frame::none().show(ui, |ui| {
                        ui.style_mut()
                            .visuals
                            .widgets
                            .noninteractive
                            .bg_stroke
                            .color = ui.style().visuals.warn_fg_color;
                        egui::Frame::group(ui.style())
                            .fill(ui.visuals().gray_out(ui.visuals().gray_out(
                                ui.visuals().gray_out(ui.style().visuals.warn_fg_color),
                            )))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(egui::RichText::new("Setting the maximum above 999 may introduce performance issues and instability").color(ui.style().visuals.warn_fg_color));
                            });
                    });
                }

                ui.horizontal(|ui| {
                    luminol_components::close_options_ui(ui, &mut keep_open, &mut needs_save);

                    if let Some(size) = new_size {
                        ui.add(egui::DragValue::new(size).range(1..=usize::MAX));
                        if ui.button("Set Maximum").clicked() {
                            M::resize(update_state, *size);
                        }
                    }

                    egui::TextEdit::singleline(search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                });
            });

        if needs_save {
            *data = *selected_id;
        }

        if !win_open || !keep_open {
            self.state = State::Closed;
        }
    }
}
