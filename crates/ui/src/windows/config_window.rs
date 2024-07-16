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
use luminol_core::data_formats::Handler as FormatHandler;
use luminol_data::rpg;
use luminol_filesystem::{FileSystem, OpenFlags};
use std::sync::atomic::{AtomicUsize, Ordering};
use strum::IntoEnumIterator;

pub struct Window {
    selected_data_format: luminol_config::DataFormat,
    convert: Option<Convert>,
}

struct Convert {
    promise: poll_promise::Promise<color_eyre::Result<()>>,
    converting: Converting,
}

impl Window {
    pub fn new(config: &luminol_config::project::Config) -> Self {
        Self {
            selected_data_format: config.project.data_format,
            convert: None,
        }
    }
}

type Converting = std::sync::Arc<(AtomicUsize, AtomicUsize)>;

const CONVERTING_ACTORS: usize = 0;
const CONVERTING_ANIMATIONS: usize = 1;
const CONVERTING_ARMORS: usize = 2;
const CONVERTING_CLASSES: usize = 3;
const CONVERTING_COMMON_EVENTS: usize = 4;
const CONVERTING_ENEMIES: usize = 5;
const CONVERTING_ITEMS: usize = 6;
const CONVERTING_SKILLS: usize = 7;
const CONVERTING_STATES: usize = 8;
const CONVERTING_TILESETS: usize = 9;
const CONVERTING_TROOPS: usize = 10;
const CONVERTING_WEAPONS: usize = 11;
const CONVERTING_SCRIPTS: usize = 12;
const CONVERTING_SYSTEM: usize = 13;
const CONVERTING_MAPINFOS: usize = 14;

fn converting_to_string(
    converting: usize,
    map_id: usize,
    selected_data_format: luminol_config::DataFormat,
) -> String {
    let text = match converting {
        CONVERTING_ACTORS => "Actors",
        CONVERTING_ANIMATIONS => "Animations",
        CONVERTING_ARMORS => "Armors",
        CONVERTING_CLASSES => "Classes",
        CONVERTING_COMMON_EVENTS => "CommonEvents",
        CONVERTING_ENEMIES => "Enemies",
        CONVERTING_ITEMS => "Items",
        CONVERTING_SKILLS => "Skills",
        CONVERTING_STATES => "States",
        CONVERTING_TILESETS => "Tilesets",
        CONVERTING_TROOPS => "Troops",
        CONVERTING_WEAPONS => "Weapons",

        CONVERTING_SCRIPTS => "Scripts",
        CONVERTING_SYSTEM => "System",
        CONVERTING_MAPINFOS => "MapInfos",
        _ => {
            return format!("Map{map_id:0>3}.{}", selected_data_format.extension());
        }
    };
    format!("{}.{}", text, selected_data_format.extension())
}

const FORMAT_WARNING: &str = "Luminol will need to convert your project.\nThis is not 100% safe yet, make backups!\nPress OK to continue.";

// Mostly async, opening files is not however.
// We should probably provide async fns for that
async fn convert_nil_padded<T>(
    from: FormatHandler,
    to: FormatHandler,
    read_buf: &mut Vec<u8>,
    write_buf: &mut Vec<u8>,
    filename: &str,
    host: &luminol_filesystem::host::FileSystem,
) -> color_eyre::Result<Vec<T>>
where
    T: ::serde::de::DeserializeOwned + serde::Serialize,
    T: for<'de> alox_48::Deserialize<'de> + alox_48::Serialize,
{
    read_buf.clear();
    write_buf.clear();

    let mut file = host.open_file(from.path_for(filename), OpenFlags::Read)?;
    file.read_to_end(read_buf).await?;

    let data = from.read_nil_padded_from::<T>(read_buf)?;

    to.write_nil_padded_to(&data, write_buf)?;

    let mut file = host.open_file(
        to.path_for(filename),
        OpenFlags::Write | OpenFlags::Truncate | OpenFlags::Create,
    )?;
    file.write_all(write_buf).await?;
    file.flush().await?;

    from.remove_file(host, filename)?;

    Ok(data)
}

async fn convert_regular<T>(
    from: FormatHandler,
    to: FormatHandler,
    read_buf: &mut Vec<u8>,
    write_buf: &mut Vec<u8>,
    filename: &str,
    host: &luminol_filesystem::host::FileSystem,
) -> color_eyre::Result<T>
where
    T: ::serde::de::DeserializeOwned + serde::Serialize,
    T: for<'de> alox_48::Deserialize<'de> + alox_48::Serialize,
{
    read_buf.clear();
    write_buf.clear();

    let mut file = host.open_file(from.path_for(filename), OpenFlags::Read)?;
    file.read_to_end(read_buf).await?;

    let data = from.read_data_from::<T>(read_buf)?;

    to.write_data_to(&data, write_buf)?;

    let mut file = host.open_file(
        to.path_for(filename),
        OpenFlags::Write | OpenFlags::Truncate | OpenFlags::Create,
    )?;
    file.write_all(write_buf).await?;
    file.flush().await?;

    from.remove_file(host, filename)?;

    Ok(data)
}

