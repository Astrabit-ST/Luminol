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

use std::sync::mpsc::Receiver;

use alacritty_terminal::{
    event::Event,
    grid::Dimensions,
    term::{test::TermSize, Term},
    vte,
};

use super::EventListener;

pub struct Channel {
    processor: vte::ansi::Processor,
    term: Term<EventListener>,
    event_reciever: Receiver<Event>,
    byte_recv: Receiver<u8>,
}

impl Channel {
    pub fn new(byte_recv: Receiver<u8>) -> Self {
        let processor = vte::ansi::Processor::new();

        let (sender, event_reciever) = std::sync::mpsc::channel();
        let event_proxy = EventListener(sender);

        let term_size = TermSize::new(80, 24);
        let term = Term::new(
            alacritty_terminal::term::Config::default(),
            &term_size,
            event_proxy,
        );

        Self {
            processor,
            term,
            event_reciever,
            byte_recv,
        }
    }
}

impl super::Backend for Channel {
    fn with_term<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Term<EventListener>) -> T,
    {
        f(&mut self.term)
    }

    fn with_event_recv<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Receiver<Event>) -> T,
    {
        f(&mut self.event_reciever)
    }

    fn size(&self) -> (usize, usize) {
        (self.term.columns(), self.term.screen_lines())
    }

    fn resize(&mut self, rows: usize, cols: usize) {
        self.term.resize(TermSize::new(cols, rows))
    }

    fn update(&mut self) {
        for byte in self.byte_recv.try_iter() {
            self.processor.advance(&mut self.term, byte);
        }
    }
}
