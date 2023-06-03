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

use parking_lot::Mutex;
use std::hash::Hash;

/// The tree type;
type Tree<T> = egui_dock::Tree<T>;

/// Helper struct for tabs.
pub struct Tabs<T> {
    tree: Mutex<Tree<T>>,
    id: egui::Id,
}

impl<T> Tabs<T>
where
    T: Tab,
{
    /// Create a new Tab viewer without any tabs.
    pub fn new(id: impl Hash, tabs: Vec<T>) -> Self {
        Self {
            id: egui::Id::new(id),
            tree: Tree::new(tabs).into(),
        }
    }

    /// Display all tabs.
    pub fn ui(&self, ui: &mut egui::Ui) {
        egui_dock::DockArea::new(&mut self.tree.lock())
            .id(self.id)
            .show_inside(
                ui,
                &mut TabViewer {
                    marker: std::marker::PhantomData,
                },
            );
    }

    /// Add a tab.
    pub fn add_tab(&self, tab: T) {
        let mut tree = self.tree.lock();
        for n in tree.iter() {
            if let egui_dock::Node::Leaf { tabs, .. } = n {
                if tabs.iter().any(|t| t.id() == tab.id()) {
                    return;
                }
            }
        }
        tree.push_to_focused_leaf(tab);
    }

    /// Clean tabs by if they need the filesystem.
    pub fn clean_tabs<F: FnMut(&mut T) -> bool>(&self, mut f: F) {
        let mut tree = self.tree.lock();
        for node in tree.iter_mut() {
            if let egui_dock::Node::Leaf { tabs, .. } = node {
                tabs.drain_filter(&mut f);
            }
        }
    }

    /// Returns the name of the focused tab.
    pub fn focused_name(&self) -> Option<String> {
        let mut tree = self.tree.lock();
        tree.find_active().map(|(_, t)| t.name())
    }
}

struct TabViewer<T: Tab> {
    // we don't actually own any types of T, but we use them in TabViewer
    // *const is used here to avoid needing lifetimes and to indicate to the drop checker that we don't own any types of T
    marker: std::marker::PhantomData<*const T>,
}

impl<T> egui_dock::TabViewer for TabViewer<T>
where
    T: Tab,
{
    type Tab = T;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.push_id(tab.id(), |ui| tab.show(ui));
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        tab.force_close()
    }
}

/// A tab trait.
pub trait Tab {
    /// Optionally used as the title of the tab.
    fn name(&self) -> String {
        "Untitled Window".to_string()
    }

    /// Required to prevent duplication.
    fn id(&self) -> egui::Id;

    /// Show this tab.
    fn show(&mut self, ui: &mut egui::Ui);

    /// Does this tab need the filesystem?
    fn requires_filesystem(&self) -> bool {
        false
    }

    /// Does this tab need to be closed?
    fn force_close(&mut self) -> bool {
        false
    }
}

impl Tab for Box<dyn Tab + Send> {
    fn force_close(&mut self) -> bool {
        self.as_mut().force_close()
    }

    fn name(&self) -> String {
        self.as_ref().name()
    }

    fn id(&self) -> egui::Id {
        self.as_ref().id()
    }

    fn requires_filesystem(&self) -> bool {
        self.as_ref().requires_filesystem()
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        self.as_mut().show(ui)
    }
}
