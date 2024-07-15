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

use async_std::io::{ReadExt, WriteExt};
use egui::Widget;
use itertools::Itertools;
use luminol_filesystem::{FileSystem, OpenFlags};
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::IntoEnumIterator;

pub struct Window {
    selected_data_format: luminol_config::DataFormat,
    convert: Option<Convert>,
}

struct Convert {
    promise: poll_promise::Promise<color_eyre::Result<()>>,
    progress: Progress,
}
type Progress = std::sync::Arc<(AtomicUsize, AtomicUsize)>;

impl Window {
    pub fn new(config: &luminol_config::project::Config) -> Self {
        Self {
            selected_data_format: config.project.data_format,
            convert: None,
        }
    }
}

const FORMAT_WARNING: &str = "Luminol will need to convert your project.\nThis is not 100% safe yet, make backups!\nPress OK to continue.";

fn convert_project(
    config: &mut luminol_config::project::Config,
    selected_data_format: luminol_config::DataFormat,
    data_cache: &mut luminol_core::Data,
    filesystem: &mut luminol_filesystem::project::FileSystem,
    progress: Progress,
) -> color_eyre::Result<impl std::future::Future<Output = color_eyre::Result<()>>> {
    let from_handler = luminol_core::data_formats::Handler::new(config.project.data_format);
    let to_handler = luminol_core::data_formats::Handler::new(selected_data_format);

    let pretty_config = ron::ser::PrettyConfig::new()
        .struct_names(true)
        .enumerate_arrays(true);
    config.project.data_format = selected_data_format;
    let project_config = ron::ser::to_string_pretty(&config.project, pretty_config)?;
    filesystem.write(".luminol/config", project_config)?;

    to_handler.write_nil_padded(&data_cache.actors().data, filesystem, "Actors")?;
    from_handler.remove_file(filesystem, "Actors")?;

    to_handler.write_nil_padded(&data_cache.animations().data, filesystem, "Animations")?;
    from_handler.remove_file(filesystem, "Animations")?;

    to_handler.write_nil_padded(&data_cache.armors().data, filesystem, "Armors")?;
    from_handler.remove_file(filesystem, "Armors")?;

    to_handler.write_nil_padded(&data_cache.classes().data, filesystem, "Classes")?;
    from_handler.remove_file(filesystem, "Classes")?;

    to_handler.write_nil_padded(&data_cache.common_events().data, filesystem, "CommonEvents")?;
    from_handler.remove_file(filesystem, "CommonEvents")?;

    to_handler.write_nil_padded(&data_cache.enemies().data, filesystem, "Enemies")?;
    from_handler.remove_file(filesystem, "Enemies")?;

    to_handler.write_nil_padded(&data_cache.items().data, filesystem, "Items")?;
    from_handler.remove_file(filesystem, "Items")?;

    to_handler.write_nil_padded(&data_cache.skills().data, filesystem, "Skills")?;
    from_handler.remove_file(filesystem, "Skills")?;

    to_handler.write_nil_padded(&data_cache.states().data, filesystem, "States")?;
    from_handler.remove_file(filesystem, "States")?;

    to_handler.write_nil_padded(&data_cache.tilesets().data, filesystem, "Tilesets")?;
    from_handler.remove_file(filesystem, "Tilesets")?;

    to_handler.write_nil_padded(&data_cache.troops().data, filesystem, "Troops")?;
    from_handler.remove_file(filesystem, "Troops")?;

    to_handler.write_nil_padded(&data_cache.weapons().data, filesystem, "Weapons")?;
    from_handler.remove_file(filesystem, "Weapons")?;

    // special handling
    to_handler.write_data(
        &data_cache.scripts().data,
        filesystem,
        &config.project.scripts_path,
    )?;
    from_handler.remove_file(filesystem, &config.project.scripts_path)?;

    to_handler.write_data(&*data_cache.system(), filesystem, "System")?;
    from_handler.remove_file(filesystem, "System")?;

    let mapinfos = data_cache.map_infos();
    to_handler.write_data(&mapinfos.data, filesystem, "MapInfos")?;
    from_handler.remove_file(filesystem, "MapInfos")?;

    let map_ids = mapinfos.data.keys().copied().collect_vec();
    let host = filesystem.host().unwrap();

    let fut = async move {
        let mut read_buf = Vec::new();
        let mut write_buf = Vec::<u8>::new();
        for (index, map_id) in map_ids.into_iter().enumerate() {
            progress.0.store(index, Ordering::Relaxed);
            progress.1.store(map_id, Ordering::Relaxed);

            read_buf.clear();
            write_buf.clear();
            let map_filename = format!("Map{map_id:0>3}");

            let mut map_file =
                host.open_file(from_handler.path_for(&map_filename), OpenFlags::Read)?;
            map_file.read_to_end(&mut read_buf).await?;

            let map: luminol_data::rpg::Map = from_handler.read_data_from(&read_buf)?;

            to_handler.write_data_to(&map, &mut write_buf)?;

            let mut map_file = host.open_file(
                to_handler.path_for(&map_filename),
                OpenFlags::Write | OpenFlags::Truncate | OpenFlags::Create,
            )?;
            map_file.write_all(&write_buf).await?;

            from_handler.remove_file(&host, &map_filename)?;
        }
        Ok(())
    };

    Ok(fut)
}

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

        let mut modified = false;

        egui::Window::new("Project Config")
            .open(open)
            .show(ctx, |ui| {
                ui.label("Editor Settings");
                ui.group(|ui| {
                    ui.label("Project name");
                    modified |= ui
                        .text_edit_singleline(&mut config.project.project_name)
                        .changed();
                    ui.label("Scripts path (editor)")
                        .on_hover_text("Applies to Luminol (not your game!)");
                    let scripts_changed = ui
                        .text_edit_singleline(&mut config.project.scripts_path)
                        .changed();
                    modified |= scripts_changed;
                    if scripts_changed {
                        update_state.data.scripts().modified = true; // TODO this should remove the old file
                    }

                    ui.label("Playtest Executable");
                    modified |= ui
                        .text_edit_singleline(&mut config.project.playtest_exe)
                        .changed();

                    ui.separator();

                    egui::ComboBox::from_label("Data Format")
                        .selected_text(self.selected_data_format.to_string())
                        .show_ui(ui, |ui| {
                            for format in luminol_config::DataFormat::iter() {
                                modified |= ui
                                    .selectable_value(
                                        &mut self.selected_data_format,
                                        format,
                                        format.to_string(),
                                    )
                                    .changed();
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
                            let progress = Progress::default();
                            let future = convert_project(
                                config,
                                self.selected_data_format,
                                update_state.data,
                                update_state.filesystem,
                                progress.clone(),
                            )
                            .unwrap(); //TODO handle
                            let promise = poll_promise::Promise::spawn_async(future);
                            self.convert = Some(Convert { promise, progress });
                        }
                    }

                    ui.separator();

                    egui::ComboBox::from_label("RGSS Version")
                        .selected_text(config.project.rgss_ver.to_string())
                        .show_ui(ui, |ui| {
                            for ver in luminol_config::RGSSVer::iter() {
                                modified |= ui
                                    .selectable_value(
                                        &mut config.project.rgss_ver,
                                        ver,
                                        ver.to_string(),
                                    )
                                    .changed();
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
                    modified |= ui.text_edit_singleline(&mut game_title).changed();
                    general_section.insert("Title", game_title);

                    ui.separator();

                    for rtp in ["RTP1", "RTP2", "RTP3"] {
                        let mut rtp_name = general_section.remove(rtp).unwrap_or_default();
                        ui.label(rtp);
                        modified |= ui
                            .text_edit_singleline(&mut rtp_name)
                            .on_hover_text(
                                "You may have to reload the project for changes to take effect",
                            )
                            .changed();
                        general_section.insert(rtp, rtp_name);
                    }

                    ui.separator();

                    let mut scripts_path = general_section.remove("Scripts").unwrap_or_default();

                    ui.label("Scripts path (runtime)");
                    modified |= ui
                        .text_edit_singleline(&mut scripts_path)
                        .on_hover_text("Applies only to your game (not Luminol!)")
                        .changed();
                    general_section.insert("Scripts", scripts_path);
                });
            });

        if let Some(convert) = self.convert.take() {
            let modal = egui_modal::Modal::new(ctx, "converting_project_modal");
            modal.show(|ui| {
                modal.title(ui, "Converting Project...");

                let map_infos = update_state.data.map_infos();
                let map_count = map_infos.data.len();

                let progress = convert.progress.0.load(Ordering::Relaxed);
                let map_id = convert.progress.1.load(Ordering::Relaxed);

                let progress_percent = (progress as f32 + 1.0) / map_count as f32;
                ui.label(format!(
                    "Converting Map{map_id:0>3}.{}",
                    self.selected_data_format.extension()
                ));
                egui::ProgressBar::new(progress_percent)
                    .animate(true)
                    .ui(ui);
            });
            match convert.promise.try_take() {
                Ok(Ok(())) => {}
                Ok(Err(err)) => {
                    luminol_core::error!(update_state.toasts, err)
                }
                Err(promise) => {
                    modal.open();
                    self.convert = Some(Convert {
                        promise,
                        progress: convert.progress,
                    });
                }
            }
        }

        if modified {
            update_state.modified.set(true);
        }
    }

    fn requires_filesystem(&self) -> bool {
        true
    }
}
