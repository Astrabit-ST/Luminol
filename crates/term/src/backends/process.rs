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

use std::sync::{
    mpsc::{Receiver, Sender},
    Arc,
};

use alacritty_terminal::{
    event::{Event, Notify, WindowSize},
    event_loop::{Msg, Notifier},
    grid::Dimensions,
    sync::FairMutex,
    term::{test::TermSize, Term},
};

use super::Backend;

pub struct Process {
    term: Arc<FairMutex<Term<ForwardEventListener>>>,
    notifier: Notifier,
    event_reciever: Receiver<Event>,
}

#[derive(Clone)]
pub struct ForwardEventListener(Sender<Event>, egui::Context);

impl alacritty_terminal::event::EventListener for ForwardEventListener {
    fn send_event(&self, event: Event) {
        let needs_repaint = matches!(event, Event::Wakeup);
        let _ = self.0.send(event);

        if needs_repaint {
            self.1.request_repaint();
        }
    }
}

impl Process {
    pub fn new(
        options: &alacritty_terminal::tty::Options,
        update_state: &luminol_core::UpdateState<'_>,
    ) -> std::io::Result<Self> {
        let config = &update_state.global_config.terminal;
        let pty = alacritty_terminal::tty::new(
            options,
            WindowSize {
                num_cols: config.initial_size.0,
                num_lines: config.initial_size.1,
                cell_width: 0,
                cell_height: 0,
            },
            0,
        )?;

        let (sender, event_reciever) = std::sync::mpsc::channel();
        let event_proxy = ForwardEventListener(sender, update_state.ctx.clone());

        let term_size = TermSize::new(
            config.initial_size.0 as usize,
            config.initial_size.1 as usize,
        );
        let term = Term::new(
            alacritty_terminal::term::Config::default(),
            &term_size,
            event_proxy.clone(),
        );
        let term = Arc::new(FairMutex::new(term));

        let event_loop = alacritty_terminal::event_loop::EventLoop::new(
            term.clone(),
            event_proxy,
            pty,
            false,
            false,
        )?;
        let event_loop_sender = event_loop.channel();
        let notifier = Notifier(event_loop_sender);
        event_loop.spawn();

        Ok(Self {
            term,
            notifier,
            event_reciever,
        })
    }
}

impl Backend for Process {
    type EventListener = ForwardEventListener;

    fn with_term<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Term<ForwardEventListener>) -> T,
    {
        f(&mut self.term.lock())
    }

    fn with_event_recv<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Receiver<Event>) -> T,
    {
        f(&mut self.event_reciever)
    }

    fn size(&self) -> (usize, usize) {
        let term = self.term.lock();
        (term.columns(), term.screen_lines())
    }

    fn resize(&mut self, rows: usize, cols: usize) {
        let _ = self.notifier.0.send(Msg::Resize(WindowSize {
            num_cols: cols as _,
            num_lines: rows as _,
            cell_height: 0,
            cell_width: 0,
        }));
        self.term.lock().resize(TermSize::new(cols, rows))
    }

    fn send(&mut self, bytes: impl Into<std::borrow::Cow<'static, [u8]>>) {
        let bytes = bytes.into();
        self.notifier.notify(bytes);
    }

    fn kill(&mut self) {
        let _ = self.notifier.0.send(Msg::Shutdown);
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.kill()
    }
}
