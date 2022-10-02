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

use std::cell::RefCell;

use super::started::Started;
use crate::UpdateInfo;

pub type Tree = egui_dock::Tree<Box<dyn Tab>>;

/// Helper struct for tabs.
pub struct Tabs {
    tree: RefCell<Tree>,
}

impl Default for Tabs {
    fn default() -> Self {
        // Add the basic "get started" tab
        Self {
            tree: RefCell::new(Tree::new(vec![Box::new(Started::new())])),
        }
    }
}

impl Tabs {
    pub fn ui(&self, ui: &mut egui::Ui, info: &UpdateInfo<'_>) {
        ui.group(|ui| {
            egui_dock::DockArea::new(&mut self.tree.borrow_mut())
                .show_inside(ui, &mut TabViewer { info });
        });
    }

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

    pub fn clean_tabs(&self) {
        todo!()
    }
}

pub struct TabViewer<'a> {
    pub info: &'a UpdateInfo<'a>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
    type Tab = Box<dyn Tab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.show(ui, self.info);
    }
}

pub trait Tab {
    fn name(&self) -> String;

    fn show(&mut self, ui: &mut egui::Ui, info: &UpdateInfo<'_>);

    fn requires_filesystem(&self) -> bool {
        false
    }
}
