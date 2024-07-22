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
#[cfg(not(target_arch = "wasm32"))]
use egui::Widget;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct Window {
    #[cfg(not(target_arch = "wasm32"))]
    edit_rtp_path_name: String,
    #[cfg(not(target_arch = "wasm32"))]
    edit_rtp_path_path: String,

    tab: Tab,
}

#[derive(Clone, Copy)]
#[derive(Default, PartialEq, Eq)]
#[derive(strum::EnumIter, strum::Display)]
enum Tab {
    #[cfg_attr(not(target_arch = "wasm32"), default)]
    #[strum(to_string = "Editor Settings")]
    #[cfg(not(target_arch = "wasm32"))]
    // only setting right now is editing rtp paths, which is not supported on wasm
    EditorSettings,
    #[strum(to_string = "Egui Visuals")]
    #[cfg_attr(target_arch = "wasm32", default)]
    EguiVisuals,
    #[strum(to_string = "Preset Visuals")]
    PresetVisuals,
    #[strum(to_string = "Code Theme")]
    CodeTheme,
    #[cfg(not(target_arch = "wasm32"))]
    Terminal,
}

const CODE_SAMPLE: &str = luminol_macros::include_asset_str!("assets/ruby/code_sample.rb");

#[derive(Clone)]
struct PresetTheme {
    name: &'static str,
    visuals: egui::Visuals,
    description: &'static str,
}

macro_rules! preset_theme {
    ($name:literal, $path:literal, $description:literal) => {
        PresetTheme {
            name: $name,
            visuals: ron::from_str(luminol_macros::include_asset_str!($path)).expect(concat!(
                "failed to load ",
                $name,
                " preset theme"
            )),
            description: $description,
        }
    };
}

