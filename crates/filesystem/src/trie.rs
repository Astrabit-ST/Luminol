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

use qp_trie::wrapper::{BStr, BString};

type Trie<T> = qp_trie::Trie<BString, DirTrie<T>>;
type DirTrie<T> = qp_trie::Trie<BString, Option<T>>;

pub struct FileSystemTrieDirIter<'a, T>(FileSystemTrieDirIterInner<'a, T>);

enum FileSystemTrieDirIterInner<'a, T> {
    Direct(qp_trie::Iter<'a, BString, Option<T>>, usize),
    Prefix(std::iter::Once<(&'a str, Option<&'a T>)>),
}

impl<'a, T> std::iter::FusedIterator for FileSystemTrieDirIter<'a, T> {}

impl<'a, T> std::iter::ExactSizeIterator for FileSystemTrieDirIter<'a, T> {
    fn len(&self) -> usize {
        match &self.0 {
            FileSystemTrieDirIterInner::Direct(_, len) => *len,
            FileSystemTrieDirIterInner::Prefix(iter) => iter.len(),
        }
    }
}

impl<'a, T> Iterator for FileSystemTrieDirIter<'a, T> {
    type Item = (&'a str, Option<&'a T>);
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            FileSystemTrieDirIterInner::Direct(iter, len) => {
                *len = len.saturating_sub(1);
                iter.next()
                    .map(|(key, value)| (key.as_str(), value.as_ref()))
            }
            FileSystemTrieDirIterInner::Prefix(iter) => iter.next(),
        }
    }
}

pub struct FileSystemTrieIter<'a, T> {
    path: camino::Utf8PathBuf,
    trie: &'a Trie<T>,
    root_done: bool,
    trie_iter: Option<qp_trie::Iter<'a, BString, DirTrie<T>>>,
    dir_iter: Option<(camino::Utf8PathBuf, qp_trie::Iter<'a, BString, Option<T>>)>,
}

impl<'a, T> std::iter::FusedIterator for FileSystemTrieIter<'a, T> {}

