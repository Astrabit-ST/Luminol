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
use crate::prelude::*;

#[derive(Default)]
pub struct Window {
    egui_settings_open: bool,
}

impl super::Window for Window {
    fn id(&self) -> egui::Id {
        egui::Id::new("luminol_appearance_window")
    }

    fn name(&self) -> String {
        "Luminol Appearance".to_string()
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name()).open(open).show(ctx, |ui| {
            // Or these together so if one OR the other is true the window shows.
            self.egui_settings_open =
                ui.button("Egui Settings").clicked() || self.egui_settings_open;

            ui.menu_button("Catppuccin theme", |ui| {
                if ui.button("Frappe").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::FRAPPE);
                }
                if ui.button("Latte").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::LATTE);
                }
                if ui.button("Macchiato").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::MACCHIATO);
                }
                if ui.button("Mocha").clicked() {
                    catppuccin_egui::set_theme(ui.ctx(), catppuccin_egui::MOCHA);
                }
            });

            let theme = &mut global_config!().theme;
            ui.menu_button("Code Theme", |ui| {
                theme.ui(ui);

                ui.label("Code sample");
                ui.label(syntax_highlighting::highlight(
                    ui.ctx(),
                    *theme,
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
                state!().image_cache.clear();
                state!().atlas_cache.clear();
            }
        });
    }
}