fn convert_project(
    config: &mut luminol_config::project::Config,
    selected_data_format: luminol_config::DataFormat,
    filesystem: &luminol_filesystem::project::FileSystem,
    converting: Converting,
) -> impl std::future::Future<Output = color_eyre::Result<()>> {
    let from = FormatHandler::new(config.project.data_format);
    let to = FormatHandler::new(selected_data_format);

    // TODO handle errors
    let pretty_config = ron::ser::PrettyConfig::new()
        .struct_names(true)
        .enumerate_arrays(true);
    config.project.data_format = selected_data_format;
    let project_config = ron::ser::to_string_pretty(&config.project, pretty_config).unwrap();
    filesystem.write(".luminol/config", project_config).unwrap();

    let host = filesystem.host().unwrap(); // This bypasses the path cache (which is BAD!) so we will need to regen it later
    let scripts_filename = config.project.scripts_path.clone();

    async move {
        let mut read_buf = Vec::new();
        let mut write_buf = Vec::<u8>::new();

        let host = &host;
        let read_buf = &mut read_buf;
        let write_buf = &mut write_buf;

        let (converting_progress, converting_map_id) = &*converting;

        // FIXME: have some kind of trait system to determine filenames rather than hardcoding them like this
        converting_progress.store(CONVERTING_ACTORS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Actor>(from, to, read_buf, write_buf, "Actors", host).await?;

        converting_progress.store(CONVERTING_ANIMATIONS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Animation>(from, to, read_buf, write_buf, "Animations", host)
            .await?;

        converting_progress.store(CONVERTING_ARMORS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Armor>(from, to, read_buf, write_buf, "Armors", host).await?;

        converting_progress.store(CONVERTING_CLASSES, Ordering::Relaxed);
        convert_nil_padded::<rpg::Class>(from, to, read_buf, write_buf, "Classes", host).await?;

        converting_progress.store(CONVERTING_COMMON_EVENTS, Ordering::Relaxed);
        convert_nil_padded::<rpg::CommonEvent>(from, to, read_buf, write_buf, "CommonEvents", host)
            .await?;

        converting_progress.store(CONVERTING_ENEMIES, Ordering::Relaxed);
        convert_nil_padded::<rpg::Enemy>(from, to, read_buf, write_buf, "Enemies", host).await?;

        converting_progress.store(CONVERTING_ITEMS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Item>(from, to, read_buf, write_buf, "Items", host).await?;

        converting_progress.store(CONVERTING_SKILLS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Skill>(from, to, read_buf, write_buf, "Skills", host).await?;

        converting_progress.store(CONVERTING_STATES, Ordering::Relaxed);
        convert_nil_padded::<rpg::State>(from, to, read_buf, write_buf, "States", host).await?;

        converting_progress.store(CONVERTING_TILESETS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Tileset>(from, to, read_buf, write_buf, "Tilesets", host).await?;

        converting_progress.store(CONVERTING_TROOPS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Troop>(from, to, read_buf, write_buf, "Troops", host).await?;

        converting_progress.store(CONVERTING_WEAPONS, Ordering::Relaxed);
        convert_nil_padded::<rpg::Weapon>(from, to, read_buf, write_buf, "Weapons", host).await?;

        converting_progress.store(CONVERTING_SCRIPTS, Ordering::Relaxed);
        convert_regular::<Vec<rpg::Script>>(from, to, read_buf, write_buf, &scripts_filename, host)
            .await?;

        converting_progress.store(CONVERTING_SYSTEM, Ordering::Relaxed);
        convert_regular::<rpg::System>(from, to, read_buf, write_buf, "System", host).await?;

        converting_progress.store(CONVERTING_MAPINFOS, Ordering::Relaxed);
        let mapinfos: std::collections::HashMap<usize, rpg::MapInfo> =
            convert_regular(from, to, read_buf, write_buf, "MapInfos", host).await?;

        for (index, map_id) in mapinfos.keys().copied().enumerate() {
            converting_progress.store(CONVERTING_MAPINFOS + index, Ordering::Relaxed);
            converting_map_id.store(map_id, Ordering::Relaxed);

            let map_filename = format!("Map{map_id:0>3}");
            convert_regular::<rpg::Map>(from, to, read_buf, write_buf, &map_filename, host).await?;
        }
        Ok(())
    }
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
                    if let luminol_config::DataFormat::Json { pretty }
                    | luminol_config::DataFormat::Ron { pretty } = &mut self.selected_data_format
                    {
                        ui.checkbox(pretty, "Pretty Print").on_hover_text("This will make the data files human-readable, but significantly larger!");
                    }

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
                            let converting = Converting::default();
                            let future = convert_project(
                                config,
                                self.selected_data_format,
                                update_state.filesystem,
                                converting.clone(),
                            );
                            #[cfg(not(target_arch = "wasm32"))]
                            let promise = poll_promise::Promise::spawn_async(future);
                            #[cfg(target_arch = "wasm32")]
                            let promise = poll_promise::Promise::spawn_local(future);
                            self.convert = Some(Convert {
                                promise,
                                converting,
                            });
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

                let current_progress = convert.converting.0.load(Ordering::Relaxed);
                let current_map_id = convert.converting.1.load(Ordering::Relaxed);

                let total = CONVERTING_MAPINFOS + map_count + 1;
                let progress = (current_progress + 2) as f32 / total as f32;

                let current_text = converting_to_string(
                    current_progress,
                    current_map_id,
                    self.selected_data_format,
                );

                ui.label(format!(
                    "Converting {current_text} {}/{total}",
                    current_progress + 2
                ));
                egui::ProgressBar::new(progress).animate(true).ui(ui);
            });
            match convert.promise.try_take() {
                Ok(Ok(())) => {
                    // we've drastically edited the data folder, so the path cache needs to be rebuilt
                    update_state.filesystem.rebuild_path_cache();
                }
                Ok(Err(err)) => {
                    luminol_core::error!(update_state.toasts, err);
                    luminol_core::warn!(
                        update_state.toasts,
                        "WARNING: Your project may be corrupted!"
                    );
                }
                Err(promise) => {
                    modal.open();
                    self.convert = Some(Convert { promise, ..convert });
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
