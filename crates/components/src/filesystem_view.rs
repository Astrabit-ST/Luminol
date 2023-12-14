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

use itertools::Itertools;

pub struct FileSystemView<T> {
    arena: indextree::Arena<Entry>,
    id: egui::Id,
    filesystem: T,
    root_name: String,
    root_node_id: indextree::NodeId,
    row_index: usize,
}

enum Entry {
    File {
        /// Name of this file with extension.
        name: String,
        /// Whether or not this file is selected in the filesystem view.
        selected: bool,
    },
    Dir {
        /// Name of this directory.
        name: String,
        /// Whether or not we've cached the contents of this directory.
        initialized: bool,
        /// Whether or not the memory for this subtree's collapsing header has been deleted.
        depersisted: bool,
        /// Whether or not this directory is fully selected in the filesystem view.
        selected: bool,
        /// Whether or not the subtree for this directory is expanded.
        expanded: bool,
        /// Number of files and directories that are subentries of this one. Only includes direct
        /// children, not indirect descendants.
        total_children: usize,
        /// Number of file and directories that are subentries of this one and are (fully)
        /// selected. Only includes direct children, not indirect descendants.
        selected_children: usize,
        /// Number of subdirectories that are partially selected. Only includes direct children,
        /// not indirect descendants.
        partial_children: usize,
    },
}

impl Entry {
    fn name(&self) -> &str {
        match self {
            Entry::File { name, .. } => name,
            Entry::Dir { name, .. } => name,
        }
    }
}