impl<'a, T> Iterator for FileSystemTrieIter<'a, T> {
    type Item = (camino::Utf8PathBuf, &'a T);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((prefix, dir_iter)) = &mut self.dir_iter {
                match dir_iter.next() {
                    Some((filename, Some(value))) => {
                        return Some((format!("{prefix}/{}", filename.as_str()).into(), value));
                    }
                    None => {
                        self.dir_iter = None;
                        self.root_done = true;
                    }
                    _ => {}
                }
            } else if let Some(trie_iter) = &mut self.trie_iter {
                let (prefix, dir_trie) = trie_iter.next()?;
                self.dir_iter = Some((prefix.as_str().to_owned().into(), dir_trie.iter()));
            } else if self.path.as_str() == "" {
                self.root_done = true;
                self.trie_iter = Some(self.trie.iter())
            } else if self.root_done {
                self.trie_iter = Some(
                    self.trie
                        .iter_prefix::<BStr>(format!("{}/", self.path).as_str().into()),
                );
            } else {
                self.dir_iter = self
                    .trie
                    .get_str(self.path.as_str())
                    .map(|t| (self.path.clone(), t.iter()));
            }
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
            let prefix_with_trailing_slash = format!("{}/", &prefix);
            if let Some((child_path, _)) = self
                .0
                .iter_prefix_str(if prefix.is_empty() {
                    ""
                } else {
                    &prefix_with_trailing_slash
                })
                .next()
            {
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
        if let Some(dirname) = path.strip_prefix(&prefix).unwrap().iter().next() {
            let prefix_trie = self.0.get_mut_str(&prefix).unwrap();
            prefix_trie.insert_str(dirname, None);
        }

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
            || (path.is_empty() && self.0.count() != 0)
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
        self.0.get_str(dir.as_str()).map_or(false, |dir_trie| {
            dir_trie
                .get_str(filename)
                .and_then(|o| o.as_ref())
                .is_some()
        })
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
        } else if (path.is_empty() && self.0.count() != 0)
            || self
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
        let filename = path.iter().next_back()?;
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.get_str(filename))
            .and_then(|o| o.as_ref())
    }

    /// Gets a mutable reference to the value in the file at the given path, if it exists.
    pub fn get_file_mut(&mut self, path: impl AsRef<camino::Utf8Path>) -> Option<&mut T> {
        let path = path.as_ref();
        let filename = path.iter().next_back()?;
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_mut_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.get_mut_str(filename))
            .and_then(|o| o.as_mut())
    }

    /// Gets an immutable reference to the value in the file at the given path or creates the file
    /// with the given value if it doesn't.
    pub fn get_or_create_file(&mut self, path: impl AsRef<camino::Utf8Path>, value: T) -> &T {
        let path = path.as_ref();
        if !self.contains_file(path) {
            self.create_file(path, value);
        }
        self.get_file(path).unwrap()
    }

    /// Gets an immutable reference to the value in the file at the given path or creates the file
    /// with the given value if it doesn't.
    pub fn get_or_create_file_with(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        f: impl FnOnce() -> T,
    ) -> &T {
        let path = path.as_ref();
        if !self.contains_file(path) {
            self.create_file(path, f());
        }
        self.get_file(path).unwrap()
    }

    /// Gets a mutable reference to the value in the file at the given path or creates the file
    /// with the given value if it doesn't.
    pub fn get_or_create_file_mut(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        value: T,
    ) -> &mut T {
        let path = path.as_ref();
        if !self.contains_file(path) {
            self.create_file(path, value);
        }
        self.get_file_mut(path).unwrap()
    }

    /// Gets a mutable reference to the value in the file at the given path or creates the file
    /// with the given value if it doesn't.
    pub fn get_or_create_file_with_mut(
        &mut self,
        path: impl AsRef<camino::Utf8Path>,
        f: impl FnOnce() -> T,
    ) -> &mut T {
        let path = path.as_ref();
        if !self.contains_file(path) {
            self.create_file(path, f());
        }
        self.get_file_mut(path).unwrap()
    }

    /// Gets the longest prefix of the given path that is the path of a directory in the trie.
    pub fn get_dir_prefix(&self, path: impl AsRef<camino::Utf8Path>) -> &camino::Utf8Path {
        let path = path.as_ref();
        if self.0.contains_key_str(path.as_str()) {
            return self
                .0
                .longest_common_prefix::<BStr>(path.as_str().into())
                .as_str()
                .into();
        }
        let prefix = self
            .0
            .longest_common_prefix::<BStr>(format!("{path}/").as_str().into())
            .as_str();
        let prefix = if !self.0.contains_key_str(prefix) || !path.starts_with(prefix) {
            prefix.rsplit_once('/').map(|(s, _)| s).unwrap_or_default()
        } else {
            prefix
        };
        prefix.into()
    }

    /// Removes the file at the given path if it exists, and returns the original value.
    pub fn remove_file(&mut self, path: impl AsRef<camino::Utf8Path>) -> Option<T> {
        let path = path.as_ref();
        let filename = path.iter().next_back()?;
        let dir = path.parent().unwrap_or(camino::Utf8Path::new(""));
        self.0
            .get_mut_str(dir.as_str())
            .and_then(|dir_trie| dir_trie.remove_str(filename))
            .flatten()
    }

    /// Removes the directory at the given path and all of its contents if it exists, and returns
    /// whether or not it existed.
    pub fn remove_dir(&mut self, path: impl AsRef<camino::Utf8Path>) -> bool {
        let path = path.as_ref();
        let path_str = path.as_str();
        if path_str.is_empty() {
            self.0.clear();
            true
        } else if self.0.contains_key_str(path_str) {
            self.0.remove_prefix_str(&format!("{path_str}/"));
            self.0.remove_str(path_str);
            if let Some(parent) = path.parent() {
                self.create_dir(parent);
                if let (Some(parent_trie), Some(dirname)) =
                    (self.0.get_mut_str(parent.as_str()), path.iter().next_back())
                {
                    parent_trie.remove_str(dirname);
                }
            }
            true
        } else if self
            .0
            .longest_common_prefix::<BStr>(format!("{path_str}/").as_str().into())
            .as_str()
            .len()
            == path_str.len() + 1
        {
            self.0.remove_prefix_str(&format!("{path_str}/"));
            if let Some(parent) = path.parent() {
                self.create_dir(parent);
                if let (Some(parent_trie), Some(dirname)) =
                    (self.0.get_mut_str(parent.as_str()), path.iter().next_back())
                {
                    parent_trie.remove_str(dirname);
                }
            }
            true
        } else {
            false
        }
    }

    /// Given the path to a directory, returns an iterator over its children if it exists.
    /// The iterator's items are of the form `(key, value)` where `key` is the name of the child as
    /// `&str` and `value` is the data of the child if it's a file, as `Option<&T>`.
    pub fn iter_dir(
        &self,
        path: impl AsRef<camino::Utf8Path>,
    ) -> Option<FileSystemTrieDirIter<'_, T>> {
        let path = path.as_ref();
        let prefix_with_trailing_slash = format!("{}/", path.as_str());
        if let Some(dir_trie) = self.0.get_str(path.as_str()) {
            Some(FileSystemTrieDirIter(FileSystemTrieDirIterInner::Direct(
                dir_trie.iter(),
                dir_trie.count(),
            )))
        } else if let Some((key, _)) = self
            .0
            .iter_prefix_str(if path.as_str().is_empty() {
                ""
            } else {
                &prefix_with_trailing_slash
            })
            .next()
        {
            Some(FileSystemTrieDirIter(FileSystemTrieDirIterInner::Prefix(
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

    /// Given the path to a directory, returns an iterator over immutable references to its
    /// descendant files if it exists.
    pub fn iter_prefix(
        &self,
        path: impl AsRef<camino::Utf8Path>,
    ) -> Option<FileSystemTrieIter<'_, T>> {
        let path = path.as_ref();
        if self.contains_dir(path) {
            Some(FileSystemTrieIter {
                path: path.to_owned(),
                trie: &self.0,
                root_done: false,
                trie_iter: None,
                dir_iter: None,
            })
        } else {
            None
        }
    }
}
