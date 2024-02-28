// Copyright (C) 2024 Lily Lyons
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
    event_loop::{EventLoopSender, Msg},
    grid::Dimensions,
    sync::FairMutex,
    term::{test::TermSize, Term},
};

pub struct Process {
    term: Arc<FairMutex<Term<WakeupEventListener>>>,
    event_loop_sender: EventLoopSender,
    event_reciever: Receiver<Event>,
}

#[derive(Clone)]
pub struct WakeupEventListener(Sender<Event>, egui::Context);

impl alacritty_terminal::event::EventListener for WakeupEventListener {
    fn send_event(&self, event: Event) {
        println!("Recv event: {event:#?}");
        if let Event::Wakeup = event {
            println!("repainting ui");
            self.1.request_repaint();
        }
        let _ = self.0.send(event);
    }
}

impl Process {
    pub fn new(
        options: &alacritty_terminal::tty::Options,
        ctx: &egui::Context,
    ) -> std::io::Result<Self> {
        let pty = alacritty_terminal::tty::new(
            options,
            WindowSize {
                num_cols: 80,
                num_lines: 24,
                cell_width: 0,
                cell_height: 0,
            },
            0,
        )?;

        let (sender, event_reciever) = std::sync::mpsc::channel();
        let event_proxy = WakeupEventListener(sender, ctx.clone());

        let term_size = TermSize::new(80, 24);
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
        );
        let event_loop_sender = event_loop.channel();
        event_loop.spawn();

        Ok(Self {
            term,
            event_loop_sender,
            event_reciever,
        })
    }
}

impl super::Backend for Process {
    type EventListener = WakeupEventListener;

    fn with_term<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Term<WakeupEventListener>) -> T,
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
        let _ = self.event_loop_sender.send(Msg::Resize(WindowSize {
            num_cols: cols as _,
            num_lines: rows as _,
            cell_height: 0,
            cell_width: 0,
        }));
        self.term.lock().resize(TermSize::new(cols, rows))
    }

    fn send(&mut self, msg: alacritty_terminal::event_loop::Msg) {
        println!("{:?}", msg);
        let _ = self.event_loop_sender.send(msg);
    }

    fn kill(&mut self) {
        let _ = self.event_loop_sender.send(Msg::Shutdown);
    }
}
