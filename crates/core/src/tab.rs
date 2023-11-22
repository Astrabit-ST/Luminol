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

/// Helper struct for tabs.
pub struct Tabs {
    dock_state: egui_dock::DockState<Box<dyn Tab>>,

    id: egui::Id,
}

#[derive(Default)]
pub struct EditTabs {
    clean_fn: Option<CleanFn>,
    added: Vec<Box<dyn Tab>>,
    removed: std::collections::HashSet<egui::Id>,
}

type CleanFn = Box<dyn Fn(&Box<dyn Tab>) -> bool>;

struct TabViewer<'a, 'res> {
    // FIXME: variance
    update_state: &'a mut crate::UpdateState<'res>,
}

impl Tabs {
    pub fn new(id: impl std::hash::Hash) -> Self {
        Self {
            id: egui::Id::new(id),
            dock_state: egui_dock::DockState::new(Vec::with_capacity(4)),
        }
    }

    /// Create a new Tab viewer without any tabs.
    pub fn new_with_tabs(id: impl std::hash::Hash, tabs: Vec<impl Tab + 'static>) -> Self {
        Self {
            id: egui::Id::new(id),
            dock_state: egui_dock::DockState::new(
                tabs.into_iter().map(|t| Box::new(t) as Box<_>).collect(),
            ),
        }
    }

    pub fn process_edit_tabs(&mut self, mut edit_tabs: EditTabs) {
        for tab in edit_tabs.added.drain(..) {
            self.add_boxed_tab(tab)
        }
        if let Some(f) = edit_tabs.clean_fn.take() {
            self.clean_tabs(f);
        }
    }

    pub fn ui_without_edit(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut crate::UpdateState<'_>,
    ) {
        let mut style = egui_dock::Style::from_egui(ui.style());
        style.overlay.surface_fade_opacity = 1.;

        egui_dock::DockArea::new(&mut self.dock_state)
            .id(self.id)
            .style(style)
            .show_inside(ui, &mut TabViewer { update_state });
    }

    /// Display all tabs.
    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut crate::UpdateState) {
        let mut edit_tabs = EditTabs::default();
        let mut update_state = update_state.reborrow_with_edit_tabs(&mut edit_tabs);
        self.ui_without_edit(ui, &mut update_state);
        self.process_edit_tabs(edit_tabs);
    }

    /// Add a tab.
    pub fn add_tab(&mut self, tab: impl Tab + 'static) {
        self.add_boxed_tab(Box::new(tab))
    }

    fn add_boxed_tab(&mut self, tab: Box<dyn Tab>) {
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
    pub fn clean_tabs(&mut self, mut f: impl Fn(&Box<dyn Tab>) -> bool) {
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

impl EditTabs {
    pub fn clean(&mut self, f: impl Fn(&Box<dyn Tab>) -> bool + 'static) {
        self.clean_fn = Some(Box::new(f));
    }

    pub fn add_tab(&mut self, tab: impl Tab + 'static) {
        self.added.push(Box::new(tab))
    }

    pub fn remove_tab<T>(&mut self, tab: &impl Tab) -> bool {
        self.remove_tab_by_id(tab.id())
    }

    pub fn remove_tab_by_id(&mut self, id: egui::Id) -> bool {
        self.removed.insert(id)
    }
}

impl<'a, 'res> egui_dock::TabViewer for TabViewer<'a, 'res> {
    type Tab = Box<dyn Tab>;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.name(self.update_state).into()
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
    fn name(&self, _: &crate::UpdateState<'_>) -> String {
        "Untitled Window".to_string()
    }

    /// Required to prevent duplication.
    fn id(&self) -> egui::Id;

    /// Show this tab.
    fn show(&mut self, ui: &mut egui::Ui, update_state: &mut crate::UpdateState<'_>);

    /// Does this tab need the filesystem?
    fn requires_filesystem(&self) -> bool {
        false
    }

    /// Does this tab need to be closed?
    fn force_close(&mut self) -> bool {
        false
    }
}

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
