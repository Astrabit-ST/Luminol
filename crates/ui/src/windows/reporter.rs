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

/// Crash reporter window.
pub struct Window {
    normalized_report: String,
    json: ReportJson,
    send_promise: Option<poll_promise::Promise<color_eyre::Result<()>>>,
}

#[derive(Debug, Clone, serde::Serialize)]
struct ReportJson {
    reporter_version: u32,
    luminol_version: String,
    target: String,
    debug: bool,
    report: String,
}

impl Window {
    pub fn new(report: impl Into<String>) -> Self {
        let report: String = report.into();

        Self {
            normalized_report: strip_ansi_escapes::strip_str(&report),
            json: ReportJson {
                reporter_version: 1,
                luminol_version: git_version::git_version!().to_string(),
                target: target_triple::target!().to_string(),
                #[cfg(debug_assertions)]
                debug: true,
                #[cfg(not(debug_assertions))]
                debug: false,
                report,
            },
            send_promise: None,
        }
    }
}

impl luminol_core::Window for Window {
    fn name(&self) -> String {
        "Crash Reporter".into()
    }

    fn id(&self) -> egui::Id {
        egui::Id::new("reporter")
    }

    fn requires_filesystem(&self) -> bool {
        false
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let mut should_close = false;

        egui::Window::new(self.name())
            .id(egui::Id::new("reporter"))
            .default_width(500.)
            .open(open)
            .show(ctx, |ui| {
                ui.label("Luminol has crashed!");
                ui.label(
                    "Would you like to send the following crash report to the Luminol developers?",
                );

                ui.add_space(ui.spacing().indent);

                ui.label(format!("Luminol version: {}", self.json.luminol_version));
                ui.label(format!("Target platform: {}", self.json.target));
                ui.label(format!(
                    "Build profile: {}",
                    if self.json.debug { "debug" } else { "release" }
                ));

                ui.group(|ui| {
                    ui.with_layout(
                        egui::Layout {
                            cross_justify: true,
                            ..Default::default()
                        },
                        |ui| {
                            egui::ScrollArea::both()
                                .max_height(ui.spacing().interact_size.y.max(
                                    ui.text_style_height(&egui::TextStyle::Button)
                                        + 2. * ui.spacing().button_padding.y,
                                ))
                                .show(ui, |ui| {
                                    ui.add(
                                        egui::TextEdit::multiline(
                                            &mut self.normalized_report.as_str(),
                                        )
                                        .layouter(
                                            &mut |ui, text, wrap_width| {
                                                // Make the text monospace and non-wrapping
                                                egui::WidgetText::from(text)
                                                    .color(
                                                        ui.visuals()
                                                            .override_text_color
                                                            .unwrap_or_else(|| {
                                                                ui.visuals()
                                                                    .widgets
                                                                    .noninteractive
                                                                    .fg_stroke
                                                                    .color
                                                            }),
                                                    )
                                                    .into_galley(
                                                        ui,
                                                        Some(false),
                                                        wrap_width,
                                                        egui::TextStyle::Monospace,
                                                    )
                                                    .galley
                                            },
                                        ),
                                    );
                                });
                        },
                    );
                });

                ui.with_layout(
                    egui::Layout {
                        cross_justify: true,
                        cross_align: egui::Align::Center,
                        ..Default::default()
                    },
                    |ui| {
                        if self.send_promise.is_none() {
                            ui.columns(2, |columns| {
                                if columns[0].button("Don't send").clicked() {
                                    should_close = true;
                                }

                                if columns[1].button("Send").clicked() {
                                    let json = self.json.clone();
                                    self.send_promise =
                                        Some(luminol_core::spawn_future(async move {
                                            let client = reqwest::Client::new();
                                            let response = client
                                                .post("http://localhost:3246")
                                                .json(&json)
                                                .fetch_mode_no_cors()
                                                .send()
                                                .await
                                                .map_err(|e| color_eyre::eyre::eyre!(e))?;
                                            if response.status().is_success() {
                                                Ok(())
                                            } else {
                                                Err(color_eyre::eyre::eyre!(format!(
                                                    "Request returned {}",
                                                    response.status()
                                                )))
                                            }
                                        }));
                                }
                            });
                        } else {
                            ui.spinner();
                        }
                    },
                );
            });

        if let Some(p) = self.send_promise.take() {
            match p.try_take() {
                Ok(Ok(())) => {
                    luminol_core::info!(update_state.toasts, "Crash report sent!");
                    should_close = true;
                }
                Ok(Err(e)) => {
                    luminol_core::error!(
                        update_state.toasts,
                        e.wrap_err("Error sending crash report")
                    );
                }
                Err(p) => self.send_promise = Some(p),
            }
        }

        if should_close {
            *open = false;
        }
    }
}
