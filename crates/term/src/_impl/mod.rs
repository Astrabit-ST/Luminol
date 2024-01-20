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

use std::sync::Arc;

use alacritty_terminal::{
    event::Event,
    event_loop::{EventLoop, EventLoopSender},
    sync::FairMutex,
    term::{
        cell::{Cell, Flags},
        Config,
    },
    Grid, Term,
};

type ArcMutex<T> = Arc<FairMutex<T>>;

mod into;

pub struct Terminal {
    state: ArcMutex<TerminalState>,
    term: ArcMutex<Term<EventListener>>,
    event_loop_sender: EventLoopSender,
}

#[derive(Debug)]
struct TerminalState {
    title: String,
}

#[derive(Clone)]
struct EventListener {
    state: ArcMutex<TerminalState>,
}

#[allow(clippy::single_match)]
impl alacritty_terminal::event::EventListener for EventListener {
    fn send_event(&self, event: Event) {
        let mut state = self.state.lock();
        println!("{event:#?}");
        match event {
            Event::Title(title) => state.title = title,
            _ => {}
        }
    }
}

impl Terminal {
    pub fn new(ctx: &egui::Context) -> std::io::Result<Self> {
        let pty = alacritty_terminal::tty::new(
            &alacritty_terminal::tty::Options::default(),
            // todo what do these do
            alacritty_terminal::event::WindowSize {
                num_cols: 80,
                num_lines: 24,
                // dummy values for now
                cell_width: 12,
                cell_height: 12,
            },
            0,
        )?;

        let state = Arc::new(FairMutex::new(TerminalState {
            title: "Terminal".to_string(),
        }));

        // ???
        let grid = Grid::<Cell>::new(24, 80, 100);
        let term = Term::new(
            Config::default(),
            &grid,
            EventListener {
                state: state.clone(),
            },
        );
        let term = Arc::new(FairMutex::new(term));

        let event_loop = EventLoop::new(
            term.clone(),
            EventListener {
                state: state.clone(),
            },
            pty,
            false,
            false,
        );
        let event_loop_sender = event_loop.channel();
        event_loop.spawn(); // todo: do we need to keep this join handle around?

        Ok(Self {
            state,
            term,
            event_loop_sender,
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
        let state = self.state.lock();
        state.title.clone()
    }

    pub fn id(&self) -> egui::Id {
        // todo!()
        egui::Id::new("luminol_term_terminal")
    }

    pub fn set_size(
        &mut self,
        update_state: &mut luminol_core::UpdateState<'_>,
        cols: usize,
        rows: usize,
    ) {
        todo!()
    }

    pub fn set_cols(&mut self, update_state: &mut luminol_core::UpdateState<'_>, cols: usize) {
        todo!()
    }

    pub fn set_rows(&mut self, update_state: &mut luminol_core::UpdateState<'_>, rows: usize) {
        todo!()
    }

    pub fn size(&self) -> (usize, usize) {
        // todo!()
        (80, 24)
    }

    pub fn cols(&self) -> usize {
        todo!()
    }

    pub fn rows(&self) -> usize {
        todo!()
    }

    pub fn erase_scrollback(&mut self) {
        todo!()
    }

    pub fn erase_scrollback_and_viewport(&mut self) {
        todo!()
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> color_eyre::Result<()> {
        // todo!()
        let term = self.term.lock();
        let content = term.renderable_content();

        let mut job = egui::text::LayoutJob::default();
        for cell in content.display_iter {
            let mut buf = [0; 4];
            let text = cell.c.encode_utf8(&mut buf);

            let format = egui::TextFormat {
                font_id: egui::FontId::monospace(12.),
                color: into::color_to_egui(cell.fg),
                background: into::color_to_egui(cell.bg),
                ..Default::default()
            };

            job.append(text, 0.0, format);

            if cell.point.column >= 79 {
                job.append("\n", 0.0, Default::default());
            }
        }
        let galley = ui.fonts(|f| f.layout_job(job));
        let (response, painter) =
            ui.allocate_painter(galley.rect.size(), egui::Sense::click_and_drag());

        painter.rect_filled(
            galley.rect.translate(response.rect.min.to_vec2()),
            0.0,
            egui::Color32::BLACK,
        );

        painter.galley(response.rect.min, galley);

        Ok(())
    }

    pub fn kill(&mut self) {
        // todo!()
        self.term.lock().exit()
    }
}
