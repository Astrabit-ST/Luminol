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

use crate::Window;
use std::hash::Hash;

/// Helper struct for tabs.
pub struct Tabs<T> {
    dock_state: egui_dock::DockState<T>,

    id: egui::Id,
}

pub struct EditTabs<T> {
    clean_fn: Option<CleanFn<T>>,
    added: Vec<T>,
    removed: std::collections::HashSet<egui::Id>,
}

type CleanFn<T> = Box<dyn Fn(&T) -> bool>;

impl<T> Default for EditTabs<T> {
    fn default() -> Self {
        Self {
            clean_fn: None,
            added: Vec::new(),
            removed: std::collections::HashSet::new(),
        }
    }
}

struct TabViewer<'a, 'res, W, T>
where
    T: Tab,
{
    // FIXME: variance
    update_state: &'a mut crate::UpdateState<'res, W, T>,
}

impl<T> Tabs<T>
where
    T: Tab,
{
    /// Create a new Tab viewer without any tabs.
    pub fn new(id: impl Hash, tabs: Vec<T>) -> Self {
        Self {
            id: egui::Id::new(id),
            dock_state: egui_dock::DockState::new(tabs),
        }
    }

    /// Display all tabs.
    pub fn ui<W, O>(&mut self, ui: &mut egui::Ui, update_state: &mut crate::UpdateState<'_, W, O>)
    where
        W: Window,
    {
        let mut edit_tabs = EditTabs::default();
        let mut update_state = update_state.reborrow_with_edit_tabs(&mut edit_tabs);

        egui_dock::DockArea::new(&mut self.dock_state)
            .id(self.id)
            .show_inside(
                ui,
                &mut TabViewer {
                    update_state: &mut update_state,
                },
            );

        for tab in edit_tabs.added {
            self.add_tab(tab);
        }
        if let Some(f) = edit_tabs.clean_fn {
            self.clean_tabs(f);
        }
    }

    /// Add a tab.
    pub fn add_tab(&mut self, tab: T) {
        // FIXME O(n)
        for node in self.dock_state.iter_nodes() {
            if let egui_dock::Node::Leaf { tabs, .. } = node {
                if tabs.iter().any(|t| t.id() == tab.id()) {
                    return;
                }
            }
        }
        self.dock_state.push_to_focused_leaf(tab);
    }

    /// Removes tabs that the provided closure returns `false` when called.
    pub fn clean_tabs(&mut self, mut f: impl Fn(&T) -> bool) {
        // i hate egui dock
        for i in 0.. {
            let Some(surface) = self.dock_state.get_surface_mut(egui_dock::SurfaceIndex(i)) else {
                break;
            };
            if let Some(tree) = surface.node_tree_mut() {
                for node in tree.iter_mut() {
                    if let egui_dock::Node::Leaf { tabs, .. } = node {
                        tabs.retain(&mut f);
                    }
                }
            }
        }
    }

    /// Returns the name of the focused tab.
    pub fn focused_name(&self) -> Option<String> {
        None
    }
}

impl<T> EditTabs<T>
where
    T: Tab,
{
    pub fn clean(&mut self, f: impl Fn(&T) -> bool + 'static) {
        self.clean_fn = Some(Box::new(f));
    }

    pub fn add_tab(&mut self, tab: impl Into<T>) {
        self.added.push(tab.into())
    }

    pub fn remove_tab(&mut self, tab: &T) -> bool {
        self.remove_tab_by_id(tab.id())
    }

    pub fn remove_tab_by_id(&mut self, id: egui::Id) -> bool {
        self.removed.insert(id)
    }
}

impl<'a, 'res, W, T> egui_dock::TabViewer for TabViewer<'a, 'res, W, T>
where
    W: Window,
    T: Tab,
{
    type Tab = T;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.push_id(tab.id(), |ui| tab.show(ui, self.update_state));
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
    fn show<W, T>(&mut self, ui: &mut egui::Ui, update_state: &mut crate::UpdateState<'_, W, T>)
    where
        W: Window,
        T: Tab;

    /// Does this tab need the filesystem?
    fn requires_filesystem(&self) -> bool {
        false
    }

    /// Does this tab need to be closed?
    fn force_close(&mut self) -> bool {
        false
    }
}

// FIXME: not object safe
/*
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

    fn show<'res, W, T>(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut crate::UpdateState<'res, W, T>,
    ) where
        W: crate::Window,
        T: Tab,
    {
        self.as_mut().show(ui, update_state)
    }
}
*/
