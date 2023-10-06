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

use command_lib::{Index, Parameter, ParameterKind};
use eframe::egui;

use strum::IntoEnumIterator;

use crate::fl;

pub fn parameter_ui(
    ui: &mut egui::Ui,
    parameter: &mut Parameter,
    del_idx: (usize, &mut Option<usize>),
) {
    ui.horizontal(|ui| {
        ui.menu_button(format!("{} ⏷", <&str>::from(&*parameter)), |ui| {
            for iter_kind in Parameter::iter() {
                if let Parameter::Group { ref mut guid, .. }
                | Parameter::Selection { ref mut guid, .. } = parameter
                {
                    *guid = rand::random();
                }
                let text: &str = (&iter_kind).into();
                ui.selectable_value(parameter, iter_kind, text);
            }
        });

        if let Parameter::Single { ref mut index, .. }
        | Parameter::Selection { ref mut index, .. } = parameter
        {
            ui.label(format!("{}: ", fl!("position")))
                .on_hover_text_at_pointer(fl!("window_commandgen_position_onhover_label"));
            match index {
                Index::Overridden(ref mut idx) => {
                    ui.add(egui::DragValue::new(idx));
                }
                Index::Assumed(ref mut assumed_idx) => {
                    if ui.add(egui::DragValue::new(assumed_idx)).changed() {
                        *index = Index::Overridden(*assumed_idx);
                    }
                }
            }
        }

        if ui
            .button(
                egui::RichText::new("-")
                    .monospace()
                    .color(egui::Color32::RED),
            )
            .clicked()
        {
            *del_idx.1 = Some(del_idx.0);
        }
    });

    match parameter {
        Parameter::Group {
            ref mut parameters,
            guid,
        } => {
            ui.push_id(guid, |ui| {
                egui::CollapsingHeader::new(fl!("window_commandgen_grouped_params_label"))
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut del_idx = None;
                        for (ele, parameter) in parameters.iter_mut().enumerate() {
                            parameter_ui(ui, parameter, (ele, &mut del_idx))
                        }

                        if let Some(idx) = del_idx {
                            parameters.remove(idx);
                        }

                        if ui
                            .button(
                                egui::RichText::new("+")
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                            )
                            .clicked()
                        {
                            parameters.push(Parameter::default());
                        }
                    })
                    .header_response
                    .on_hover_text(fl!("window_commandgen_grouped_params_onhover_label"));
            });
        }
        Parameter::Selection {
            ref mut parameters,
            guid,
            ..
        } => {
            ui.push_id(guid, |ui| {
                egui::CollapsingHeader::new(fl!("window_commandgen_subparams_label"))
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut del_idx = None;
                        for (ele, (id, parameter)) in parameters.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.add(egui::DragValue::new(id));

                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                        parameter_ui(ui, parameter, (ele, &mut del_idx))
                                    });
                                });
                            });
                        }

                        if let Some(idx) = del_idx {
                            parameters.remove(idx);
                        }

                        if ui
                            .button(
                                egui::RichText::new("+")
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                            )
                            .clicked()
                        {
                            parameters.push((0, Parameter::default()));
                        }
                    })
                    .header_response
                    .on_hover_text(fl!("window_commandgen_subparams_onhover_label"));
            });
        }
        Parameter::Single {
            description,
            name,
            kind,
            ..
        } => {
            ui.horizontal(|ui| {
                ui.label(fl!("name"));
                ui.text_edit_singleline(name);
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}:", fl!("description")));
                ui.text_edit_singleline(description)
                    .on_hover_text(fl!("window_commandgen_description_onhover_label"));
            });

            ui.horizontal(|ui| {
                ui.label(format!("{}: ", fl!("type")));
                ui.menu_button(format!("{} ⏷", <&str>::from(&*kind)), |ui| {
                    for iter_kind in ParameterKind::iter() {
                        let text: &str = (&iter_kind).into();
                        ui.selectable_value(kind, iter_kind, text);
                    }
                });
            });

            if let ParameterKind::Enum { ref mut variants } = kind {
                egui::CollapsingHeader::new(fl!("variants"))
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut del_idx = None;
                        for (ele, (name, id)) in variants.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                ui.text_edit_singleline(name);
                                ui.add(egui::DragValue::new(id));

                                if ui
                                    .button(
                                        egui::RichText::new("-")
                                            .monospace()
                                            .color(egui::Color32::RED),
                                    )
                                    .clicked()
                                {
                                    del_idx = Some(ele);
                                }
                            });
                        }

                        if let Some(idx) = del_idx {
                            variants.remove(idx);
                        }

                        if ui
                            .button(
                                egui::RichText::new("+")
                                    .monospace()
                                    .color(egui::Color32::GREEN),
                            )
                            .clicked()
                        {
                            variants.push(("".to_string(), 0));
                        }
                    })
                    .header_response
                    .on_disabled_hover_text(fl!("window_commandgen_variants_onhover_label"));
            }
        }
        Parameter::Dummy => {}
        Parameter::Label(label) => {
            ui.text_edit_singleline(label);
        }
    }
    ui.separator();
}
