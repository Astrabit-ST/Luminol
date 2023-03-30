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

use std::io::prelude::*;
use std::sync::mpsc::{sync_channel, Receiver};
use std::sync::Arc;

mod into;
use into::{IntoEgui, IntoWez, TryIntoWez};

pub struct Terminal {
    terminal: wezterm_term::Terminal,
    reader: Receiver<Vec<termwiz::escape::Action>>,

    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        if let Err(e) = self.child.kill() {
            eprintln!("error killing child: {e}");
        }
    }
}

impl Terminal {
    pub fn new(command: portable_pty::CommandBuilder) -> Result<Self, ()> {
        let pty_system = portable_pty::native_pty_system();
        let pair = pty_system
            .openpty(portable_pty::PtySize::default())
            .unwrap();
        let child = pair.slave.spawn_command(command).unwrap();

        let mut reader = pair.master.try_clone_reader().unwrap();
        let writer = pair.master.take_writer().unwrap();

        let terminal = wezterm_term::Terminal::new(
            wezterm_term::TerminalSize::default(),
            Arc::new(Config),
            "luminol-term",
            "1.0",
            Box::new(writer),
        );

        let (sender, reciever) = sync_channel(1);
        std::thread::spawn(move || {
            let mut buf = [0; 2usize.pow(10)];
            let mut parser = termwiz::escape::parser::Parser::new();

            loop {
                let Ok(len) = reader.read(&mut buf) else {
                    return
                };
                let actions = parser.parse_as_vec(&buf[0..len]);
                let Ok(_) = sender.send(actions) else {
                    return
                };
            }
        });

        Ok(Self {
            terminal,
            reader: reciever,
            child,
        })
    }

    pub fn title(&self) -> &str {
        self.terminal.get_title()
    }

    pub fn id(&self) -> egui::Id {
        if let Some(id) = self.child.process_id() {
            egui::Id::new(id)
        } else {
            egui::Id::new(self.title())
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> std::io::Result<()> {
        if let Ok(actions) = self.reader.try_recv() {
            self.terminal.perform_actions(actions);
        }

        let mut size = self.terminal.get_size();
        let cursor_pos = self.terminal.cursor_pos();
        let palette = self.terminal.get_config().color_palette();

        let mut job = egui::text::LayoutJob {
            wrap: egui::epaint::text::TextWrapping {
                ..Default::default()
            },
            ..Default::default()
        };
        self.terminal.screen().for_each_phys_line(|i, l| {
            for cluster in l.cluster(None) {
                let fg_color = palette.resolve_fg(cluster.attrs.foreground()).into_egui();
                let bg_color = palette.resolve_bg(cluster.attrs.background()).into_egui();
                let underline =
                    if !matches!(cluster.attrs.underline(), wezterm_term::Underline::None) {
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
            job.append(
                "\n",
                0.0,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(12.0),
                    ..Default::default()
                },
            );
        });
        let galley = ui.fonts(|f| f.layout_job(job));
        let text_width = ui.fonts(|f| f.glyph_width(&egui::FontId::monospace(12.0), ' '));

        let (response, painter) =
            ui.allocate_painter(galley.rect.size(), egui::Sense::click_and_drag());
        painter.galley(response.rect.min, galley);
        let text_height = ui.text_style_height(&egui::TextStyle::Monospace);
        let cursor_pos = response.rect.min
            + egui::vec2(
                cursor_pos.x as f32 * text_width,
                cursor_pos.y as f32 * text_height,
            );
        painter.rect_stroke(
            egui::Rect::from_min_size(cursor_pos, egui::vec2(text_width, text_height)),
            egui::Rounding::none(),
            egui::Stroke::new(1.0, egui::Color32::WHITE),
        );

        ui.input(|i| {
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
                        let relative_pos = i.pointer.interact_pos().unwrap() - response.rect.min;
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
                    egui::Event::Key { key, modifiers, .. } => {
                        if let Ok(key) = key.try_into_wez() {
                            self.terminal.key_down(key, modifiers.into_wez())
                        } else {
                            Ok(())
                        }
                    }
                    egui::Event::Text(t) => t.chars().try_for_each(|c| {
                        self.terminal
                            .key_down(wezterm_term::KeyCode::Char(c), i.modifiers.into_wez())
                    }),
                    _ => Ok(()),
                };
                if let Err(e) = result {
                    eprintln!("temrinal input error {e:?}");
                }
            }
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui
                .button(egui::RichText::new("KILL").color(egui::Color32::RED))
                .clicked()
            {
                if let Err(e) = self.child.kill() {
                    eprintln!("error killing child: {e}");
                }
            }

            if ui.add(egui::DragValue::new(&mut size.rows)).changed() {
                self.terminal.resize(size);
            }
            ui.label("x");
            if ui.add(egui::DragValue::new(&mut size.cols)).changed() {
                self.terminal.resize(size);
            }
        });

        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(16));

        Ok(())
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
