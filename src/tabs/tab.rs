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

use std::{cell::RefCell, hash::Hash};

use super::started::Started;
use crate::UpdateInfo;

/// The tree type;
type Tree = egui_dock::Tree<Box<dyn Tab>>;

/// Helper struct for tabs.
pub struct Tabs {
    tree: RefCell<Tree>,
    id: egui::Id,
}

impl Default for Tabs {
    fn default() -> Self {
        // Add the basic "get started" tab
        Self {
            tree: RefCell::new(Tree::new(vec![Box::new(Started::new())])),
            id: egui::Id::new("tab_area"),
        }
    }
}

impl Tabs {
    /// Create a new Tab viewer without any tabs.
    pub fn new(id: impl Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            tree: Default::default(),
        }
    }

    /// Display all tabs.
    pub fn ui(&self, ui: &mut egui::Ui, info: &'static UpdateInfo) {
        ui.group(|ui| {
            egui_dock::DockArea::new(&mut self.tree.borrow_mut())
                .id(self.id)
                .show_inside(ui, &mut TabViewer { info });
        });
    }

    /// Add a tab.
    pub fn add_tab<T>(&self, tab: T)
    where
        T: Tab + 'static,
    {
        let mut tree = self.tree.borrow_mut();
        for n in tree.iter() {
            if let egui_dock::Node::Leaf { tabs, .. } = n {
                if tabs.iter().any(|t| t.name() == tab.name()) {
                    return;
                }
            }
        }
        tree.push_to_focused_leaf(Box::new(tab));
    }

    /// Clean tabs by if they need the filesystem.
    pub fn clean_tabs(&self) {
        let mut tree = self.tree.borrow_mut();
        for node in tree.iter_mut() {
            if let egui_dock::Node::Leaf { tabs, .. } = node {
                tabs.drain_filter(|t| t.requires_filesystem());
            }
        }
    }

    /// Returns the name of the focused tab.
    pub fn focused_name(&self) -> Option<String> {
        let mut tree = self.tree.borrow_mut();
        tree.find_active().map(|(_, t)| t.name())
    }

    /// The discord rpc text to display.
    #[cfg(feature = "discord-rpc")]
    pub fn discord_display(&self) -> String {
        let mut tree = self.tree.borrow_mut();
        if let Some((_, tab)) = tree.find_active() {
            tab.discord_display()
        } else {
            "No tab open".to_string()
        }
    }
}

struct TabViewer {
    pub info: &'static UpdateInfo,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = Box<dyn Tab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui, self.info);
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        tab.force_close()
    }
}

/// A tab trait.
pub trait Tab {
    /// The name of the tab.
    fn name(&self) -> String;

    /// Show this tab.
    fn show(&mut self, ui: &mut egui::Ui, info: &'static UpdateInfo);

    /// Does this tab need the filesystem?
    fn requires_filesystem(&self) -> bool {
        false
    }

    /// Does this tab need to be closed?
    fn force_close(&mut self) -> bool {
        false
    }

    /// The discord rpc text to display for this tab.
    #[cfg(feature = "discord-rpc")]
    fn discord_display(&self) -> String {
        "Idling".to_string()
    }
}
