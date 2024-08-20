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

use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, Sender},
};

pub struct LogWindow {
    pub(super) term_shown: bool,
    byte_rx: Receiver<u8>,
    byte_tx: Sender<u8>,
    buffer: VecDeque<u8>,
    term: luminol_term::widget::ChannelTerminal,
    save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
}
const MAX_BUFFER_CAPACITY: usize = 1 << 24;

impl LogWindow {
    pub fn new(
        config: &luminol_config::terminal::Config,
        byte_rx: std::sync::mpsc::Receiver<u8>,
    ) -> Self {
        let (byte_tx, term_byte_rx) = std::sync::mpsc::channel();
        let term = luminol_term::widget::Terminal::channel(term_byte_rx, config);

        Self {
            byte_rx,
            byte_tx,
            buffer: VecDeque::new(),
            term,
            term_shown: false,
            save_promise: None,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
        let mut did_recv = false;
        for byte in self.byte_rx.try_iter() {
            let _ = self.byte_tx.send(byte);

            if self.buffer.len() >= MAX_BUFFER_CAPACITY {
                self.buffer.pop_front();
                self.buffer.push_back(byte);
            } else {
                self.buffer.push_back(byte);
            }

            did_recv = true;
        }

        if did_recv {
            update_state.ctx.request_repaint();
        }

        // We update the log terminal even if it's not open so that we don't encounter
        // performance problems when the terminal has to parse all the new input at once
        self.term.update();

        egui::Window::new("Log")
            .id(self.term.id)
            .open(&mut self.term_shown)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(ui.style().spacing.indent);

                    if ui.button("Clear").clicked() {
                        self.buffer.clear();
                        self.term.erase_scrollback_and_viewport();
                    }

                    if let Some(p) = self.save_promise.take() {
                        ui.spinner();

                        match p.try_take() {
                            Ok(Ok(())) => {
                                luminol_core::info!(update_state.toasts, "Successfully saved log!")
                            }
                            Ok(Err(e))
                                if !matches!(
                                    e.root_cause().downcast_ref(),
                                    Some(luminol_filesystem::Error::CancelledLoading)
                                ) =>
                            {
                                luminol_core::error!(
                                    update_state.toasts,
                                    color_eyre::eyre::eyre!(e)
                                        .wrap_err("Error saving the log to a file")
                                )
                            }
                            Ok(Err(_)) => {}
                            Err(p) => self.save_promise = Some(p),
                        }
                    } else if ui.button("Save to file").clicked() {
                        let buffer = self.buffer.make_contiguous().to_vec();

                        self.save_promise =
                            Some(luminol_core::spawn_future(Self::save_promise(buffer)));
                    }
                });

                ui.add_space(ui.spacing().item_spacing.y);

                if let Err(e) = self.term.ui(update_state, ui) {
                    luminol_core::error!(
                        update_state.toasts,
                        e.wrap_err("Error displaying log window"),
                    );
                }
            });
    }

    async fn save_promise(buffer: Vec<u8>) -> luminol_filesystem::Result<()> {
        use futures_lite::AsyncWriteExt;

        let mut tmp = luminol_filesystem::host::File::new()?;
        let mut cursor = async_std::io::Cursor::new(buffer);
        async_std::io::copy(&mut cursor, &mut tmp).await?;
        tmp.flush().await?;
        tmp.save("luminol.log", "Log files").await?;
        Ok(())
    }
}