static PRESET_VISUALS: once_cell::sync::Lazy<[PresetTheme; 7]> = once_cell::sync::Lazy::new(|| {
    //
    let catppuccin_frappe = preset_theme!(
        "Catppuccin Frappe",
        "themes/catppuccin_frappe.ron",
        "A less vibrant alternative theme using subdued colors for a muted aesthetic"
    );
    let catppuccin_latte = preset_theme!(
            "Catppuccin Latte",
            "themes/catppuccin_latte.ron",
            "Catppuccin's lightest theme harmoniously inverting the essence of Catppuccin's dark themes"
        );
    let catppuccin_macchiato = preset_theme!(
        "Catppuccin Macchiato",
        "themes/catppuccin_macchiato.ron",
        "A theme with medium contrast and gentle colors creating a soothing atmosphere"
    );
    let catppuccin_mocha = preset_theme!(
        "Catppuccin Mocha",
        "themes/catppuccin_mocha.ron",
        "Catppuccin's darkest variant offering a cozy feeling with color-rich accents"
    );
    let luminol = preset_theme!(
        "Luminol",
        "themes/luminol.ron",
        "A high-contrast dark theme based on the Luminol website"
    );

    let egui_dark = PresetTheme {
        name: "Egui Dark",
        visuals: egui::Visuals::dark(),
        description: "The default theme from Luminol's GUI framework",
    };
    let egui_light = PresetTheme {
        name: "Egui Light",
        visuals: egui::Visuals::light(),
        description: "The default light theme from Luminol's GUI framework.",
    };

    [
        catppuccin_latte,
        catppuccin_frappe,
        catppuccin_macchiato,
        catppuccin_mocha,
        luminol,
        egui_dark,
        egui_light,
    ]
});

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_preferences_window")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Preferences").open(open).show(ctx, |ui| {
            ui.horizontal(|ui| {
                for mode in Tab::iter() {
                    ui.selectable_value(&mut self.tab, mode, mode.to_string());
                }
            });
            ui.separator();

            match self.tab {
                Tab::EguiVisuals => {
                    // TODO maybe make a custom visuals editor?
                    let mut visuals = ctx.style().visuals.clone();
                    visuals.ui(ui);
                    ctx.set_visuals(visuals);
                }
                Tab::PresetVisuals => ui.columns(2, |cols| {
                    let [left, right] = cols else { unreachable!() };

                    let mut hover_visual = None;
                    egui::ScrollArea::vertical().show(left, |ui| {
                        ui.visuals_mut().button_frame = false;
                        for PresetTheme {
                            name,
                            visuals,
                            description,
                        } in PRESET_VISUALS.iter()
                        {
                            let response = ui.button(*name).on_hover_text(*description);
                            if response.hovered() {
                                hover_visual = Some(visuals.clone());
                            }
                            if response.clicked() {
                                ctx.set_visuals(visuals.clone())
                            }
                        }
                    });

                    if let Some(hover_visual) = hover_visual {
                        *right.visuals_mut() = hover_visual;
                    };

                    let frame = egui::Frame {
                        shadow: egui::epaint::Shadow::NONE,
                        ..egui::Frame::window(right.style())
                    };
                    frame.show(right, |ui| {
                        egui::Grid::new("luminol-preset-theme-gallery")
                            .num_columns(2)
                            .spacing([40.0, 4.0])
                            .striped(true)
                            .show(ui, gallery_grid_contents)
                    });
                }),
                Tab::CodeTheme => {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            for t in luminol_config::SyntectTheme::iter() {
                                ui.radio_value(
                                    &mut update_state.global_config.theme.syntect_theme,
                                    t,
                                    t.to_string(),
                                );
                            }
                        });

                        ui.vertical(|ui| {
                            ui.label("Code sample");
                            ui.label(luminol_components::syntax_highlighting::highlight(
                                ui.ctx(),
                                update_state.global_config.theme,
                                CODE_SAMPLE,
                                "rb",
                            ));
                        });
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                Tab::EditorSettings => {
                    ui.label("RTP Paths");
                    ui.separator();

                    ui.columns(2, |columns| {
                        let mut new_rtp_paths: indexmap::IndexMap<_, _> = update_state
                            .global_config
                            .rtp_paths
                            .drain(..)
                            .filter_map(|(mut rtp_name, mut rtp_path)| {
                                let res = columns[0].horizontal(|ui| {
                                    let res = ui.button(
                                        egui::RichText::new("-")
                                            .monospace()
                                            .color(egui::Color32::RED),
                                    );
                                    ui.text_edit_singleline(&mut rtp_name);
                                    res.clicked()
                                });
                                columns[1].text_edit_singleline(&mut rtp_path);
                                (!res.inner).then_some((rtp_name, rtp_path))
                            })
                            .collect();

                        columns[0].horizontal(|ui| {
                            if ui
                                .button(
                                    egui::RichText::new("+")
                                        .monospace()
                                        .color(egui::Color32::GREEN),
                                )
                                .clicked()
                            {
                                new_rtp_paths.insert(
                                    std::mem::take(&mut self.edit_rtp_path_name),
                                    std::mem::take(&mut self.edit_rtp_path_path),
                                );
                            }
                            ui.text_edit_singleline(&mut self.edit_rtp_path_name);
                        });
                        columns[1].text_edit_singleline(&mut self.edit_rtp_path_path);

                        update_state.global_config.rtp_paths = new_rtp_paths;
                    });
                }
                #[cfg(not(target_arch = "wasm32"))]
                Tab::Terminal => {
                    let config = &mut update_state.global_config.terminal;
                    ui.horizontal(|ui| {
                        ui.label("Initial terminal size:");
                        egui::DragValue::new(&mut config.initial_size.0)
                            .range(1..=999)
                            .ui(ui);
                        ui.label("column(s)");
                        egui::DragValue::new(&mut config.initial_size.1)
                            .range(1..=999)
                            .ui(ui);
                        ui.label("rows(s)");
                    });
                    // ui.horizontal(|ui| {
                    // ui.label("Font family");
                    // luminol_components::EnumMenuButton::new(
                    //     &mut self.font_family,
                    //     "luminol_term_config_ui_font_family",
                    // )
                    // .ui(ui);
                    // let is_custom = matches!(self.font_family, FontFamily::Custom(_));
                    // ui.add_enabled_ui(is_custom, |ui| {
                    //     let mut dummy_text = String::new(); // this doesn't allocate so this is fine, for display purposes
                    //     let text = match &mut self.font_family {
                    //         FontFamily::Custom(t) => t,
                    //         _ => &mut dummy_text,
                    //     };
                    //     ui.text_edit_singleline(text);
                    // });
                    // ui.label("Font size");
                    // egui::DragValue::new(&mut self.font_size)
                    //     .clamp_range(1..=80)
                    //     .update_while_editing(false)
                    //     .ui(ui);
                    // if ui.button("Apply").clicked() {
                    //     config.font.family = match &self.font_family {
                    //         FontFamily::Monospace => egui::FontFamily::Monospace,
                    //         FontFamily::Proportional => egui::FontFamily::Proportional,
                    //         FontFamily::Custom(name) => {
                    //             egui::FontFamily::Name(name.as_str().into())
                    //         } // FIXME doesn't properly handle missing fonts
                    //     };
                    //     config.font.size = self.font_size;
                    // }
                    // });
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

                    if ui
                        .button(egui::RichText::new("Reset").color(egui::Color32::RED))
                        .clicked()
                    {
                        *config = luminol_config::terminal::Config::default()
                    }
                }
            }
        });
    }
}

