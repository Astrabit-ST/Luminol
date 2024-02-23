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

mod channel;
mod process;

use alacritty_terminal::event::Event;
use std::sync::mpsc::{Receiver, Sender};

pub use channel::Channel;
pub use process::Process;

#[derive(Clone)]
pub struct EventListener(Sender<Event>);

impl alacritty_terminal::event::EventListener for EventListener {
    fn send_event(&self, event: Event) {
        println!("Recv event: {event:#?}");
        let _ = self.0.send(event);
    }
}

pub trait Backend {
    fn with_term<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut alacritty_terminal::Term<EventListener>) -> T;

    fn with_event_recv<T, F>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Receiver<Event>) -> T;

    fn size(&self) -> (usize, usize);

    fn resize(&mut self, rows: usize, cols: usize);

    fn update(&mut self) {}

    fn send(&mut self, _msg: alacritty_terminal::event_loop::Msg) {}

    fn kill(&mut self) {}
}
