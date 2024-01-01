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

pub struct Output {
    pub(super) term_shown: bool,
    term: luminol_term::Terminal,
    save_promise: Option<poll_promise::Promise<luminol_filesystem::Result<()>>>,
    buffer: Vec<u8>,
    byte_rx: luminol_term::ByteReceiver,
}

impl Output {
    pub fn new(term: luminol_term::Terminal, byte_rx: luminol_term::ByteReceiver) -> Self {
        Self {
            term_shown: false,
            save_promise: None,
            buffer: Vec::new(),
            term,
            byte_rx,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut luminol_core::UpdateState<'_>) {
        // We update the output console even if it's not open so that we don't encounter
        // performance problems when the terminal has to parse all the new input at once
        self.term.update();

        for bytes in self.byte_rx.try_iter() {
            self.buffer.extend_from_slice(&bytes);
        }

        egui::Window::new("Output")
            .id(self.term.id())
            .open(&mut self.term_shown)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    let mut resize = false;
                    let (mut cols, mut rows) = self.term.size();

                    resize |= ui.add(egui::DragValue::new(&mut cols)).changed();
                    ui.label("×");
                    resize |= ui.add(egui::DragValue::new(&mut rows)).changed();

                    if resize {
                        self.term.set_size(update_state, cols, rows);
                    }

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
                        let buffer = self.buffer.clone();

                        self.save_promise = Some(luminol_core::spawn_future(async {
                            use futures_lite::AsyncWriteExt;

                            let mut tmp = luminol_filesystem::host::File::new()?;
                            let mut cursor = async_std::io::Cursor::new(buffer);
                            async_std::io::copy(&mut cursor, &mut tmp).await?;
                            tmp.flush().await?;
                            tmp.save("luminol.log", "Log files").await?;
                            Ok(())
                        }));
                    }
                });

                ui.separator();

                if let Err(e) = self.term.ui(ui) {
                    luminol_core::error!(
                        update_state.toasts,
                        color_eyre::eyre::eyre!(e).wrap_err("Error displaying output console"),
                    );
                }
            });
    }
}
