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

use alacritty_terminal::event::Event;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::TermMode;
use alacritty_terminal::{event_loop::Msg, grid::Dimensions};

use crate::backends::Backend;

mod keys;

mod theme;
pub use theme::Theme;

pub struct Terminal<T> {
    backend: T,
    theme: Theme, // TODO convert into shared config (possibly do this in luminol-preferences)
    pub id: egui::Id,
    pub title: String,
}

pub type ProcessTerminal = Terminal<crate::backends::Process>;
pub type ChannelTerminal = Terminal<crate::backends::Channel>;

impl<T> Terminal<T> {
    fn new(backend: T) -> Self {
        Self {
            backend,
            id: egui::Id::new("luminol_term_terminal"),
            theme: Theme::default(),
            title: "Luminol Terminal".to_string(),
        }
    }
}

impl ProcessTerminal {
    pub fn process(options: &alacritty_terminal::tty::Options) -> std::io::Result<Self> {
        crate::backends::Process::new(options).map(Self::new)
    }
}

impl ChannelTerminal {
    pub fn channel(recv: std::sync::mpsc::Receiver<u8>) -> Self {
        let backend = crate::backends::Channel::new(recv);
        Self::new(backend)
    }
}

const FILTER: egui::EventFilter = egui::EventFilter {
    tab: true,
    horizontal_arrows: true,
    vertical_arrows: true,
    escape: true,
};

impl<T> Terminal<T>
where
    T: Backend,
{
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
        self.backend.size().0
    }

    pub fn rows(&self) -> usize {
        self.backend.size().1
    }

    pub fn erase_scrollback(&mut self) {
        self.backend.with_term(|term| {
            term.grid_mut().clear_history();
        });
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        self.backend.with_term(|term| {
            term.grid_mut().clear_viewport();
        });
    }

    pub fn update(&mut self) {
        self.backend.update()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> color_eyre::Result<()> {
        self.backend.update();

        self.backend.with_event_recv(|recv| {
            //
            for event in recv.try_iter() {
                match event {
                    Event::Title(title) => self.title = title,
                    Event::ResetTitle => "Luminol Terminal".clone_into(&mut self.title),
                    _ => {}
                }
            }
        });

        let response = self.backend.with_term(|term| {
            let font_id = egui::FontId::monospace(14.);
            let (row_height, char_width) = ui.fonts(|f| {
                (
                    f.row_height(&font_id).round(),
                    f.glyph_width(&font_id, '*').round(),
                )
            });

            let terminal_size = egui::vec2(
                char_width * term.columns() as f32,
                row_height * term.screen_lines() as f32,
            );

            let (response, painter) =
                ui.allocate_painter(terminal_size, egui::Sense::click_and_drag());

            // TODO cache render jobs
            let content = term.renderable_content();
            let mut job = egui::text::LayoutJob::default();
            for cell in content.display_iter {
                let mut buf = [0; 4];
                let text = cell.c.encode_utf8(&mut buf);

                let (color, background) =
                    if cell.point == term.grid().cursor.point && response.has_focus() {
                        (egui::Color32::BLACK, egui::Color32::WHITE)
                    } else {
                        (
                            self.theme.get_ansi_color(cell.fg),
                            self.theme.get_ansi_color(cell.bg),
                        )
                    };

                let italics = cell.flags.contains(Flags::ITALIC);
                let underline = cell
                    .flags
                    .contains(Flags::UNDERLINE)
                    .then_some(egui::Stroke::new(1.0, color))
                    .unwrap_or_default();
                let strikethrough = cell
                    .flags
                    .contains(Flags::STRIKEOUT)
                    .then_some(egui::Stroke::new(1.0, color))
                    .unwrap_or_default();

                let format = egui::TextFormat {
                    font_id: font_id.clone(),
                    color,
                    background,
                    italics,
                    underline,
                    strikethrough,
                    ..Default::default()
                };

                job.append(text, 0.0, format);

                if cell.point.column >= term.columns() - 1 {
                    job.append("\n", 0.0, Default::default());
                }
            }

            let galley = ui.fonts(|f| f.layout_job(job));

            painter.rect_filled(
                egui::Rect::from_min_size(response.rect.min, terminal_size),
                0.0,
                egui::Color32::from_rgb(40, 39, 39),
            );

            painter.galley(response.rect.min, galley.clone(), egui::Color32::WHITE);

            if response.hovered() {
                ui.output_mut(|o| o.mutable_text_under_cursor = true);
                ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
            }

            if response.clicked() && !response.lost_focus() {
                response.request_focus();
            }

            let cursor_pos = term.grid().cursor.point;
            let ppp = painter.ctx().pixels_per_point();

            let cursor_rect = egui::Rect::from_min_size(
                egui::pos2(
                    cursor_pos.column.0 as f32 * char_width + response.rect.min.x,
                    cursor_pos.line.0 as f32 * row_height + response.rect.min.y + 1.,
                ),
                egui::vec2(char_width - ppp, row_height - ppp),
            );

            if !response.has_focus() {
                painter.rect(
                    cursor_rect,
                    egui::Rounding::ZERO,
                    egui::Color32::TRANSPARENT,
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );
            }

            response
        });

        if response.has_focus() {
            ui.memory_mut(|mem| mem.set_focus_lock_filter(response.id, FILTER));

            let (events, modifiers) = ui.input(|i| (i.filtered_events(&FILTER), i.modifiers));
            self.process_egui_events(events, modifiers);
        }

        Ok(())
    }

    fn process_egui_events(&mut self, events: Vec<egui::Event>, modifiers: egui::Modifiers) {
        let term_mode = self.backend.with_term(|term| *term.mode());
        for event in events {
            match event {
                egui::Event::Paste(text) | egui::Event::Text(text) => {
                    let bytes = text.into_bytes();
                    let cow = std::borrow::Cow::Owned(bytes);
                    self.backend.send(Msg::Input(cow));
                }
                egui::Event::Key {
                    key, pressed: true, ..
                } => {
                    if let Some(bytes) = keys::key_to_codes(key, modifiers, term_mode) {
                        let cow = std::borrow::Cow::Borrowed(bytes);
                        self.backend.send(Msg::Input(cow));
                    }
                }
                egui::Event::Scroll(scroll_delta) => {
                    let delta = scroll_delta.y as i32;
                    let alt_scroll = self.backend.with_term(|term| {
                        term.mode()
                            .contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL)
                    });
                    if alt_scroll {
                        let line_cmd = if delta.is_positive() { b'A' } else { b'B' };
                        let mut bytes = vec![];

                        for _ in 0..delta.abs() {
                            bytes.push(0x1b);
                            bytes.push(b'O');
                            bytes.push(line_cmd);
                        }

                        let cow = std::borrow::Cow::Owned(bytes);
                        self.backend.send(Msg::Input(cow));
                    } else {
                        self.backend.with_term(|term| {
                            term.grid_mut()
                                .scroll_display(alacritty_terminal::grid::Scroll::Delta(delta));
                        });
                    }
                }
                _ => {}
            }
        }
    }

    pub fn kill(&mut self) {
        self.backend.kill();
    }
}
