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

use crate::{
    components::{toolbar::Toolbar, top_bar::TopBar},
    UpdateInfo,
};

pub struct Luminol {
    top_bar: TopBar,
    toolbar: Toolbar,
    info: &'static UpdateInfo,
    #[cfg(feature = "discord-rpc")]
    discord: crate::discord::DiscordClient,
}

impl Luminol {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            top_bar: TopBar::default(),
            toolbar: Toolbar::default(),
            info: Box::leak(Box::new(UpdateInfo::default())), // This is bad but I don't care
            #[cfg(feature = "discord-rpc")]
            discord: crate::discord::DiscordClient::default(),
        }
    }
}

impl eframe::App for Luminol {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value::<Option<()>>(storage, eframe::APP_KEY, &None);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_toolbar").show(ctx, |ui| {
            // We want the top menubar to be horizontal. Without this it would fill up vertically.
            ui.horizontal_wrapped(|ui| {
                // Turn off button frame.
                ui.visuals_mut().button_frame = false;
                // Show the bar
                self.top_bar.ui(self.info, ui);
            });
        });

        egui::SidePanel::left("toolbar")
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.toolbar.ui(self.info, ui);
                });
            });

        // Central panel with tabs.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.info.tabs.ui(ui, self.info);
        });

        // Update all windows.
        self.info.windows.update(ctx, self.info);

        // Show toasts.
        self.info.toasts.show(ctx);

        // Update discord
        #[cfg(feature = "discord-rpc")]
        self.discord.update(
            self.info.tabs.discord_display(),
            self.info.filesystem
                .project_path()
                .map(|p| p.display().to_string()),
        );
    }
}
