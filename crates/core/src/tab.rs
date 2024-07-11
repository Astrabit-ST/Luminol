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

/// Helper struct for tabs.
pub struct Tabs {
    dock_state: egui_dock::DockState<Box<dyn Tab>>,

    id: egui::Id,
    allowed_in_windows: bool,
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
    focused_id: Option<egui::Id>,
    allowed_in_windows: bool,
}

impl Tabs {
    pub fn new(id: impl std::hash::Hash, allowed_in_windows: bool) -> Self {
        Self {
            id: egui::Id::new(id),
            allowed_in_windows,
            dock_state: egui_dock::DockState::new(Vec::with_capacity(4)),
        }
    }

    /// Create a new Tab viewer without any tabs.
    pub fn new_with_tabs(
        id: impl std::hash::Hash,
        tabs: Vec<impl Tab + 'static>,
        allowed_in_windows: bool,
    ) -> Self {
        Self {
            id: egui::Id::new(id),
            allowed_in_windows,
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

        let focused_id = ui
            .memory(|m| m.focused().is_none())
            .then_some(self.dock_state.find_active_focused().map(|(_, t)| t.id()))
            .flatten();
        egui_dock::DockArea::new(&mut self.dock_state)
            .id(self.id)
            .style(style)
            .show_inside(
                ui,
                &mut TabViewer {
                    update_state,
                    focused_id,
                    allowed_in_windows: self.allowed_in_windows,
                },
            );
    }

    /// Display all tabs.
    pub fn ui(&mut self, ui: &mut egui::Ui, update_state: &mut crate::UpdateState<'_>) {
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
        for (_, node) in self.dock_state.iter_all_nodes() {
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
        let focused_id = self
            .dock_state
            .find_active_focused()
            .map(|(_, tab)| tab.id());
        let focused_leaf = self.dock_state.focused_leaf();
        let mut focused_leaf_was_removed = focused_leaf.is_none();

        // i hate egui dock
        for i in 0.. {
            let Some(surface) = self.dock_state.get_surface_mut(egui_dock::SurfaceIndex(i)) else {
                break;
            };

            if let Some(tree) = surface.node_tree_mut() {
                let mut is_window_empty = !egui_dock::SurfaceIndex(i).is_main();
                let mut empty_leaves = Vec::new();

                for (j, node) in tree.iter_mut().enumerate() {
                    if let egui_dock::Node::Leaf { active, tabs, .. } = node {
                        tabs.retain(&mut f);

                        if !tabs.is_empty() {
                            is_window_empty = false;
                        } else {
                            empty_leaves.push(egui_dock::NodeIndex(j));
                            if focused_leaf.is_some_and(|(surface_index, node_index)| {
                                i == surface_index.0 && j == node_index.0
                            }) {
                                focused_leaf_was_removed = true;
                            }
                        }

                        if let Some((k, _)) = focused_id
                            .and_then(|id| tabs.iter().enumerate().find(|(_, tab)| tab.id() == id))
                        {
                            // If the previously focused tab hasn't been removed, refocus it
                            // since its index in the `tabs` array may have changed
                            *active = egui_dock::TabIndex(k);
                        } else if active.0 >= tabs.len() {
                            // If the active tab index for this leaf node is out of bounds, reset
                            // it to the first tab in this node
                            *active = egui_dock::TabIndex(0);
                        }
                    }
                }

                if is_window_empty {
                    // Remove empty windows
                    self.dock_state.remove_surface(egui_dock::SurfaceIndex(i));
                } else {
                    for node_index in empty_leaves {
                        // Remove empty leaf nodes
                        tree.remove_leaf(node_index);
                    }
                }
            }
        }

        // If the previously focused leaf node was removed, unfocus all tabs
        if focused_leaf_was_removed {
            self.dock_state.set_focused_node_and_surface((
                egui_dock::SurfaceIndex(usize::MAX),
                egui_dock::NodeIndex(usize::MAX),
            ));
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

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        tab.id()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let id = tab.id();
        ui.push_id(id, |ui| {
            tab.show(
                ui,
                self.update_state,
                self.focused_id.is_some_and(|focused_id| focused_id == id),
            )
        });
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        tab.force_close()
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        // We need to disable scroll bars for at least the map editor because otherwise it'll start
        // jiggling when the screen or tab is resized. We're not making that type of game.
        [false, false]
    }

    fn allowed_in_windows(&self, _tab: &mut Self::Tab) -> bool {
        self.allowed_in_windows
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
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        update_state: &mut crate::UpdateState<'_>,
        is_focused: bool,
    );

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
        is_focused: bool,
    ) where
        W: crate::Window,
        T: Tab,
    {
        self.as_mut().show(ui, update_state, is_focused)
    }
}
*/
