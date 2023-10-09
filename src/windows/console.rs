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

pub struct Console {
    term: luminol_term::Terminal,
}

impl Console {
    pub fn new(command: luminol_term::CommandBuilder) -> Result<Self, luminol_term::Error> {
        Ok(Self {
            term: luminol_term::Terminal::new(command)?,
        })
    }
}

impl super::window::Window for Console {
    fn name(&self) -> String {
        self.term.title()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("Console")
    }

    fn requires_filesystem(&self) -> bool {
        true
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .id(self.term.id())
            .open(open)
            .resizable(false)
            .show(ctx, |ui| {
                if let Err(e) = self.term.ui(ui) {
                    crate::state!()
                        .toasts
                        .error(format!("error displaying terminal: {e:?}"));
                }
            });
    }
}
