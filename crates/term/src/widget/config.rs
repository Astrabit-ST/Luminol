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

use egui::Widget;

use super::Theme;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    pub font: egui::FontId,
    pub initial_size: (u16, u16),
    pub bell_enabled: bool,

    pub cursor_blinking: CursorBlinking,
    pub theme: Theme,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::Display)]
pub enum CursorBlinking {
    #[strum(to_string = "Terminal defined")]
    Terminal,
    Always,
    Never,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            font: Self::default_font(),
            initial_size: (80, 24),
            bell_enabled: true,
            cursor_blinking: CursorBlinking::Always,
            theme: Theme::default(),
        }
    }
}

pub struct ConfigUi {
    font_family: FontFamily,
    font_size: f32,
}

#[derive(PartialEq, Eq)]
#[derive(strum::EnumIter, strum::Display)]
enum FontFamily {
    Proportional,
    Monospace,
    Custom(String),
}

impl ConfigUi {
    pub fn new(config: &Config) -> Self {
        Self {
            font_family: match &config.font.family {
                egui::FontFamily::Monospace => FontFamily::Monospace,
                egui::FontFamily::Proportional => FontFamily::Proportional,
                egui::FontFamily::Name(n) => FontFamily::Custom(n.to_string()),
            },
            font_size: config.font.size,
        }
    }

    pub fn ui(&mut self, config: &mut Config, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Initial terminal size:");

            egui::DragValue::new(&mut config.initial_size.0)
                .clamp_range(1..=999)
                .ui(ui);
            ui.label("column(s)");

            egui::DragValue::new(&mut config.initial_size.1)
                .clamp_range(1..=999)
                .ui(ui);
            ui.label("rows(s)");
        });

        ui.horizontal(|ui| {
            ui.label("Font family");
            luminol_components::EnumMenuButton::new(
                &mut self.font_family,
                "luminol_term_config_ui_font_family",
            )
            .ui(ui);

            let is_custom = matches!(self.font_family, FontFamily::Custom(_));
            ui.add_enabled_ui(is_custom, |ui| {
                let mut dummy_text = String::new(); // this doesn't allocate so this is fine, for display purposes

                let text = match &mut self.font_family {
                    FontFamily::Custom(t) => t,
                    _ => &mut dummy_text,
                };
                ui.text_edit_singleline(text);
            });

            ui.label("Font size");
            egui::DragValue::new(&mut self.font_size)
                .clamp_range(1..=80)
                .update_while_editing(false)
                .ui(ui);

            if ui.button("Apply").clicked() {
                config.font.family = match &self.font_family {
                    FontFamily::Monospace => egui::FontFamily::Monospace,
                    FontFamily::Proportional => egui::FontFamily::Proportional,
                    FontFamily::Custom(name) => egui::FontFamily::Name(name.as_str().into()), // FIXME doesn't properly handle missing fonts
                };
                config.font.size = self.font_size;
            }
        });

        luminol_components::Field::new(
            "Cursor blinking",
            luminol_components::EnumComboBox::new(
                "luminol_term_config_ui_cursor_blinking",
                &mut config.cursor_blinking,
            )
            .max_width(12.),
        )
        .ui(ui);

        ui.add_space(6.);
        ui.label("Ui colors");
        ui.separator();

        ui.columns(2, |cols| {
            let [left, right] = cols else {
                unreachable!();
            };

            left.label("Cursor");
            let mut arr = color_to_rgb(config.theme.cursor_color);
            left.color_edit_button_srgb(&mut arr);
            config.theme.cursor_color = color_from_rgb(arr);

            right.label("Background");
            let mut arr = color_to_rgb(config.theme.background_color);
            right.color_edit_button_srgb(&mut arr);
            config.theme.background_color = color_from_rgb(arr);
        });

        ui.add_space(6.);
        ui.label("Pallette");

        for colors in config.theme.color_pallette.chunks_mut(8) {
            ui.horizontal(|ui| {
                for color in colors {
                    let mut arr = color_to_rgb(*color);
                    ui.color_edit_button_srgb(&mut arr);
                    *color = color_from_rgb(arr);
                }
            });
        }
    }
}

impl Config {
    pub fn default_font() -> egui::FontId {
        egui::FontId {
            size: 14.,
            family: egui::FontFamily::Name("Iosevka Term".into()),
        }
    }
}

fn color_to_rgb(color: egui::Color32) -> [u8; 3] {
    let [r, g, b, _] = color.to_array();
    [r, g, b]
}

fn color_from_rgb([r, g, b]: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}