impl<T> FileSystemView<T>
where
    T: luminol_filesystem::FileSystem,
{
    pub fn new(id: egui::Id, filesystem: T, root_name: String) -> Self {
        let mut arena = indextree::Arena::new();
        let root_node_id = arena.new_node(Entry::Dir {
            name: "".to_string(),
            initialized: false,
            depersisted: false,
            selected: false,
            expanded: true,
            total_children: 0,
            selected_children: 0,
            partial_children: 0,
        });
        Self {
            arena,
            id,
            filesystem,
            root_name,
            root_node_id,
            row_index: 0,
        }
    }

    pub fn filesystem(&self) -> &T {
        &self.filesystem
    }

    pub fn root_name(&self) -> &str {
        &self.root_name
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) -> luminol_filesystem::Result<()> {
        self.row_index = 0;
        self.render_subtree(ui, self.root_node_id, &self.root_name.to_string())
    }

    fn render_subtree(
        &mut self,
        ui: &mut egui::Ui,
        node_id: indextree::NodeId,
        name: &str,
    ) -> luminol_filesystem::Result<()> {
        let mut length = None;

        if let Entry::Dir {
            initialized: initialized @ false,
            selected,
            expanded: true,
            ..
        } = self.arena[node_id].get_mut()
        {
            let selected = *selected;
            *initialized = true;

            let mut ancestors = node_id
                .ancestors(&self.arena)
                .filter_map(|n| {
                    let name = self.arena[n].get().name();
                    (!name.is_empty()).then_some(name)
                })
                .collect_vec();
            ancestors.reverse();
            let path = ancestors.join("/");

            let mut subentries = self.filesystem.read_dir(path)?;
            subentries.sort_unstable_by(|a, b| {
                let path_a = a.path.iter().next_back().unwrap();
                let path_b = b.path.iter().next_back().unwrap();
                path_a.partial_cmp(path_b).unwrap()
            });
            length = Some(subentries.len());

            for subentry in subentries {
                let subentry_name = subentry.path.iter().next_back().unwrap().to_string();
                if subentry.metadata.is_file {
                    node_id.append_value(
                        Entry::File {
                            name: subentry_name,
                            selected,
                        },
                        &mut self.arena,
                    );
                } else {
                    node_id.append_value(
                        Entry::Dir {
                            name: subentry_name,
                            initialized: false,
                            depersisted: false,
                            selected,
                            expanded: false,
                            total_children: 0,
                            selected_children: 0,
                            partial_children: 0,
                        },
                        &mut self.arena,
                    );
                }
            }
        }

        if let Some(length) = length {
            if let Entry::Dir {
                selected,
                total_children,
                selected_children,
                ..
            } = self.arena[node_id].get_mut()
            {
                *total_children = length;
                if *selected {
                    *selected_children = length;
                }
            }
        }

        let mut should_toggle = false;

        let mut frame = egui::containers::Frame::none();
        if self.row_index % 2 != 0 {
            frame = frame.fill(ui.visuals().faint_bg_color);
        }
        self.row_index += 1;

        match self.arena[node_id].get_mut() {
            Entry::File { name, selected } => {
                frame.show(ui, |ui| {
                    if ui
                        .add(egui::SelectableLabel::new(*selected, name.to_string()))
                        .clicked()
                    {
                        should_toggle = true;
                    };
                });
            }
            Entry::Dir {
                depersisted,
                selected,
                expanded,
                selected_children,
                partial_children,
                ..
            } => {
                let id = self.id.with(node_id);

                // De-persist state of the collapsing headers since the underlying filesystem may
                // have changed since this view was last used
                if !*depersisted {
                    *depersisted = true;
                    egui::collapsing_header::CollapsingState::load(ui.ctx(), id)
                        .map(|h| h.remove(ui.ctx()));
                    ui.ctx().animate_bool_with_time(id, *expanded, 0.);
                }

                let header = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    *expanded,
                );

                *expanded = header.openness(ui.ctx()) >= 0.2;

                let layout = *ui.layout();
                let (_response, _header_response, body_response) = header
                    .show_header(ui, |ui| {
                        ui.with_layout(layout, |ui| {
                            frame.show(ui, |ui| {
                                if ui
                                    .add(egui::SelectableLabel::new(
                                        *selected,
                                        format!(
                                            "{}   {}",
                                            if *selected {
                                                '▣'
                                            } else if *selected_children == 0
                                                && *partial_children == 0
                                            {
                                                '☐'
                                            } else {
                                                '⊟'
                                            },
                                            name
                                        ),
                                    ))
                                    .clicked()
                                {
                                    should_toggle = true;
                                };
                            });
                        });
                    })
                    .body::<luminol_filesystem::Result<()>>(|ui| {
                        for node_id in node_id.children(&self.arena).collect_vec() {
                            self.render_subtree(
                                ui,
                                node_id,
                                &self.arena[node_id].get().name().to_string(),
                            )?;
                        }
                        Ok(())
                    });

                if let Some(body_response) = body_response {
                    body_response.inner?;
                }
            }
        }

        if should_toggle {
            self.toggle(node_id);
        }

        Ok(())
    }

    fn toggle(&mut self, node_id: indextree::NodeId) {
        match self.arena[node_id].get() {
            Entry::File { selected, .. } => {
                if *selected {
                    self.deselect(node_id)
                } else {
                    self.select(node_id)
                }
            }
            Entry::Dir { selected, .. } => {
                if *selected {
                    self.deselect(node_id)
                } else {
                    self.select(node_id)
                }
            }
        }
    }

    /// Marks the given node as (completely) selected. Also marks all descendant nodes as selected
    /// and updates ancestor nodes correspondingly.
    ///
    /// When run m times in a row (without running `deselect`) on arbitrary nodes in a tree with n
    /// nodes, this takes worst case O(m + n) time thanks to memoization.
    fn select(&mut self, node_id: indextree::NodeId) {
        // We can skip nodes that are marked as selected because they're guaranteed to have all of
        // their subentries selected as well
        if matches!(self.arena[node_id].get(), Entry::Dir { selected: true, .. }) {
            return;
        }

        // Select all of this node's descendants in a postorder traversal
        for node_id in node_id.children(&self.arena).collect_vec() {
            self.select(node_id);
        }

        let mut child_is_selected = true;
        let mut child_was_partial = false;

        // Select this node
        match self.arena[node_id].get_mut() {
            Entry::File { selected, .. } => {
                if *selected {
                    return;
                }
                *selected = true;
            }
            Entry::Dir {
                selected,
                total_children,
                selected_children,
                partial_children,
                ..
            } => {
                if *selected {
                    return;
                }
                *selected = true;
                child_was_partial = *selected_children != 0 || *partial_children != 0;
                *selected_children = *total_children;
                *partial_children = 0;
            }
        }

        // Visit and update ancestor nodes until we either reach the root node or we reach an
        // ancestor that does not change state (not selected / completely selected / partially
        // selected) after updating it (that implies that the ancestors of *that* node will also
        // not change state after updating, so visiting them would be redundant)
        for node_id in node_id.ancestors(&self.arena).skip(1).collect_vec() {
            if let Entry::Dir {
                selected,
                total_children,
                selected_children,
                partial_children,
                ..
            } = self.arena[node_id].get_mut()
            {
                let was_partial = *selected_children != 0 || *partial_children != 0;
                if child_is_selected {
                    *selected_children += 1;
                    if child_was_partial {
                        *partial_children -= 1;
                    }
                } else if !child_was_partial {
                    *partial_children += 1;
                }
                let is_selected = *selected_children == *total_children;
                if is_selected {
                    *selected = true;
                } else if was_partial {
                    break;
                }
                child_is_selected = is_selected;
                child_was_partial = was_partial;
            }
        }
    }

    /// Marks the given node as (completely) deselected. Also marks all descendant nodes as
    /// deselected and updates ancestor nodes correspondingly.
    ///
    /// When run m times in a row (without running `select`) on arbitrary nodes in a tree with n
    /// nodes, this takes worst case O(m + n) time thanks to memoization.
    fn deselect(&mut self, node_id: indextree::NodeId) {
        // We can skip nodes that are not marked as completely selected and have zero selected or
        // partially selected children
        match self.arena[node_id].get() {
            Entry::File { selected, .. } => {
                if !*selected {
                    return;
                }
            }
            Entry::Dir {
                selected,
                selected_children,
                partial_children,
                ..
            } => {
                if !*selected && *selected_children == 0 && *partial_children == 0 {
                    return;
                }
            }
        }

        // Deelect all of this node's descendants in a postorder traversal
        for node_id in node_id.children(&self.arena).collect_vec() {
            self.deselect(node_id);
        }

        let mut child_is_deselected = true;
        let mut child_was_partial = false;

        // Deselect this node
        match self.arena[node_id].get_mut() {
            Entry::File { selected, .. } => {
                if !*selected {
                    return;
                }
                *selected = false;
            }
            Entry::Dir {
                selected,
                total_children,
                selected_children,
                partial_children,
                ..
            } => {
                if !*selected && *selected_children == 0 && *partial_children == 0 {
                    return;
                }
                *selected = false;
                child_was_partial = *selected_children != *total_children;
                *selected_children = 0;
                *partial_children = 0;
            }
        }

        // Visit and update ancestor nodes until we either reach the root node or we reach an
        // ancestor that does not change state (not selected / completely selected / partially
        // selected) after updating it (that implies that the ancestors of *that* node will also
        // not change state after updating, so visiting them would be redundant)
        for node_id in node_id.ancestors(&self.arena).skip(1).collect_vec() {
            if let Entry::Dir {
                selected,
                total_children,
                selected_children,
                partial_children,
                ..
            } = self.arena[node_id].get_mut()
            {
                *selected = false;
                let was_partial = *selected_children != *total_children;
                if child_was_partial {
                    *partial_children -= 1;
                } else {
                    *selected_children -= 1;
                    if !child_is_deselected {
                        *partial_children += 1;
                    }
                }
                let is_deselected = *selected_children == 0 && *partial_children == 0;
                if !is_deselected && was_partial {
                    break;
                }
                child_is_deselected = is_deselected;
                child_was_partial = was_partial;
            }
        }
    }
}
