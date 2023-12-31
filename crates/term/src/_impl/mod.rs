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

use crossbeam_channel::{unbounded, Receiver, Sender};
use std::io::prelude::*;
use std::sync::Arc;
pub use termwiz;

mod into;
use into::{IntoEgui, IntoWez, TryIntoWez};

pub type TermSender = Sender<Vec<termwiz::escape::Action>>;
pub type TermReceiver = Receiver<Vec<termwiz::escape::Action>>;

pub fn channel() -> (TermSender, TermReceiver) {
    unbounded()
}

pub use portable_pty::CommandBuilder;

pub use termwiz::Error;

pub struct Terminal {
    terminal: wezterm_term::Terminal,
    reader: TermReceiver,
    process: Option<Process>,
    id: Option<egui::Id>,
    title: Option<String>,
}

struct Process {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    pair: portable_pty::PtyPair,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.kill()
    }
}

impl Terminal {
    pub fn new(
        ctx: &egui::Context,
        command: portable_pty::CommandBuilder,
    ) -> Result<Self, termwiz::Error> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system.openpty(portable_pty::PtySize::default())?;
        let child = pair.slave.spawn_command(command)?;

        let reader = pair.master.try_clone_reader()?;
        let writer = pair.master.take_writer()?;

        let terminal = wezterm_term::Terminal::new(
            wezterm_term::TerminalSize::default(),
            Arc::new(Config),
            "luminol-term",
            "1.0",
            writer,
        );

        let ctx = ctx.clone();
        let (sender, reciever) = unbounded();
        std::thread::spawn(move || {
            let mut buf = [0; 2usize.pow(10)];
            let mut reader = std::io::BufReader::new(reader);
            let mut parser = termwiz::escape::parser::Parser::new();

            loop {
                let Ok(len) = reader.read(&mut buf) else {
                    return;
                };
                let actions = parser.parse_as_vec(&buf[0..len]);
                if !actions.is_empty() {
                    ctx.request_repaint();
                }
                let Ok(_) = sender.send(actions) else { return };
            }
        });

