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

/// The switch picker modal.
pub struct Modal {
    switch_id_range: std::ops::Range<usize>,
    search_text: String,
}

impl luminol_core::Modal for Modal {
    type Data = usize;

    fn button(
        this: &mut Option<Self>,
        ui: &mut egui::Ui,
        data: &mut Self::Data,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let system = update_state.data.system();

        if ui
            .button(format!("{data}: {}", system.switches[*data - 1]))
            .clicked()
        {
            this.get_or_insert(Self {
                switch_id_range: *data..*data,
                search_text: String::new(),
            });
        }

        drop(system);

        Modal::show(this, ui.ctx(), data, update_state);
    }

    fn show(
        this_opt: &mut Option<Self>,
        ctx: &egui::Context,
        data: &mut Self::Data,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut win_open = this_opt.is_some();
        let mut needs_close = this_opt.is_some();

        egui::Window::new("Switch Picker")
            .resizable(false)
            .open(&mut win_open)
            .show(ctx, |ui| {
                let this = this_opt.as_mut().unwrap();

                let system = update_state.data.system();

                ui.group(|ui| {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .max_height(384.)
                        .show(ui, |ui| {
                            for (id, name) in
                                system.switches.iter().enumerate().filter(|(id, s)| {
                                    (id + 1).to_string().contains(&this.search_text)
                                        || s.contains(&this.search_text)
                                })
                            {
                                let id = id + 1;
                                let mut text = egui::RichText::new(format!("{id}: {name}"));

                                if this.switch_id_range.start == id {
                                    text = text.color(egui::Color32::YELLOW);
                                }

                                let response = ui.selectable_value(data, id, text);

                                if this.switch_id_range.end == id {
                                    this.switch_id_range.end = usize::MAX;
                                    this.switch_id_range.start = id;

                                    response.scroll_to_me(None);
                                }

                                if response.double_clicked() {
                                    needs_close = true;
                                }
                            }
                        })
                });

                ui.horizontal(|ui| {
                    needs_close |= ui.button("Ok").clicked();
                    needs_close |= ui.button("Cancel").clicked();

                    if ui
                        .add(
                            egui::DragValue::new(&mut this.switch_id_range.start)
                                .clamp_range(0..=system.switches.len()),
                        )
                        .changed()
                    {
                        this.switch_id_range.end = this.switch_id_range.start;
                    };
                    egui::TextEdit::singleline(&mut this.search_text)
                        .hint_text("Search 🔎")
                        .show(ui);
                });
            });

        if !win_open || needs_close {
            *this_opt = None;
        }
    }
}
