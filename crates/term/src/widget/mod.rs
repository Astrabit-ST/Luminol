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

use alacritty_terminal::event::Event;
use alacritty_terminal::grid::{Dimensions, GridIterator};
use alacritty_terminal::index::{Column, Line, Point};
use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::term::{LineDamageBounds, TermDamage, TermMode};
use alacritty_terminal::vte::ansi::CursorShape;
use alacritty_terminal::Grid;
use egui::epaint::text::cursor::RCursor;
use luminol_config::terminal::CursorBlinking;

use crate::backends::Backend;

mod keys;

pub struct Terminal<T> {
    backend: T,
    stable_time: f32,
    scroll_pt: f32,

    layout_job: egui::text::LayoutJob,
    ime_text: Option<String>,

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
    fn new(backend: T, id: egui::Id) -> Self {
        Self {
            backend,
            id, // FIXME add unique id system
            scroll_pt: 0.0,
            stable_time: 0.0,

            layout_job: egui::text::LayoutJob::default(),
            ime_text: None,

            title: "Luminol Terminal".to_string(),
        }
    }
}

impl ProcessTerminal {
    pub fn process(
        exec: ExecOptions,
        update_state: &luminol_core::UpdateState<'_>,
    ) -> std::io::Result<Self> {
        let options = alacritty_terminal::tty::Options {
            shell: exec
                .program
                .map(|program| alacritty_terminal::tty::Shell::new(program, exec.args)),
            working_directory: exec.working_directory,
            hold: false,
        };
        let backend = crate::backends::Process::new(&options, update_state)?;
        Ok(Self::new(
            backend,
            egui::Id::new("luminol_term_process").with(std::time::Instant::now()),
        ))
    }
}

impl ChannelTerminal {
    pub fn channel(
        recv: std::sync::mpsc::Receiver<u8>,
        config: &luminol_config::terminal::Config,
    ) -> Self {
        let backend = crate::backends::Channel::new(recv, config);
        Self::new(backend, egui::Id::new("luminol_term_channel"))
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
        self.layout_job = Default::default();
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        self.backend.with_term(|term| {
            term.grid_mut().reset();
        });
        self.layout_job = Default::default();
    }

    pub fn update(&mut self) {
        self.backend.update()
    }

    fn layout_job_damage(
        job: &mut egui::text::LayoutJob,
        config: &luminol_config::terminal::Config,
        grid: &Grid<Cell>,
        damage: impl IntoIterator<Item = LineDamageBounds>,
    ) {
        for line in damage {
            for column in line.left..=line.right {
                let point = Point::new(Line(line.line as i32), Column(column));
                let cell = &grid[point];

                // we have to offset the index by the line to account for additional newlines
                let index = line.line * grid.columns() + column + line.line; // index is an index into a section

                let section = &mut job.sections[index];

                let mut buf = [0; 4];
                let text = cell.c.encode_utf8(&mut buf);
                let text_format = Self::text_format_for_cell(config, cell);

                let old_range = section.byte_range.clone();
                let delta = (old_range.end - old_range.start) as isize - text.len() as isize;

                // I'm pretty sure this handles multibyte correctly?
                job.text.replace_range(old_range.clone(), text);
                section.byte_range.end = section.byte_range.start + text.len();
                section.format = text_format;

                // we need to update the byte ranges of everything *after* this
                for section in &mut job.sections[index + 1..] {
                    section.byte_range.start =
                        section.byte_range.start.checked_add_signed(-delta).unwrap();
                    section.byte_range.end =
                        section.byte_range.end.checked_add_signed(-delta).unwrap();
                }
            }
        }
    }

    fn layout_job_full(
        columns: usize,
        config: &luminol_config::terminal::Config,
        display_iter: GridIterator<'_, Cell>,
    ) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        for cell in display_iter {
            let mut buf = [0; 4];
            let text = cell.c.encode_utf8(&mut buf);

            let format = Self::text_format_for_cell(config, &cell);
            job.append(text, 0.0, format);

            if cell.point.column >= columns - 1 {
                job.append("\n", 0.0, Default::default())
            }
        }

