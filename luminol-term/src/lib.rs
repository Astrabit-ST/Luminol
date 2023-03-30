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

const TERM_SIZE: usize = 2usize.pow(8);

pub struct Terminal {
    terminal: wezterm_term::Terminal,
    reader: Receiver<([u8; TERM_SIZE], usize)>,

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
            let mut buf = [0; TERM_SIZE];
            loop {
                let Ok(len) = reader.read(&mut buf) else {
                    return
                };
                let Ok(_) = sender.send((buf, len)) else {
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

    pub fn ui(&mut self, ui: &mut egui::Ui) -> std::io::Result<()> {
        if let Ok((buf, len)) = self.reader.try_recv() {
            self.terminal.advance_bytes(&buf[0..len]);
        }

        let palette = self.terminal.get_config().color_palette();
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, -2.0);

            self.terminal.screen().for_each_phys_line(|i, l| {
                ui.horizontal(|ui| {
                    for c in l.visible_cells() {
                        let attrs = c.attrs();

                        ui.monospace(
                            egui::RichText::new(c.str())
                                .color(palette.resolve_fg(attrs.foreground()).into_egui())
                                .background_color(
                                    palette.resolve_bg(attrs.background()).into_egui(),
                                ),
                        );
                    }
                });
            });

            ui.input(|input| {
                for eve in input.events.iter() {
                    match eve {
                        egui::Event::Key {
                            key,
                            pressed,
                            modifiers,
                            ..
                        } => {
                            if *pressed {
                                self.terminal.key_down(key.into_wez(), modifiers.into_wez());
                            } else {
                                self.terminal.key_up(key.into_wez(), modifiers.into_wez());
                            }
                        }
                        egui::Event::PointerButton {
                            pos,
                            button,
                            pressed,
                            modifiers,
                        } if matches!(
                            *button,
                            egui::PointerButton::Primary
                                | egui::PointerButton::Secondary
                                | egui::PointerButton::Middle
                        ) =>
                        {
                            let event = wezterm_term::MouseEvent {
                                kind: if *pressed {
                                    wezterm_term::MouseEventKind::Press
                                } else {
                                    wezterm_term::MouseEventKind::Release
                                },
                                x: pos.x as usize,
                                y: pos.y as i64,
                                x_pixel_offset: 0,
                                y_pixel_offset: 0,
                                button: match *button {
                                    egui::PointerButton::Primary => wezterm_term::MouseButton::Left,
                                    egui::PointerButton::Secondary => {
                                        wezterm_term::MouseButton::Right
                                    }
                                    egui::PointerButton::Middle => {
                                        wezterm_term::MouseButton::Middle
                                    }
                                    _ => unreachable!(),
                                },
                                modifiers: modifiers.into_wez(),
                            };
                            self.terminal.mouse_event(event);
                        }
                        egui::Event::Scroll(v) => {
                            let event = wezterm_term::MouseEvent {
                                kind: wezterm_term::MouseEventKind::Move,
                                x: 0,
                                y: 0,
                                x_pixel_offset: 0,
                                y_pixel_offset: 0,
                                button: if v.y.is_sign_positive() {
                                    wezterm_term::MouseButton::WheelUp(v.y as _)
                                } else {
                                    wezterm_term::MouseButton::WheelUp(-v.y as _)
                                },
                                modifiers: wezterm_term::KeyModifiers::NONE,
                            };
                            self.terminal.mouse_event(event);
                        }
                        _ => {}
                    }
                }
            });
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

            let mut size = self.terminal.get_size();
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

trait IntoEgui<T> {
    fn into_egui(self) -> T;
}

impl IntoEgui<egui::Color32> for wezterm_term::color::SrgbaTuple {
    fn into_egui(self) -> egui::Color32 {
        let (r, g, b, a) = self.to_srgb_u8();
        egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

trait IntoWez<T> {
    fn into_wez(self) -> T;
}

impl IntoWez<wezterm_term::KeyCode> for egui::Key {
    fn into_wez(self) -> wezterm_term::KeyCode {
        match self {
            egui::Key::ArrowDown => wezterm_term::KeyCode::ApplicationDownArrow,
            egui::Key::ArrowLeft => wezterm_term::KeyCode::ApplicationLeftArrow,
            egui::Key::ArrowRight => wezterm_term::KeyCode::ApplicationRightArrow,
            egui::Key::ArrowUp => wezterm_term::KeyCode::ApplicationUpArrow,
            egui::Key::Escape => wezterm_term::KeyCode::Escape,
            egui::Key::Tab => wezterm_term::KeyCode::Tab,
            egui::Key::Backspace => wezterm_term::KeyCode::Backspace,
            egui::Key::Enter => wezterm_term::KeyCode::Enter,
            egui::Key::Space => wezterm_term::KeyCode::Char(' '),
            egui::Key::Insert => wezterm_term::KeyCode::Insert,
            egui::Key::Delete => wezterm_term::KeyCode::Delete,
            egui::Key::Home => wezterm_term::KeyCode::Home,
            egui::Key::End => wezterm_term::KeyCode::End,
            egui::Key::PageUp => wezterm_term::KeyCode::PageUp,
            egui::Key::PageDown => wezterm_term::KeyCode::PageDown,
            egui::Key::Minus => wezterm_term::KeyCode::Char('-'),
            egui::Key::PlusEquals => wezterm_term::KeyCode::Char('+'),
            egui::Key::Num0 => wezterm_term::KeyCode::Numpad0,
            egui::Key::Num1 => wezterm_term::KeyCode::Numpad1,
            egui::Key::Num2 => wezterm_term::KeyCode::Numpad2,
            egui::Key::Num3 => wezterm_term::KeyCode::Numpad3,
            egui::Key::Num4 => wezterm_term::KeyCode::Numpad4,
            egui::Key::Num5 => wezterm_term::KeyCode::Numpad5,
            egui::Key::Num6 => wezterm_term::KeyCode::Numpad6,
            egui::Key::Num7 => wezterm_term::KeyCode::Numpad7,
            egui::Key::Num8 => wezterm_term::KeyCode::Numpad8,
            egui::Key::Num9 => wezterm_term::KeyCode::Numpad9,
            egui::Key::A => wezterm_term::KeyCode::Char('a'),
            egui::Key::B => wezterm_term::KeyCode::Char('b'),
            egui::Key::C => wezterm_term::KeyCode::Char('c'),
            egui::Key::D => wezterm_term::KeyCode::Char('d'),
            egui::Key::E => wezterm_term::KeyCode::Char('e'),
            egui::Key::F => wezterm_term::KeyCode::Char('f'),
            egui::Key::G => wezterm_term::KeyCode::Char('g'),
            egui::Key::H => wezterm_term::KeyCode::Char('h'),
            egui::Key::I => wezterm_term::KeyCode::Char('i'),
            egui::Key::J => wezterm_term::KeyCode::Char('j'),
            egui::Key::K => wezterm_term::KeyCode::Char('k'),
            egui::Key::L => wezterm_term::KeyCode::Char('l'),
            egui::Key::M => wezterm_term::KeyCode::Char('m'),
            egui::Key::N => wezterm_term::KeyCode::Char('n'),
            egui::Key::O => wezterm_term::KeyCode::Char('o'),
            egui::Key::P => wezterm_term::KeyCode::Char('p'),
            egui::Key::Q => wezterm_term::KeyCode::Char('q'),
            egui::Key::R => wezterm_term::KeyCode::Char('r'),
            egui::Key::S => wezterm_term::KeyCode::Char('s'),
            egui::Key::T => wezterm_term::KeyCode::Char('t'),
            egui::Key::U => wezterm_term::KeyCode::Char('u'),
            egui::Key::V => wezterm_term::KeyCode::Char('v'),
            egui::Key::W => wezterm_term::KeyCode::Char('w'),
            egui::Key::X => wezterm_term::KeyCode::Char('x'),
            egui::Key::Y => wezterm_term::KeyCode::Char('y'),
            egui::Key::Z => wezterm_term::KeyCode::Char('z'),
            egui::Key::F1 => wezterm_term::KeyCode::Function(1),
            egui::Key::F2 => wezterm_term::KeyCode::Function(2),
            egui::Key::F3 => wezterm_term::KeyCode::Function(3),
            egui::Key::F4 => wezterm_term::KeyCode::Function(4),
            egui::Key::F5 => wezterm_term::KeyCode::Function(5),
            egui::Key::F6 => wezterm_term::KeyCode::Function(6),
            egui::Key::F7 => wezterm_term::KeyCode::Function(7),
            egui::Key::F8 => wezterm_term::KeyCode::Function(8),
            egui::Key::F9 => wezterm_term::KeyCode::Function(9),
            egui::Key::F10 => wezterm_term::KeyCode::Function(10),
            egui::Key::F11 => wezterm_term::KeyCode::Function(11),
            egui::Key::F12 => wezterm_term::KeyCode::Function(12),
            egui::Key::F13 => wezterm_term::KeyCode::Function(13),
            egui::Key::F14 => wezterm_term::KeyCode::Function(14),
            egui::Key::F15 => wezterm_term::KeyCode::Function(15),
            egui::Key::F16 => wezterm_term::KeyCode::Function(16),
            egui::Key::F17 => wezterm_term::KeyCode::Function(17),
            egui::Key::F18 => wezterm_term::KeyCode::Function(18),
            egui::Key::F19 => wezterm_term::KeyCode::Function(19),
            egui::Key::F20 => wezterm_term::KeyCode::Function(20),
        }
    }
}

impl IntoWez<wezterm_term::KeyModifiers> for egui::Modifiers {
    fn into_wez(self) -> wezterm_term::KeyModifiers {
        let mut keymod = wezterm_term::KeyModifiers::NONE;
        keymod.set(wezterm_term::KeyModifiers::ALT, self.alt);
        keymod.set(wezterm_term::KeyModifiers::CTRL, self.ctrl);
        keymod.set(wezterm_term::KeyModifiers::SHIFT, self.shift);
        keymod
    }
}
