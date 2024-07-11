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

use egui::load::BytesLoader;
use strum::IntoEnumIterator;

#[derive(Default)]
pub struct Window {
    egui_settings_open: bool,
}

impl luminol_core::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_appearance_window")
    }

    fn show(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        update_state: &mut luminol_core::UpdateState<'_>,
    ) {
        egui::Window::new("Luminol Appearance")
            .open(open)
            .show(ctx, |ui| {
                // Or these together so if one OR the other is true the window shows.
                self.egui_settings_open =
                    ui.button("Egui Settings").clicked() || self.egui_settings_open;

                ui.menu_button("Code Theme", |ui| {
                    for t in luminol_config::SyntectTheme::iter() {
                        ui.radio_value(
                            &mut update_state.global_config.theme.syntect_theme,
                            t,
                            t.to_string(),
                        );
                    }

                    ui.label("Code sample");
                    ui.label(luminol_components::syntax_highlighting::highlight(
                        ui.ctx(),
                        update_state.global_config.theme,
                        r#"
                        class Foo < Array 
                        end
                        def bar(baz) 
                        end
                        print 1, 2.0
                        puts [0x3, :4, '5']
                        "#,
                        "rb",
                    ));
                });

                if ui
                    .button("Clear Loaded Textures")
                    .on_hover_text(
                        "You may need to reopen maps/windows for any changes to take effect.",
                    )
                    .clicked()
                {
                    update_state.graphics.texture_loader.clear();
                    update_state.graphics.atlas_loader.clear();
                    update_state.bytes_loader.forget_all();
                }
            });
    }
}
