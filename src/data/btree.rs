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

use serde::{Deserialize, Serialize};

/// A simple binary tree.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Node<T> {
    /// The left branch.
    left: Option<Box<Node<T>>>,
    /// The right branch.
    right: Option<Box<Node<T>>>,
    /// The data for this Node.
    pub data: T,
}

/// Branch type
pub enum Branch {
    /// Left branch
    Left,
    /// Right branch
    Right,
}

impl<T> Node<T> {
    /// Create a new Node
    pub fn new(data: T, left: Option<Node<T>>, right: Option<Node<T>>) -> Self {
        Self {
            data,
            left: left.map(Box::new),
            right: right.map(Box::new),
        }
    }

    /// If this node has a left branch.
    pub fn has_left(&self) -> bool {
        self.left.is_some()
    }

    /// If this node has a right branch.
    pub fn has_right(&self) -> bool {
        self.right.is_some()
    }

    /// If this node has a branch, call the provided closure.
    ///
    /// If not, just return [`self`]
    pub fn branch(&mut self, branch: Branch, mut f: impl FnMut(&mut Node<T>)) -> &mut Self {
        if let Some(ref mut branch) = self.get_branch(branch) {
            f(branch.as_mut())
        }
        self
    }

    /// Sever a branch, returning it.
    ///
    /// Returns [`None`] if there was no branch in the first place.
    pub fn sever(&mut self, branch: Branch) -> Option<Node<T>> {
        self.get_branch(branch).take().map(|n| *n)
    }

    /// Insert a [`Node`], by transplanting the original branch onto the provided node.
    ///
    /// This function will return the replaced branch of the provided node.
    /// Returns [`None`] if the replaced branch didn't exist.
    ///
    /// If the provided [`Node`] is [`None`], then the original branch is returned instead.
    /// (Behaves like [`Self::swap`])
    pub fn insert(
        &mut self,
        node: Option<Node<T>>,
        branch_from: Branch,
        branch_to: Branch,
    ) -> Option<Node<T>> {
        // Get the node as a box.
        let mut node = node.map(Box::new);
        let branch = self.get_branch(branch_from);

        // Swap our branch with the node.
        std::mem::swap(branch, &mut node);
        // Swap the new branch's other branch with our old branch.
        if let Some(branch) = branch {
            std::mem::swap(branch.get_branch(branch_to), &mut node);
        }

        // Return what used to be under node.
        node.map(|n| *n)
    }

    /// Swap a node on the branch of this node.
    ///
    /// If this node had something on the branch it is returned.
    pub fn swap(&mut self, node: Option<Node<T>>, branch: Branch) -> Option<Node<T>> {
        let mut node = node.map(Box::new);
        std::mem::swap(self.get_branch(branch), &mut node);
        node.map(|n| *n)
    }

    fn get_branch(&mut self, branch: Branch) -> &mut Option<Box<Node<T>>> {
        match branch {
            Branch::Left => &mut self.left,
            Branch::Right => &mut self.right,
        }
    }
}
