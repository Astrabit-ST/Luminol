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

use alacritty_terminal::grid::Dimensions;

use crate::backends::Backend;

mod theme;
pub use theme::Theme;

pub struct Terminal {
    // REVIEW should we use generics or trait objects?
    backend: Box<dyn Backend>,
    theme: Theme, // TODO convert into shared config (possibly do this in luminol-preferences)
    title: String,
}

impl Terminal {
    fn new(backend: impl Backend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
            theme: Theme::default(),
            title: "Luminol Terminal".to_string(),
        }
    }

    pub fn process(options: &alacritty_terminal::tty::Options) -> std::io::Result<Self> {
        crate::backends::Process::new(options).map(Self::new)
    }

    pub fn channel(recv: std::sync::mpsc::Receiver<u8>) -> Self {
        let backend = crate::backends::Channel::new(recv);
        Self::new(backend)
    }
}

impl Terminal {
    pub fn title(&self) -> String {
        self.title.to_string()
    }

    pub fn id(&self) -> egui::Id {
        egui::Id::new("luminol_term_terminal").with(&self.title)
    }

    pub fn set_size(&mut self, cols: usize, lines: usize) {
        self.backend.resize(lines, cols)
    }

    pub fn set_cols(&mut self, cols: usize) {
        self.set_size(cols, self.rows())
    }

    pub fn set_rows(&mut self, rows: usize) {
        self.set_size(self.cols(), rows)
    }

    pub fn size(&self) -> (usize, usize) {
        self.backend.size()
    }

    pub fn cols(&self) -> usize {
        self.backend.size().1
    }

    pub fn rows(&self) -> usize {
        self.backend.size().0
    }

    pub fn erase_scrollback(&mut self) {
        self.backend.with_term(&mut |term| {
            term.grid_mut().clear_history();
        });
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        self.backend.with_term(&mut |term| {
            term.grid_mut().clear_viewport();
        });
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> color_eyre::Result<()> {
        self.backend.update();

        self.backend.with_term(&mut |term| {
            let content = term.renderable_content();

            let mut job = egui::text::LayoutJob::default();
            for cell in content.display_iter {
                let mut buf = [0; 4];
                let text = cell.c.encode_utf8(&mut buf);

                let format = egui::TextFormat {
                    font_id: egui::FontId::monospace(12.),
                    color: self.theme.get_ansi_color(cell.fg),
                    background: self.theme.get_ansi_color(cell.bg),
                    ..Default::default()
                };

                job.append(text, 0.0, format);

                if cell.point.column >= term.columns() - 1 {
                    job.append("\n", 0.0, Default::default());
                }
            }
            let galley = ui.fonts(|f| f.layout_job(job));
            let (response, painter) =
                ui.allocate_painter(galley.rect.size(), egui::Sense::click_and_drag());

            painter.rect_filled(
                galley.rect.translate(response.rect.min.to_vec2()),
                0.0,
                egui::Color32::from_rgb(40, 39, 39),
            );

            painter.galley(response.rect.min, galley, egui::Color32::WHITE);

            if response.hovered() {
                ui.output_mut(|o| o.mutable_text_under_cursor = true);
                ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            }
        });

        Ok(())
    }

    pub fn kill(&mut self) {
        self.backend.kill();
    }
}
