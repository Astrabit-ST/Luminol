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

use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use alacritty_terminal::{
    event::{Event, WindowSize},
    event_loop::{EventLoop, EventLoopSender, Msg},
    grid::Dimensions,
    sync::FairMutex,
    term::{test::TermSize, Config},
    Term,
};

use crate::theme::Theme;

pub struct Terminal {
    reciever: Receiver<Event>,
    term: Arc<FairMutex<Term<EventListener>>>,
    event_loop_sender: EventLoopSender,

    theme: Theme, // TODO convert into shared config (possibly do this in luminol-preferences)
    title: String,
}

#[derive(Clone)]
struct EventListener(Sender<Event>);

impl alacritty_terminal::event::EventListener for EventListener {
    fn send_event(&self, event: Event) {
        println!("{event:#?}");
        self.0
            .send(event)
            // panic here in case we failed to send an event, which probably means the event loop thread should stop
            .expect("failed to send event (reciever closed?)")
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        // don't panic in case the event loop thread is paused
        let _recv = self.event_loop_sender.send(Msg::Shutdown);
    }
}

fn context_monospace_char_size(ctx: &egui::Context) -> (f32, f32) {
    ctx.fonts(|f| {
        (
            f.glyph_width(&egui::FontId::monospace(12.), ' '),
            f.row_height(&egui::FontId::monospace(12.)),
        )
    })
}

impl Terminal {
    pub fn new(
        ctx: &egui::Context,
        config: &alacritty_terminal::tty::Options,
    ) -> std::io::Result<Self> {
        let (cell_width, cell_height) = context_monospace_char_size(ctx);

        let pty = alacritty_terminal::tty::new(
            config,
            WindowSize {
                num_cols: 80,
                num_lines: 24,
                cell_width: cell_width as _,
                cell_height: cell_height as _,
            },
            0,
        )?;

        let (sender, reciever) = std::sync::mpsc::channel();
        let event_proxy = EventListener(sender);

        // ???
        let term_size = TermSize::new(80, 24);
        let term = Term::new(Config::default(), &term_size, event_proxy.clone());
        let term = Arc::new(FairMutex::new(term));

        let event_loop = EventLoop::new(term.clone(), event_proxy, pty, false, false);
        let event_loop_sender = event_loop.channel();
        event_loop.spawn(); // FIXME: do we need to keep this join handle around?

        Ok(Self {
            reciever,
            term,
            event_loop_sender,

            theme: Theme::default(),
            title: "Luminol Terminal".to_string(),
        })
    }

    pub fn new_readonly(
        ctx: &egui::Context,
        id: egui::Id,
        title: impl Into<String>,
        default_cols: usize,
        default_rows: usize,
    ) -> Self {
        todo!()
    }

    pub fn title(&self) -> String {
        // todo!()
        self.title.to_string()
    }

    pub fn id(&self) -> egui::Id {
        // todo!()
        egui::Id::new("luminol_term_terminal")
    }

    pub fn set_size(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        cols: usize,
        lines: usize,
    ) {
        let (cell_width, cell_height) = context_monospace_char_size(update_state.ctx);

        // ignore any send errors
        let _ = self.event_loop_sender.send(Msg::Resize(WindowSize {
            num_lines: lines as _,
            num_cols: cols as _,
            cell_width: cell_width as _,
            cell_height: cell_height as _,
        }));
        self.term.lock().resize(TermSize::new(cols, lines))
    }

    pub fn set_cols(&mut self, update_state: &mut luminol_core::UpdateState<'_>, cols: usize) {
        let lines = self.term.lock().screen_lines();
        self.set_size(update_state, cols, lines)
    }

    pub fn set_rows(&mut self, update_state: &mut luminol_core::UpdateState<'_>, rows: usize) {
        let cols = self.term.lock().columns();
        self.set_size(update_state, cols, rows)
    }

    pub fn size(&self) -> (usize, usize) {
        // todo!()
        let term = self.term.lock();
        (term.columns(), term.screen_lines())
    }

    pub fn cols(&self) -> usize {
        self.term.lock().columns()
    }

    pub fn rows(&self) -> usize {
        self.term.lock().screen_lines()
    }

    pub fn erase_scrollback(&mut self) {
        self.term.lock().grid_mut().clear_history();
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        // TODO maybe reset() is better?
        let mut term = self.term.lock();
        term.grid_mut().clear_history();
        term.grid_mut().clear_viewport();
    }

    pub fn update(&mut self) {
        for event in self.reciever.try_iter() {
            match event {
                // we could use clone_from/clone_into to save resources but it's not necessary here
                Event::Title(title) => self.title = title,
                Event::ResetTitle => self.title = "Luminol Terminal".to_string(),
                _ => {}
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> color_eyre::Result<()> {
        self.update();

        let mut term = self.term.lock();
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
        let focused = response.has_focus();

        ui.input(|input| {
            if !focused {
                return;
            }

            for event in input.events.iter() {
                match event {
                    egui::Event::Scroll(pos) => {
                        term.scroll_display(alacritty_terminal::grid::Scroll::Delta(pos.y as _));
                    }
                    egui::Event::Text(t) => {
                        let cow = t.to_string().into_bytes().into();
                        let _ = self.event_loop_sender.send(Msg::Input(cow));
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    pub fn kill(&mut self) {
        // todo!()
        self.term.lock().exit();
        let _recv = self.event_loop_sender.send(Msg::Shutdown);
    }
}
