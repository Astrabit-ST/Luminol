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

use qp_trie::wrapper::{BStr, BString};

type Trie<T> = qp_trie::Trie<BString, DirTrie<T>>;
type DirTrie<T> = qp_trie::Trie<BString, Option<T>>;

pub struct FileSystemTrieIter<'a, T>(FileSystemTrieIterInner<'a, T>);

enum FileSystemTrieIterInner<'a, T> {
    Direct(qp_trie::Iter<'a, BString, Option<T>>),
    Prefix(std::iter::Once<(&'a str, Option<&'a T>)>),
}

impl<'a, T> std::iter::FusedIterator for FileSystemTrieIter<'a, T> {}

impl<'a, T> Iterator for FileSystemTrieIter<'a, T> {
    type Item = (&'a str, Option<&'a T>);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            FileSystemTrieIterInner::Direct(iter) => iter
                .next()
                .map(|(key, value)| (key.as_str(), value.as_ref())),
            FileSystemTrieIterInner::Prefix(iter) => iter.next(),
        }
    }
}

/// A container for a directory tree-like cache where the "files" are a data type of your choice.
#[derive(Debug, Clone)]
pub struct FileSystemTrie<T>(Trie<T>);

impl<T> Default for FileSystemTrie<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> FileSystemTrie<T> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a directory to the trie. If parent directories do not exist, they will be created.
    pub fn create_dir(&mut self, path: impl AsRef<camino::Utf8Path>) {
        let path = path.as_ref();

        // Nothing to do if the path is already in the trie
        if self.0.contains_key_str(path.as_str()) {
            return;
        }

        // Otherwise, find the longest prefix in the trie that is shared with some directory
        // path that is in the trie
        let prefix = self.get_dir_prefix(path).as_str().to_string();

        // Check if the trie already contains an entry for this prefix, and if not, create one
        if !self.0.contains_key_str(&prefix) {
            let mut dir_trie = DirTrie::new();
            if let Some((child_path, _)) = self.0.iter_prefix_str(&prefix).skip(1).next() {
                // If there is a different path in the trie that has this as a prefix (there can be
                // at most one), add it to this directory entry
                dir_trie.insert_str(
                    camino::Utf8Path::new(child_path.as_str())
                        .strip_prefix(&prefix)
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap(),
                    None,
                );
            }
            self.0.insert_str(&prefix, dir_trie);
        }

        // Add the new path to the entry at this prefix
        let prefix_trie = self.0.get_mut_str(&prefix).unwrap();
        prefix_trie.insert_str(
            path.strip_prefix(&prefix).unwrap().iter().next().unwrap(),
            None,
        );

        // Add the actual entry for the new path
        self.0.insert_str(path.as_str(), DirTrie::new());
    }

    /// Adds a file to the trie with the given value. If parent directories do not exist, they will
    /// be created. If there already was a file at the given path, the original value will be
    /// returned.
    pub fn create_file(&mut self, path: impl AsRef<camino::Utf8Path>, value: T) -> Option<T> {
        let path = path.as_ref().to_owned();

        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        let filename = path.iter().next_back().unwrap();

        // Add the parent directory to the trie
        self.create_dir(dir);
        let dir_trie = self.0.get_mut_str(dir.as_str()).unwrap();

        if let Some(option) = dir_trie.get_mut_str(filename) {
            // If the path is already in the trie, replace the value and return the original
            option.replace(value)
        } else {
            // Add the file to the parent directory's entry
            dir_trie.insert_str(filename, Some(value));
            None
        }
    }

    /// Returns whether or not a directory exists at the given path.
    pub fn contains_dir(&self, path: impl AsRef<camino::Utf8Path>) -> bool {
        let path = path.as_ref().as_str();
        if path.is_empty() {
            return true;
        }
        self.0.contains_key_str(path)
            || self
                .0
                .longest_common_prefix::<BStr>(format!("{path}/").as_str().into())
                .as_str()
                .len()
                == path.len() + 1
    }

    /// Returns whether or not a file exists at the given path.
    pub fn contains_file(&self, path: impl AsRef<camino::Utf8Path>) -> bool {
        let path = path.as_ref();
        let Some(filename) = path.iter().next_back() else {
            return false;
        };
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_str(dir.as_str())
            .map_or(false, |dir_trie| dir_trie.contains_key_str(filename))
    }

    /// Returns whether or not a file or directory exists at the given path.
    pub fn contains(&self, path: impl AsRef<camino::Utf8Path>) -> bool {
        self.contains_file(&path) || self.contains_dir(&path)
    }

    /// Gets the number of direct children in the directory at the given path, if it exists.
    pub fn get_dir_size(&self, path: impl AsRef<camino::Utf8Path>) -> Option<usize> {
        let path = path.as_ref().as_str();
        if let Some(dir_trie) = self.0.get_str(path) {
            Some(dir_trie.count())
        } else if self
            .0
            .longest_common_prefix::<BStr>(format!("{path}/").as_str().into())
            .as_str()
            .len()
            == path.len() + 1
        {
            Some(1)
        } else {
            None
        }
    }

    /// Gets an immutable reference to the value in the file at the given path, if it exists.
    pub fn get_file(&self, path: impl AsRef<camino::Utf8Path>) -> Option<&T> {
        let path = path.as_ref();
        let Some(filename) = path.iter().next_back() else {
            return None;
        };
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.get_str(filename))
            .map(|o| o.as_ref())
            .flatten()
    }

    /// Gets a mutable reference to the value in the file at the given path, if it exists.
    pub fn get_mut_file(&mut self, path: impl AsRef<camino::Utf8Path>) -> Option<&mut T> {
        let path = path.as_ref();
        let Some(filename) = path.iter().next_back() else {
            return None;
        };
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_mut_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.get_mut_str(filename))
            .map(|o| o.as_mut())
            .flatten()
    }

    /// Gets the longest prefix of the given path that is a prefix of the path of a directory in
    /// the trie.
    pub fn get_dir_prefix(&self, path: impl AsRef<camino::Utf8Path>) -> &camino::Utf8Path {
        let path = path.as_ref().as_str();
        if self.0.contains_key_str(path) {
            return self
                .0
                .longest_common_prefix::<BStr>(path.into())
                .as_str()
                .into();
        }
        let prefix = self
            .0
            .longest_common_prefix::<BStr>(format!("{}/", path).as_str().into())
            .as_str();
        let prefix = if !self.0.contains_key_str(prefix) {
            prefix.rsplit_once('/').map(|(s, _)| s).unwrap_or(prefix)
        } else {
            prefix
        };
        prefix.into()
    }

    /// Removes the file at the given path if it exists, and returns the original value.
    pub fn remove_file(&mut self, path: impl AsRef<camino::Utf8Path>) -> Option<T> {
        let path = path.as_ref();
        let Some(filename) = path.iter().next_back() else {
            return None;
        };
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_mut_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.remove_str(filename))
            .flatten()
    }

    /// Removes the directory at the given path and all of its contents if it exists, and returns
    /// whether or not it existed.
    pub fn remove_dir(&mut self, path: impl AsRef<camino::Utf8Path>) -> bool {
        let path = path.as_ref().as_str();
        if path.is_empty() {
            true
        } else if self.0.contains_key_str(path) {
            self.0.remove_prefix_str(&format!("{path}/"));
            self.0.remove_str(path);
            true
        } else if self
            .0
            .longest_common_prefix::<BStr>(format!("{path}/").as_str().into())
            .as_str()
            .len()
            == path.len() + 1
        {
            self.0.remove_prefix_str(&format!("{path}/"));
            true
        } else {
            false
        }
    }

    /// Given the path to a directory, returns an iterator over its children if it exists.
    /// The iterator's items are of the form `(key, value)` where `key` is the name of the child as
    /// `&str` and `value` is the data of the child if it's a file, as `Option<&T>`.
    pub fn iter(&self, path: impl AsRef<camino::Utf8Path>) -> Option<FileSystemTrieIter<'_, T>> {
        let path = path.as_ref();
        if let Some(dir_trie) = self.0.get_str(path.as_str()) {
            Some(FileSystemTrieIter(FileSystemTrieIterInner::Direct(
                dir_trie.iter(),
            )))
        } else if let Some((key, _)) = self.0.iter_prefix_str(path.as_str()).next() {
            Some(FileSystemTrieIter(FileSystemTrieIterInner::Prefix(
                std::iter::once((
                    camino::Utf8Path::new(key.as_str())
                        .strip_prefix(path)
                        .unwrap()
                        .iter()
                        .next()
                        .unwrap(),
                    None,
                )),
            )))
        } else {
            None
        }
    }
}
