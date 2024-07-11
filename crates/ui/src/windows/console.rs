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

pub struct Window {
    term: luminol_term::widget::ProcessTerminal,
}

impl Window {
    pub fn new(
        exec: luminol_term::widget::ExecOptions,
        update_state: &luminol_core::UpdateState<'_>,
    ) -> std::io::Result<Self> {
        Ok(Self {
            // TODO
            term: luminol_term::widget::Terminal::process(exec, update_state)?,
        })
    }
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        self.term.id
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
        egui::Window::new(&self.term.title)
            .id(self.term.id)
            .open(open)
            .show(ctx, |ui| {
                if let Err(e) = self.term.ui(update_state, ui) {
                    luminol_core::error!(
                        update_state.toasts,
                        e.wrap_err("Error displaying terminal"),
                    );
                }
            });
    }
}
