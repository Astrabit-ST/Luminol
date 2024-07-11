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

use strum::IntoEnumIterator;

pub struct Window {
    selected_data_format: luminol_config::DataFormat,
}

impl Window {
    pub fn new(config: &luminol_config::project::Config) -> Self {
        Self {
            selected_data_format: config.project.data_format,
        }
    }
}

const FORMAT_WARNING: &str = "While the option is provided,\nLuminol cannot convert between formats yet.\nIt can still read other formats, however."; // "Luminol will need to convert your project.\nThis is not 100% safe yet, make backups!\nPress OK to continue.";

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("project_config_window")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        let Some(config) = update_state.project_config.as_mut() else {
            *open = false;

            return;
        };

        egui::Window::new("Project Config")
            .open(open)
            .show(ctx, |ui| {
                ui.label("Editor Settings");
                ui.group(|ui| {
                    ui.label("Project name");
                    ui.text_edit_singleline(&mut config.project.project_name);
                    ui.label("Scripts path (editor)")
                        .on_hover_text("Applies to Luminol (not your game!)");
                    ui.text_edit_singleline(&mut config.project.scripts_path);
                    ui.label("Playtest Executable");
                    ui.text_edit_singleline(&mut config.project.playtest_exe);

                    ui.separator();

                    egui::ComboBox::from_label("Data Format")
                        .selected_text(self.selected_data_format.to_string())
                        .show_ui(ui, |ui| {
                            for format in luminol_config::DataFormat::iter() {
                                ui.selectable_value(
                                    &mut self.selected_data_format,
                                    format,
                                    format.to_string(),
                                );
                            }
                        });

                    if self.selected_data_format != config.project.data_format {
                        // add warning message about needing to edit every single data file
                        egui::Frame::none().show(ui, |ui| {
                            ui.style_mut()
                                .visuals
                                .widgets
                                .noninteractive
                                .bg_stroke
                                .color = ui.style().visuals.warn_fg_color;

                            egui::Frame::group(ui.style())
                                .fill(ui.visuals().gray_out(ui.visuals().gray_out(
                                    ui.visuals().gray_out(ui.style().visuals.warn_fg_color),
                                )))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(
                                        egui::RichText::new(FORMAT_WARNING)
                                            .color(ui.style().visuals.warn_fg_color),
                                    );
                                });
                        });

                        let clicked = ui
                            .button(
                                egui::RichText::new("Ok").color(ui.style().visuals.error_fg_color),
                            )
                            .clicked();
                        if clicked {
                            config.project.data_format = self.selected_data_format;
                            // TODO add conversion logic
                        }
                    }

                    ui.separator();

                    egui::ComboBox::from_label("RGSS Version")
                        .selected_text(config.project.rgss_ver.to_string())
                        .show_ui(ui, |ui| {
                            for ver in luminol_config::RGSSVer::iter() {
                                ui.selectable_value(
                                    &mut config.project.rgss_ver,
                                    ver,
                                    ver.to_string(),
                                );
                            }
                        });
                });

                ui.label("Game.ini settings");

                ui.group(|ui| {
                    // rust-ini doesn't provide any kind of API for mutably accessing properties, so this is the best we can do.
                    // we temporarily remove the properties from the game ini and then re-insert it after we're done editing it.
                    let general_section = config.game_ini.general_section_mut();

                    let mut game_title = general_section.remove("Title").unwrap_or_default();
                    ui.label("Title");
                    ui.text_edit_singleline(&mut game_title);
                    general_section.insert("Title", game_title);

                    ui.separator();

                    for rtp in ["RTP1", "RTP2", "RTP3"] {
                        let mut rtp_name = general_section.remove(rtp).unwrap_or_default();
                        ui.label(rtp);
                        ui.text_edit_singleline(&mut rtp_name).on_hover_text(
                            "You may have to reload the project for changes to take effect",
                        );
                        general_section.insert(rtp, rtp_name);
                    }

                    ui.separator();

                    let mut scripts_path = general_section.remove("Scripts").unwrap_or_default();

                    ui.label("Scripts path (runtime)");
                    ui.text_edit_singleline(&mut scripts_path)
                        .on_hover_text("Applies only to your game (not Luminol!)");
                    general_section.insert("Scripts", scripts_path);
                });
            });
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