// adapted from https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/widget_gallery.rs
#[allow(unused_must_use)]
fn gallery_grid_contents(ui: &mut egui::Ui) {
    ui.label("Label");
    ui.label("Welcome to the preview gallery!");
    ui.end_row();

    ui.label("Hyperlink");
    use egui::special_emojis::GITHUB;
    ui.hyperlink_to(
        format!("{GITHUB} Luminol on GitHub"),
        "https://github.com/Astrabit-ST/Luminol",
    );
    ui.end_row();

    ui.label("TextEdit");
    ui.add(egui::TextEdit::singleline(&mut "").hint_text("Write something here"));
    ui.end_row();

    ui.label("Button");
    ui.button("Click me!");
    ui.end_row();

    ui.label("Link");
    ui.link("Click me!");
    ui.end_row();

    ui.label("Checkbox");
    ui.checkbox(&mut true, "Checkbox");
    ui.end_row();

    ui.label("RadioButton");
    ui.horizontal(|ui| {
        ui.radio_value(&mut 0, 0, "First");
        ui.radio_value(&mut 0, 1, "Second");
        ui.radio_value(&mut 0, 2, "Third");
    });
    ui.end_row();

    ui.label("SelectableLabel");
    ui.horizontal(|ui| {
        ui.selectable_value(&mut 0, 0, "First");
        ui.selectable_value(&mut 0, 1, "Second");
        ui.selectable_value(&mut 0, 2, "Third");
    });
    ui.end_row();

    ui.label("ComboBox");
    egui::ComboBox::from_label("Take your pick")
        .selected_text("First")
        .show_ui(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            ui.set_min_width(60.0);
            ui.selectable_value(&mut 0, 0, "First");
            ui.selectable_value(&mut 0, 1, "Second");
            ui.selectable_value(&mut 0, 2, "Third");
        });
    ui.end_row();

    ui.label("Slider");
    ui.add(egui::Slider::new(&mut 128.0, 0.0..=360.0).suffix("°"));
    ui.end_row();

    ui.label("DragValue");
    ui.add(egui::DragValue::new(&mut 128.0).speed(1.0));
    ui.end_row();

    ui.label("ProgressBar");
    let progress = 128.0 / 360.0;
    ui.add(
        egui::ProgressBar::new(progress)
            .show_percentage()
            .animate(true),
    );
    ui.end_row();

    ui.label("Color picker");
    ui.color_edit_button_srgba(&mut egui::Color32::from_rgb(12, 208, 247));
    ui.end_row();

    ui.label("Image");
    let egui_icon = egui::ImageSource::Bytes {
        uri: "bytes://assets/icon.png".into(),
        bytes: luminol_macros::include_asset!("assets/icons/icon.png").into(),
    };
    ui.add(egui::Image::new(egui_icon.clone()));
    ui.end_row();

    ui.label("Button with image");
    ui.add(egui::Button::image_and_text(egui_icon, "Click me!"));

    ui.end_row();

    ui.label("Separator");
    ui.separator();
    ui.end_row();

    ui.label("CollapsingHeader");
    ui.collapsing("Click to see what is hidden!", |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("It's a spinner");
            ui.add_space(4.0);
            ui.add(egui::Spinner::new());
        });
    });
    ui.end_row();
}

#[allow(dead_code)]
fn color_to_rgb(color: egui::Color32) -> [u8; 3] {
    let [r, g, b, _] = color.to_array();
    [r, g, b]
}
#[allow(dead_code)]
fn color_from_rgb([r, g, b]: [u8; 3]) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}
