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
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::TermMode;
use alacritty_terminal::vte::ansi::CursorShape;

use crate::backends::Backend;

mod keys;

mod config;
pub use config::{Config, CursorBlinking};

mod theme;
pub use theme::Theme;

pub struct Terminal<T> {
    backend: T,
    config: Config, // TODO convert into shared config (possibly do this in luminol-preferences)
    config_ui: config::ConfigUi,
    stable_time: f32,
    scroll_pt: f32,
    pub id: egui::Id,
    pub title: String,
}

#[derive(Default, Clone)]
pub struct ExecOptions {
    pub program: Option<String>,
    pub args: Vec<String>,
    pub working_directory: Option<std::path::PathBuf>,
}

pub type ProcessTerminal = Terminal<crate::backends::Process>;
pub type ChannelTerminal = Terminal<crate::backends::Channel>;

impl<T> Terminal<T> {
    fn new(backend: T) -> Self {
        let config = Config::default();
        let config_ui = config::ConfigUi::new(&config);
        Self {
            backend,
            id: egui::Id::new("luminol_term_terminal"), // FIXME add unique id system
            scroll_pt: 0.0,
            stable_time: 0.0,
            config,
            config_ui,
            title: "Luminol Terminal".to_string(),
        }
    }
}

impl ProcessTerminal {
    pub fn process(exec: ExecOptions) -> std::io::Result<Self> {
        let options = alacritty_terminal::tty::Options {
            shell: exec
                .program
                .map(|program| alacritty_terminal::tty::Shell::new(program, exec.args)),
            working_directory: exec.working_directory,
            hold: false,
        };
        crate::backends::Process::new(&options).map(Self::new)
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

    pub fn ui(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ui: &mut egui::Ui,
    ) -> color_eyre::Result<()> {
        egui::Window::new("config test")
            .show(ui.ctx(), |ui| self.config_ui.ui(&mut self.config, ui));

        self.backend.update();

        self.backend.with_event_recv(|recv| {
            //
            for event in recv.try_iter() {
                match event {
                    Event::Title(title) => self.title = title,
                    Event::ResetTitle => "Luminol Terminal".clone_into(&mut self.title),
                    Event::Bell => {}
                    _ => {}
                }
            }
        });

        let (response, galley) = self.backend.with_term(|term| {
            let font_id = self.config.font.clone();
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

                let color = self.config.theme.get_ansi_color(cell.fg);
                let background = self.config.theme.get_ansi_color(cell.bg);

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

            let mut cursor_shape = term.cursor_style().shape;
            if !response.has_focus() {
                cursor_shape = CursorShape::HollowBlock;
            }

            self.stable_time += ui.input(|i| i.stable_dt.min(0.1));
            let (mut inner_color, outer_color) = match cursor_shape {
                CursorShape::Block | CursorShape::Underline => (
                    self.config.theme.cursor_color,
                    self.config.theme.cursor_color,
                ),
                CursorShape::Beam => (self.config.theme.cursor_color, egui::Color32::TRANSPARENT),
                CursorShape::HollowBlock => {
                    (egui::Color32::TRANSPARENT, self.config.theme.cursor_color)
                }
                CursorShape::Hidden => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT),
            };

            let blink = match self.config.cursor_blinking {
                CursorBlinking::Always => true,
                CursorBlinking::Never => false,
                CursorBlinking::Terminal => term.cursor_style().blinking,
            };

            if blink {
                let sin_component = self.stable_time / std::f32::consts::FRAC_PI_2 * 13.;
                let alpha = (sin_component.sin() + 1.) / 2.;
                inner_color = inner_color.gamma_multiply(alpha);
            }

            let cursor_point = term.grid().cursor.point;
            let cursor = galley.from_rcursor(egui::epaint::text::cursor::RCursor {
                row: cursor_point.line.0 as usize,
                column: cursor_point.column.0,
            });

            let mut cursor_pos = galley.pos_from_cursor(&cursor).min + response.rect.min.to_vec2();

            let cursor_rect = match cursor_shape {
                CursorShape::Block | CursorShape::HollowBlock | CursorShape::Hidden => {
                    egui::Rect::from_min_size(cursor_pos, egui::vec2(char_width, row_height))
                }
                CursorShape::Beam => {
                    egui::Rect::from_min_size(cursor_pos, egui::vec2(2.0, row_height))
                }
                CursorShape::Underline => {
                    cursor_pos.y += row_height - 2.0;
                    egui::Rect::from_min_size(cursor_pos, egui::vec2(char_width, 2.0))
                }
            };

            painter.rect(
                cursor_rect,
                egui::Rounding::ZERO,
                inner_color,
                egui::Stroke::new(1.0, outer_color),
            );

            painter
                .ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));

            (response, galley)
        });

        if response.has_focus() {
            ui.memory_mut(|mem| mem.set_focus_lock_filter(response.id, FILTER));

            let (events, modifiers) = ui.input(|i| (i.filtered_events(&FILTER), i.modifiers));
            self.process_egui_events(events, modifiers, response.rect.min, &galley);
        }

        Ok(())
    }

    fn handle_scroll(&mut self, scroll_delta: egui::Vec2) {
        self.scroll_pt += scroll_delta.y;
        let delta = (self.scroll_pt / 16.).trunc() as i32;
        self.scroll_pt %= 16.;

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

            self.backend.send(bytes);
        } else {
            self.backend.with_term(|term| {
                term.grid_mut()
                    .scroll_display(alacritty_terminal::grid::Scroll::Delta(delta));
            });
        }
    }

    fn process_egui_events(
        &mut self,
        events: Vec<egui::Event>,
        modifiers: egui::Modifiers,
        response_pos: egui::Pos2,
        galley: &egui::Galley,
    ) {
        let term_mode = self.backend.with_term(|term| *term.mode());
        let mut term_modified = false;
        for event in events {
            match event {
                egui::Event::Paste(text) | egui::Event::Text(text) => {
                    self.backend.send(text.into_bytes());
                    term_modified = true;
                }
                egui::Event::PointerButton {
                    pos,
                    pressed,
                    button,
                    ..
                } => {
                    let relative_pos = pos - response_pos;
                    let cursor = galley.cursor_from_pos(relative_pos).rcursor;

                    if term_mode.contains(TermMode::SGR_MOUSE) && modifiers.is_none() {
                        let c = if pressed { 'M' } else { 'm' };

                        let msg = format!(
                            "\x1b[<{};{};{}{}",
                            button as u8,
                            cursor.column + 1,
                            cursor.row + 1,
                            c
                        );

                        self.backend.send(msg.into_bytes());
                        term_modified = true;
                    }
                }
                egui::Event::PointerMoved(pos) => {
                    let relative_pos = pos - response_pos;
                    let cursor = galley.cursor_from_pos(relative_pos).rcursor;

                    if term_mode.contains(TermMode::SGR_MOUSE) && modifiers.is_none() {
                        let msg = format!("\x1b[<32;{};{}M", cursor.column + 1, cursor.row + 1);

                        self.backend.send(msg.into_bytes());
                        term_modified = true;
                    }
                }
                egui::Event::Key {
                    key, pressed: true, ..
                } => {
                    if let Some(bytes) = keys::key_to_codes(key, modifiers, term_mode) {
                        self.backend.send(bytes);
                    }
                    term_modified = true;
                }
                egui::Event::Scroll(scroll_delta) => self.handle_scroll(scroll_delta),
                _ => {}
            }
        }

        if term_modified {
            self.backend.with_term(|term| {
                term.scroll_display(alacritty_terminal::grid::Scroll::Bottom);
            })
        }
    }

    pub fn kill(&mut self) {
        self.backend.kill();
    }
}
