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
use super::LOADER;
use crate::Window;

#[derive(Debug)]
pub struct PluginManagerWindow;
impl Window for PluginManagerWindow {
    fn id(&self) -> egui::Id {
        egui::Id::new("pluginmgr")
    }
    fn name(&self) -> String {
        String::from("Plugin Manager")
    }
    fn requires_filesystem(&self) -> bool {
        false
    }
    fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .id(self.id())
            .open(open)
            .show(ctx, |ui| {
                if ui.button("Reload Plugins").clicked() {
                    LOADER.load("net.somedevfox.test").unwrap();
                }
            });
    }
}