        Ok(Self {
            terminal,
            reader: reciever,
            process: Some(Process { pair, child }),
            id: None,
            title: None,
        })
    }

    pub fn new_readonly(
        ctx: &egui::Context,
        id: egui::Id,
        title: impl Into<String>,
        receiver: Receiver<Vec<termwiz::escape::Action>>,
        default_cols: usize,
        default_rows: usize,
    ) -> Self {
        let (cols, rows) = ctx.memory_mut(|m| {
            *m.data
                .get_persisted_mut_or_insert_with(id, move || (default_cols, default_rows))
        });
        let mut size = wezterm_term::TerminalSize::default();
        size.cols = cols;
        size.rows = rows;
        Self {
            terminal: wezterm_term::Terminal::new(
                size,
                Arc::new(Config),
                "luminol-term",
                "1.0",
                Box::new(std::io::sink()),
            ),
            reader: receiver,
            process: None,
            id: Some(id),
            title: Some(title.into()),
        }
    }

    pub fn title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| self.terminal.get_title().replace("wezterm", "luminol-term"))
    }

    pub fn id(&self) -> egui::Id {
        if let Some(id) = self.id {
            id
        } else if let Some(id) = self.process.as_ref().and_then(|p| p.child.process_id()) {
            egui::Id::new(id)
        } else {
            egui::Id::new(self.title())
        }
    }

    pub fn update(&mut self) {
        while let Ok(actions) = self.reader.try_recv() {
            self.terminal.perform_actions(actions);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> std::io::Result<()> {
        self.update();

        let mut size = self.terminal.get_size();
        let cursor_pos = self.terminal.cursor_pos();
        let palette = self.terminal.get_config().color_palette();

        ui.horizontal(|ui| {
            if self.process.is_some()
                && ui
                    .button(egui::RichText::new("KILL").color(egui::Color32::RED))
                    .clicked()
            {
                self.kill()
            }

            let mut resize = false;
            resize |= ui.add(egui::DragValue::new(&mut size.cols)).changed();

            ui.label("×");
            resize |= ui.add(egui::DragValue::new(&mut size.rows)).changed();

            if resize {
                self.terminal.resize(size);
                ui.memory_mut(|m| m.data.insert_persisted(self.id(), (size.cols, size.rows)));
                if let Some(process) = &mut self.process {
                    if let Err(e) = process.pair.master.resize(portable_pty::PtySize {
                        rows: size.rows as u16,
                        cols: size.cols as u16,
                        ..Default::default()
                    }) {
                        eprintln!("error resizing terminal: {e}");
                    }
                }
            }
        });

        ui.separator();

        ui.spacing_mut().item_spacing = egui::Vec2::ZERO;

        let text_width = ui.fonts(|f| f.glyph_width(&egui::FontId::monospace(12.0), '?'));
        let text_height = ui.text_style_height(&egui::TextStyle::Monospace);

        let scroll_area_height = (size.rows + 1) as f32 * text_height;
        egui::ScrollArea::vertical()
            .max_height(scroll_area_height)
            .min_scrolled_height(scroll_area_height)
            .stick_to_bottom(true)
            .show_rows(
                ui,
                text_height,
                self.terminal.screen().scrollback_rows(),
                |ui, rows| {
                    let mut job = egui::text::LayoutJob::default();
                    let mut iter = self
                        .terminal
                        .screen()
                        .lines_in_phys_range(rows)
                        .into_iter()
                        .peekable();
                    while let Some(line) = iter.next() {
                        for cluster in line.cluster(None) {
                            let fg_color =
                                palette.resolve_fg(cluster.attrs.foreground()).into_egui();
                            let bg_color =
                                palette.resolve_bg(cluster.attrs.background()).into_egui();
                            let underline = if !matches!(
                                cluster.attrs.underline(),
                                wezterm_term::Underline::None
                            ) {
                                egui::Stroke::new(
                                    1.0,
                                    palette
                                        .resolve_fg(cluster.attrs.underline_color())
                                        .into_egui(),
                                )
                            } else {
                                egui::Stroke::NONE
                            };
                            let strikethrough = if cluster.attrs.strikethrough() {
                                egui::Stroke::new(
                                    1.0,
                                    palette.resolve_fg(cluster.attrs.foreground()).into_egui(),
                                )
                            } else {
                                egui::Stroke::NONE
                            };
                            job.append(
                                &cluster.text,
                                0.0,
                                egui::TextFormat {
                                    font_id: egui::FontId::monospace(12.0),
                                    color: fg_color,
                                    background: bg_color,
                                    italics: cluster.attrs.italic(),
                                    underline,
                                    strikethrough,
                                    ..Default::default()
                                },
                            );
                        }
                        if iter.peek().is_some() {
                            job.append(
                                "\n",
                                0.0,
                                egui::TextFormat {
                                    font_id: egui::FontId::monospace(12.0),
                                    ..Default::default()
                                },
                            );
                        }
                    }

                    let galley = ui.fonts(|f| f.layout_job(job));
                    let mut galley_rect = galley.rect;
                    galley_rect.set_width(text_width * size.cols as f32);

                    let cursor = galley
                        .cursor_from_pos(egui::vec2(cursor_pos.x as f32, cursor_pos.y as f32));
                    let cursor_pos = galley.pos_from_cursor(&cursor);

                    let (response, painter) =
                        ui.allocate_painter(galley_rect.size(), egui::Sense::click_and_drag());

                    if response.clicked() && !response.has_focus() {
                        ui.memory_mut(|mem| mem.request_focus(response.id));
                    }

                    painter.rect_filled(
                        galley_rect.translate(response.rect.min.to_vec2()),
                        0.0,
                        palette.background.into_egui(),
                    );

                    painter.galley(response.rect.min, galley);

                    painter.rect_stroke(
                        egui::Rect::from_min_size(
                            cursor_pos.min,
                            egui::vec2(text_width, text_height),
                        ),
                        egui::Rounding::ZERO,
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );

                    if response.hovered() {
                        ui.output_mut(|o| o.mutable_text_under_cursor = true);
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
                    }

                    let focused = response.has_focus();
                    ui.input(|i| {
                        if !focused {
                            return;
                        }

                        for e in i.events.iter() {
                            let result = match e {
                                egui::Event::PointerMoved(pos) => {
                                    let relative_pos = *pos - response.rect.min;
                                    let char_x = (relative_pos.x / 12.0) as usize;
                                    let char_y = (relative_pos.y / 12.0) as i64;
                                    self.terminal.mouse_event(wezterm_term::MouseEvent {
                                        kind: wezterm_term::MouseEventKind::Move,
                                        x: char_x,
                                        y: char_y,
                                        x_pixel_offset: 0,
                                        y_pixel_offset: 0,
                                        button: wezterm_term::MouseButton::None,
                                        modifiers: i.modifiers.into_wez(),
                                    })
                                }
                                egui::Event::PointerButton {
                                    pos,
                                    button,
                                    pressed,
                                    modifiers,
                                } => {
                                    let relative_pos = *pos - response.rect.min;
                                    let char_x = (relative_pos.x / text_width) as usize;
                                    let char_y = (relative_pos.y / text_height) as i64;
                                    self.terminal.mouse_event(wezterm_term::MouseEvent {
                                        kind: if *pressed {
                                            wezterm_term::MouseEventKind::Press
                                        } else {
                                            wezterm_term::MouseEventKind::Release
                                        },
                                        x: char_x,
                                        y: char_y,
                                        x_pixel_offset: 0,
                                        y_pixel_offset: 0,
                                        button: button.into_wez(),
                                        modifiers: modifiers.into_wez(),
                                    })
                                }
                                egui::Event::Scroll(pos) => {
                                    let relative_pos =
                                        i.pointer.interact_pos().unwrap() - response.rect.min;
                                    let char_x = (relative_pos.x / text_width) as usize;
                                    let char_y = (relative_pos.y / text_height) as i64;
                                    self.terminal.mouse_event(wezterm_term::MouseEvent {
                                        kind: wezterm_term::MouseEventKind::Press,
                                        x: char_x,
                                        y: char_y,
                                        x_pixel_offset: 0,
                                        y_pixel_offset: 0,
                                        button: if pos.y.is_sign_positive() {
                                            wezterm_term::MouseButton::WheelUp(pos.y as usize)
                                        } else {
                                            wezterm_term::MouseButton::WheelDown(-pos.y as usize)
                                        },
                                        modifiers: i.modifiers.into_wez(),
                                    })
                                }
                                egui::Event::Key {
                                    key,
                                    modifiers,
                                    pressed,
                                    ..
                                } => {
                                    if let Ok(key) = key.try_into_wez() {
                                        if *pressed {
                                            self.terminal.key_down(key, modifiers.into_wez())
                                        } else {
                                            self.terminal.key_up(key, modifiers.into_wez())
                                        }
                                    } else {
                                        Ok(())
                                    }
                                }
                                egui::Event::Text(t) => t
                                    .chars()
                                    .try_for_each(|c| {
                                        self.terminal.key_down(
                                            wezterm_term::KeyCode::Char(c),
                                            i.modifiers.into_wez(),
                                        )
                                    })
                                    .and_then(|_| {
                                        t.chars().try_for_each(|c| {
                                            self.terminal.key_up(
                                                wezterm_term::KeyCode::Char(c),
                                                i.modifiers.into_wez(),
                                            )
                                        })
                                    }),
                                _ => Ok(()),
                            };
                            if let Err(e) = result {
                                eprintln!("terminal input error {e:?}");
                            }
                        }
                    });
                },
            );

        Ok(())
    }

    #[inline(never)]
    pub fn kill(&mut self) {
        if let Some(process) = &mut self.process {
            if let Err(e) = process.child.kill() {
                eprintln!("error killing child: {e}");
            }
        }
    }
}

#[derive(Debug)]
struct Config;

impl wezterm_term::TerminalConfiguration for Config {
    fn color_palette(&self) -> wezterm_term::color::ColorPalette {
        wezterm_term::color::ColorPalette::default()
    }

    fn enable_title_reporting(&self) -> bool {
        true
    }
}