        job
    }

    fn text_format_for_cell(
        config: &luminol_config::terminal::Config,
        cell: &Cell,
    ) -> egui::TextFormat {
        let mut color = config.theme.get_ansi_color(cell.fg);
        let mut background = config.theme.get_ansi_color(cell.bg);

        if cell.flags.contains(Flags::INVERSE) {
            color = invert_color(color);
            background = invert_color(background);
        }

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

        egui::TextFormat {
            font_id: config.font.clone(),
            color,
            background,
            italics,
            underline,
            strikethrough,
            ..Default::default()
        }
    }

    pub fn ui(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        ui: &mut egui::Ui,
    ) -> color_eyre::Result<()> {
        self.backend.update();

        self.backend.with_event_recv(|recv| {
            for event in recv.try_iter() {
                match event {
                    Event::Title(title) => self.title = title,
                    Event::ResetTitle => "Luminol Terminal".clone_into(&mut self.title),
                    Event::Bell => {
                        let bell = luminol_macros::include_asset!("assets/sounds/bell.wav");
                        let cursor = std::io::Cursor::new(bell);
                        update_state
                            .audio
                            .play_from_file(cursor, false, 25, 100, luminol_audio::Source::SE)
                            .unwrap();
                    }
                    _ => {}
                }
            }
        });

        let config = &update_state.global_config.terminal;
        let font_id = config.font.clone();

        let (screen_columns, screen_lines, total_lines, display_offset, cursor_style, cursor_point) =
            self.backend.with_term(|term| {
                match term.damage() {
                    // we only do partial repaints if the layout job is empty
                    TermDamage::Partial(damage) if !self.layout_job.is_empty() => {
                        // We have to collect here to avoid borrowing the terminal mutably twice (even though it isn't, really)
                        let damage = damage.collect::<Vec<_>>();
                        Self::layout_job_damage(&mut self.layout_job, config, term.grid(), damage)
                    }
                    _ => {
                        self.layout_job = Self::layout_job_full(
                            term.columns(),
                            config,
                            term.renderable_content().display_iter,
                        );
                    }
                }
                term.reset_damage();

                (
                    term.columns(),
                    term.screen_lines(),
                    term.total_lines(),
                    term.grid().display_offset(),
                    term.cursor_style(),
                    term.grid().cursor.point,
                )
            });

        let max_size = ui.available_size();
        let (row_height, char_width) = ui.fonts(|f| {
            (
                f.row_height(&font_id).round(),
                f.glyph_width(&font_id, '*').round(),
            )
        });

        let terminal_size = egui::vec2(
            char_width * screen_columns as f32,
            row_height * screen_lines as f32,
        );
        let total_area = terminal_size + egui::vec2(5.0, 0.0);

        let (response, painter) = egui::ScrollArea::neither()
            .show(ui, |ui| {
                ui.allocate_painter(max_size.max(total_area), egui::Sense::click_and_drag())
            })
            .inner;

        if max_size != total_area {
            let new_size = max_size - egui::vec2(5.0, 0.0);

            let cols = (new_size.x / char_width) as _;
            let lines = (new_size.y / row_height) as _;
            self.set_size(cols, lines)
        }

        painter.rect_filled(
            egui::Rect::from_min_size(response.rect.min, terminal_size),
            0.0,
            egui::Color32::from_rgb(40, 39, 39),
        );

        let galley = painter.layout_job(self.layout_job.clone());
        painter.galley(response.rect.min, galley.clone(), egui::Color32::WHITE);

        if let Some(ime_text) = self.ime_text.clone() {
            let ime_text_galley = painter.layout(
                ime_text,
                font_id.clone(),
                egui::Color32::WHITE,
                galley.rect.width(),
            );

            let background_rect = ime_text_galley.rect.translate(response.rect.min.to_vec2());
            painter.rect_filled(
                background_rect,
                egui::Rounding::ZERO,
                egui::Color32::from_rgb(40, 39, 39),
            );
            painter.galley(response.rect.min, ime_text_galley, egui::Color32::WHITE);
        }

        if response.hovered() {
            ui.output_mut(|o| o.mutable_text_under_cursor = true);
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }

        if response.clicked() && !response.lost_focus() {
            response.request_focus();
        }

        let mut cursor_shape = cursor_style.shape;
        if !response.has_focus() {
            cursor_shape = CursorShape::HollowBlock;
            self.ime_text = None;
        }

        self.stable_time += ui.input(|i| i.unstable_dt);
        let (mut inner_color, outer_color) = match cursor_shape {
            CursorShape::Block | CursorShape::Underline => {
                (config.theme.cursor_color, config.theme.cursor_color)
            }
            CursorShape::Beam => (config.theme.cursor_color, egui::Color32::TRANSPARENT),
            CursorShape::HollowBlock => (egui::Color32::TRANSPARENT, config.theme.cursor_color),
            CursorShape::Hidden => (egui::Color32::TRANSPARENT, egui::Color32::TRANSPARENT),
        };

        let blink = match config.cursor_blinking {
            CursorBlinking::Always => true,
            CursorBlinking::Never => false,
            CursorBlinking::Terminal => cursor_style.blinking,
        };

        if blink {
            update_state
                .ctx
                .request_repaint_after(std::time::Duration::from_secs(1));
        }

        if blink && (self.stable_time % 2.) > 1.0 {
            inner_color = egui::Color32::TRANSPARENT;
        }

        let rcursor_without_offset = RCursor {
            row: cursor_point.line.0 as usize,
            column: cursor_point.column.0,
        };
        let rcursor = RCursor {
            row: rcursor_without_offset.row + display_offset,
            ..rcursor_without_offset
        };

        let cursor = galley.from_rcursor(rcursor);
        let cursor_without_offset = galley.from_rcursor(rcursor_without_offset);

        let mut cursor_pos = galley.pos_from_cursor(&cursor).min + response.rect.min.to_vec2();
        let mut cursor_pos_without_offset =
            galley.pos_from_cursor(&cursor_without_offset).min + response.rect.min.to_vec2();

        let cursor_rect = match cursor_shape {
            CursorShape::Block | CursorShape::HollowBlock | CursorShape::Hidden => {
                egui::Rect::from_min_size(cursor_pos, egui::vec2(char_width, row_height))
            }
            CursorShape::Beam => egui::Rect::from_min_size(cursor_pos, egui::vec2(2.0, row_height)),
            CursorShape::Underline => {
                cursor_pos.y += row_height - 2.0;
                cursor_pos_without_offset.y += row_height - 2.0;
                egui::Rect::from_min_size(cursor_pos, egui::vec2(char_width, 2.0))
            }
        };
        let cursor_rect_without_offset =
            egui::Rect::from_min_size(cursor_pos_without_offset, cursor_rect.size());

        painter.rect(
            cursor_rect,
            egui::Rounding::ZERO,
            inner_color,
            egui::Stroke::new(1.0, outer_color),
        );

        // FIXME render to galley.rect, not response.rect. swapping them out mostly works, but the handle doesn't display quite right!
        // scrollbar
        let min = response.rect.min + egui::vec2(galley.rect.width(), 0.0);
        let size = egui::vec2(5., response.rect.height());
        let sidebar_rect = egui::Rect::from_min_size(min, size);
        painter.rect_filled(
            sidebar_rect,
            egui::Rounding::ZERO,
            ui.visuals().extreme_bg_color,
        );

        // handle
        let scrollbar_percent = screen_lines as f32 / total_lines as f32;
        let scrollbar_height = scrollbar_percent * response.rect.height();

        let scrollbar_offset_percent = display_offset as f32 / total_lines as f32;
        let scrollbar_offset = scrollbar_offset_percent * response.rect.height();
        let scrollbar_y = response.rect.max.y - scrollbar_offset - scrollbar_height;

        let min = egui::pos2(min.x, scrollbar_y);
        let size = egui::vec2(5., scrollbar_height);
        let scrollbar_rect = egui::Rect::from_min_size(min, size);

        painter.rect_filled(
            scrollbar_rect,
            egui::Rounding::same(5.),
            ui.visuals().widgets.active.fg_stroke.color,
        );

        if response.has_focus() {
            ui.memory_mut(|mem| mem.set_focus_lock_filter(response.id, FILTER));
            ui.output_mut(|o| {
                o.ime = Some(egui::output::IMEOutput {
                    rect: galley.rect,
                    cursor_rect: cursor_rect_without_offset,
                });
            });

            let (events, modifiers) = ui.input(|i| (i.filtered_events(&FILTER), i.modifiers));
            self.process_egui_events(
                events,
                modifiers,
                response.rect.min,
                response.hover_pos(),
                &galley,
            );
        }

        Ok(())
    }

    fn handle_scroll(
        &mut self,
        cursor_pos: Option<RCursor>,
        unit: egui::MouseWheelUnit,
        scroll_delta: egui::Vec2,
    ) {
        match unit {
            egui::MouseWheelUnit::Point => {
                self.scroll_pt += scroll_delta.y;
            }
            egui::MouseWheelUnit::Line => {
                self.scroll_pt += scroll_delta.y * 16.;
            }
            egui::MouseWheelUnit::Page => {
                let (_width, height) = self.backend.size();
                self.scroll_pt += scroll_delta.y * 16. * height as f32;
            }
        }
        let delta = (self.scroll_pt / 16.).trunc() as i32;
        self.scroll_pt %= 16.;

        let term_mode = self.backend.with_term(|term| *term.mode());

        if term_mode.contains(TermMode::SGR_MOUSE) {
            if let Some(cursor_pos) = cursor_pos {
                let button = 64 + delta.is_negative() as i32;

                let command = format!("\x1b[<{button};{};{}M", cursor_pos.column, cursor_pos.row);
                self.backend.send(command.into_bytes());
            }
        } else if term_mode.contains(TermMode::ALT_SCREEN | TermMode::ALTERNATE_SCROLL) {
            let line_cmd = if delta.is_negative() { b'A' } else { b'B' };
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
            self.layout_job = Default::default();
        }
    }

    fn process_egui_events(
        &mut self,
        events: Vec<egui::Event>,
        mut modifiers: egui::Modifiers,
        response_pos: egui::Pos2,
        hover_pos: Option<egui::Pos2>,
        galley: &egui::Galley,
    ) {
        // i have no idea how CMD works on mac. nor do i have a machine to test it with
        // but without this it messes up the key conversion
        modifiers.command = false;

        let term_mode = self.backend.with_term(|term| *term.mode());
        let mut term_modified = false;
        for event in events {
            match event {
                egui::Event::Text(text) => {
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
                egui::Event::Paste(text) => {
                    if let Some(bytes) = keys::key_to_codes(egui::Key::V, modifiers, term_mode) {
                        self.backend.send(bytes);
                    }

                    if modifiers.shift {
                        self.backend.send(text.into_bytes());
                    }
                    term_modified = true;
                }
                egui::Event::Copy => {
                    if let Some(bytes) = keys::key_to_codes(egui::Key::C, modifiers, term_mode) {
                        self.backend.send(bytes);
                    }
                    term_modified = true;
                }
                egui::Event::Cut => {
                    if let Some(bytes) = keys::key_to_codes(egui::Key::X, modifiers, term_mode) {
                        self.backend.send(bytes);
                    }
                    term_modified = true;
                }
                egui::Event::Key {
                    key, pressed: true, ..
                } => {
                    if let Some(bytes) = keys::key_to_codes(key, modifiers, term_mode) {
                        self.backend.send(bytes);
                    }
                    term_modified = true;
                }
                egui::Event::MouseWheel { unit, delta, .. } => self.handle_scroll(
                    hover_pos.map(|pos| galley.cursor_from_pos(pos.to_vec2()).rcursor),
                    unit,
                    delta,
                ),
                egui::Event::Ime(egui::ImeEvent::Preedit(text)) => self.ime_text = Some(text),
                egui::Event::Ime(egui::ImeEvent::Commit(text)) => {
                    self.ime_text = None;
                    self.backend.send(text.into_bytes());
                    term_modified = true;
                }
                _ => {}
            }
        }

        if term_modified {
            self.backend.with_term(|term| {
                term.scroll_display(alacritty_terminal::grid::Scroll::Bottom);
            });
            self.layout_job = Default::default();
        }
    }

    pub fn kill(&mut self) {
        self.backend.kill();
    }
}

fn invert_color(color: egui::Color32) -> egui::Color32 {
    let [r, g, b, a] = color.to_array();
    egui::Color32::from_rgba_premultiplied(!r, !g, !b, a)
}
