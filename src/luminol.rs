// Copyright (C) 2022 Lily Lyons
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

use crate::lumi::Lumi;
use crate::prelude::*;
use rfd::{MessageDialog, MessageLevel};
use std::process;

const TITLEBAR_HEIGHT: f32 = 24.;
const BUTTON_SIZE: f32 = 18.;

/// The main Luminol struct. Handles rendering, GUI state, that sort of thing.
pub struct Luminol {
    top_bar: TopBar,
    style: Arc<egui::Style>,
    lumi: Lumi,
    icon: RetainedImage,
}

impl Luminol {
    /// Called once before the first frame.
    fn new(cc: &eframe::CreationContext<'_>, try_load_path: Option<std::ffi::OsString>) -> Self {
        let icon = RetainedImage::from_image_bytes("", crate::ICON).unwrap();
        let storage = cc.storage.unwrap();

        let state = eframe::get_value(storage, "SavedState").unwrap_or_default();
        let style =
            eframe::get_value(storage, "EguiStyle").map_or_else(|| cc.egui_ctx.style(), |s| s);
        cc.egui_ctx.set_style(style.clone());

        let info = Interfaces::new(cc.gl.as_ref().unwrap().clone(), state);
        crate::set_state(info);

        if let Some(path) = try_load_path {
            interfaces!()
                .filesystem
                .try_open_project(path)
                .expect("failed to load project");
        }

        let lumi = Lumi::new().expect("failed to load lumi images");

        Self {
            top_bar: TopBar::default(),
            style,
            lumi,
            icon,
        }
    }

    /// Run Luminol.
    pub fn run(try_load_path: Option<std::ffi::OsString>) {
        let image = image::load_from_memory(crate::ICON).expect("Failed to load Icon data.");
        let native_options = eframe::NativeOptions {
            drag_and_drop_support: true,
            transparent: true,
            icon_data: Some(eframe::IconData {
                width: image.width(),
                height: image.height(),
                rgba: image.into_bytes(),
            }),
            decorated: false,
            ..Default::default()
        };

        if let Err(why) = eframe::run_native(
            "Luminol",
            native_options,
            Box::new(|cc| Box::new(Self::new(cc, try_load_path))),
        ) {
            MessageDialog::new()
                .set_title("Luminol")
                .set_description(format!("Failed to launch Luminol: {why}"))
                .set_level(MessageLevel::Error)
                .show();
            process::exit(1);
        }
    }
}

impl eframe::App for Luminol {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value::<crate::SavedState>(
            storage,
            "SavedState",
            &interfaces!().saved_state.borrow(),
        );
        eframe::set_value::<Arc<egui::Style>>(storage, "EguiStyle", &self.style);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: self.style.visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                let app_rect = ui.max_rect();
                let bar_rect = {
                    let mut rect = app_rect;
                    rect.max.y = rect.min.y + TITLEBAR_HEIGHT;
                    rect
                };
                let content_rect = {
                    let mut rect = app_rect;
                    rect.min.y = bar_rect.max.y;
                    rect
                }
                .shrink(4.);

                let painter = ui.painter();

                // Titlebar text
                painter.text(
                    bar_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    format!("Luminol Editor v{}", env!("CARGO_PKG_VERSION")),
                    egui::FontId::proportional(12.),
                    ui.style().visuals.text_color(),
                );

                let bar_response =
                    ui.interact(bar_rect, egui::Id::new("title_bar"), egui::Sense::click());

                if bar_response.double_clicked() {
                    frame.set_maximized(!frame.info().window_info.maximized);
                } else if bar_response.is_pointer_button_down_on() {
                    frame.drag_window();
                }

                fn button(
                    ui: &mut egui::Ui,
                    text: impl Into<String>,
                    on_hover_text: impl Into<egui::WidgetText>,
                ) -> egui::Response {
                    ui.add(egui::Button::new(
                        egui::RichText::new(text).size(BUTTON_SIZE),
                    ))
                    .on_hover_text(on_hover_text)
                }

                ui.allocate_ui_at_rect(bar_rect, |ui| {
                    ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                        ui.add_space(8.);

                        ui.image(self.icon.texture_id(ctx), egui::vec2(16., 16.));
                    });
                });
                ui.allocate_ui_at_rect(bar_rect, |ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().item_spacing.x = 0.;
                        ui.visuals_mut().button_frame = false;
                        ui.add_space(8.);

                        let window_info = frame.info().window_info;

                        if button(ui, "❌", "Exit Luminol").clicked() {
                            frame.close();
                        }
                        if button(
                            ui,
                            "🗗",
                            if window_info.maximized {
                                "Minimize window"
                            } else {
                                "Maximize window"
                            },
                        )
                        .clicked()
                        {
                            frame.set_maximized(!window_info.maximized);
                        }
                        if button(ui, "🗕", "Hide window").clicked() {
                            frame.set_minimized(true);
                        }
                    });
                });

                *ui = ui.child_ui(content_rect, *ui.layout());

                ctx.input(|i| {
                    if let Some(f) = i.raw.dropped_files.first() {
                        let path = f.path.clone().expect("dropped file has no path");

                        if let Err(e) = interfaces!().filesystem.try_open_project(path) {
                            interfaces!()
                                .toasts
                                .error(format!("Error opening dropped project: {e}"))
                        }
                    }
                });

                egui::TopBottomPanel::top("top_toolbar").show_inside(ui, |ui| {
                    // We want the top menubar to be horizontal. Without this it would fill up vertically.
                    ui.horizontal_wrapped(|ui| {
                        // Turn off button frame.
                        ui.visuals_mut().button_frame = false;
                        // Show the bar
                        self.top_bar.ui(ui, &mut self.style, frame);
                    });
                });

                // Central panel with tabs.
                egui::CentralPanel::default().show_inside(ui, |ui| {
                    interfaces!().tabs.ui(ui);
                });

                // Update all windows.
                interfaces!().windows.update(ctx);

                // Show toasts.
                interfaces!().toasts.show(ctx);

                poll_promise::tick_local();

                self.lumi.ui(ctx);
            });
    }

    fn persist_egui_memory(&self) -> bool {
        true
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}
