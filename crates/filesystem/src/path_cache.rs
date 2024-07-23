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

use color_eyre::eyre::WrapErr;
use itertools::Itertools;

use crate::{DirEntry, Error, FileSystem as FileSystemTrait, Metadata, OpenFlags, Result};

const TRIE_SUFFIX: &str = "\0";

#[derive(Debug, Clone)]
struct CactusNode {
    /// The path component stored in this cactus stack node.
    value: String,
    /// The index of the next node within the cactus stack, or `None` if there is no next node.
    next: Option<usize>,
    /// One more than the number of times you need to follow `next` until you get `None`.
    len: usize,
}

/// This cache stores the lowercased versions of paths and their corresponding original paths.
/// Given a lowercased path, for example "data/mapinfos", you can find the original path by first
/// appending a forward slash followed by `TRIE_SUFFIX` to the end of the path, then looking up the
/// file at that path in `trie` and then looking up the lowercased file extension.
/// This gives you the index of a node in `cactus`, which stores the
/// original path. To recover the original path, follow the chain of cactus stack nodes by
/// following the `next` field on the nodes. This gives you the path components of the original
/// path in reverse order.
#[derive(Debug, Default, Clone)]
struct Cache {
    trie: crate::FileSystemTrie<qp_trie::Trie<qp_trie::wrapper::BString, usize>>,
    cactus: slab::Slab<CactusNode>,
}

#[derive(Debug)]
pub struct FileSystem<F> {
    fs: F,
    cache: parking_lot::RwLock<Cache>,
}

impl Cache {
    fn get_path_from_cactus_index(&self, index: usize) -> camino::Utf8PathBuf {
        let Some(node) = self.cactus.get(index) else {
            return Default::default();
        };

        let mut vec = Vec::with_capacity(node.len);

        let mut node = Some(node);
        while let Some(n) = node {
            vec.push(&n.value);
            node = n.next.and_then(|next| self.cactus.get(next));
        }

        vec.iter().rev().join(std::path::MAIN_SEPARATOR_STR).into()
    }

    /// Gets the original, case-sensitive version of the given case-insensitive path from the underlying
    /// filesystem and puts it into the cache.
    /// This method memoizes: if we want to insert "this/is/a/path" and the cache already contains
    /// the case-sensitive version of the path "this/is", we will only scan the underlying filesystem
    /// for the case-sensitive names of the remaining two components in the path.
    fn regen(
        &mut self,
        fs: &impl FileSystemTrait,
        path: impl AsRef<camino::Utf8Path>,
    ) -> crate::Result<()> {
        let path = path.as_ref();

        // We don't need to do anything if there already is a path in the trie matching the given
        // path
        if self.desensitize(path).is_some() {
            return Ok(());
        }

        let extension = path.extension().unwrap_or_default().to_string();
        let mut path = to_lowercase(path);
        path.set_extension("");

        // If there is a matching path with a different file extension than this one, we may need
        // to add this new path to the extension trie
        if self.trie.contains_file(with_trie_suffix(&path)) {
            if let Some(mut desensitized_path) = self
                .trie
                .get_file(with_trie_suffix(&path))
                .unwrap()
                .values()
                .next()
                .map(|&cactus_index| self.get_path_from_cactus_index(cactus_index))
            {
                desensitized_path.set_extension(&extension);
                if fs.exists(&desensitized_path)? {
                    let extension_trie = self.trie.get_file_mut(with_trie_suffix(&path)).unwrap();
                    let sibling = self
                        .cactus
                        .get(*extension_trie.values().next().unwrap())
                        .unwrap();
                    let cactus_index = self.cactus.insert(CactusNode {
                        value: desensitized_path.file_name().unwrap_or_default().into(),
                        ..*sibling
                    });
                    extension_trie.insert_str(&extension, cactus_index);
                    return Ok(());
                }
            }
        }

        let extension = extension.to_lowercase();

        let prefix = self.trie.get_dir_prefix(&path);
        let mut cactus_index = (!prefix.as_str().is_empty()).then(|| {
            let extension_trie = self.trie.get_file(with_trie_suffix(prefix)).unwrap();
            *extension_trie
                .get_str(&extension)
                .unwrap_or(extension_trie.values().next().unwrap())
        });
        let mut len = prefix.iter().count();

        // Get the longest prefix of the path that is in the trie, convert it to lowercase and
        // remove file extensions
        let mut lower_string = prefix.to_string();
        if let Some(additional) = path
            .as_str()
            .bytes()
            .len()
            .checked_sub(lower_string.bytes().len())
        {
            lower_string.reserve(additional);
        }

        // This is the same thing as `lower_string` except with the actual letter casing from the
        // filesystem and without removing file extensions
        let mut original_string = cactus_index.map_or_else(Default::default, |i| {
            self.get_path_from_cactus_index(i).to_string()
        });
        if let Some(additional) = path
            .as_str()
            .bytes()
            .len()
            .checked_sub(original_string.bytes().len())
        {
            original_string.reserve(additional);
        }

        // Iterate over the remaining path components that aren't present in
        // `lower_string`/`original_string`
        for name in path.strip_prefix(prefix).unwrap().iter() {
            let entries = fs
                .read_dir(&original_string)
                .wrap_err_with(|| format!("While regenerating cache for path {path:?}"))?;
            len += 1;

            let mut original_name = None;
            let mut new_cactus_index = 0;
            for entry in entries.into_iter() {
                let entry_name = camino::Utf8Path::new(entry.file_name())
                    .file_stem()
                    .unwrap_or(entry.file_name())
                    .to_lowercase();
                let entry_extension = camino::Utf8Path::new(entry.file_name())
                    .extension()
                    .unwrap_or_default()
                    .to_lowercase();
                let index = self.cactus.insert(CactusNode {
                    value: entry.file_name().to_string(),
                    next: cactus_index,
                    len,
                });
                self.trie
                    .get_or_create_file_with_mut(
                        if lower_string.is_empty() {
                            with_trie_suffix(&entry_name)
                        } else {
                            format!("{lower_string}/{entry_name}/{TRIE_SUFFIX}").into()
                        },
                        Default::default,
                    )
                    .insert_str(&entry_extension, index);
                if entry_name == name {
                    original_name = Some(entry.file_name().to_string());
                    new_cactus_index = index;
                }
            }

            let Some(original_name) = original_name else {
                return Ok(());
            };
            if !lower_string.is_empty() {
                lower_string.push('/');
            }
            lower_string.push_str(name);
            if !original_string.is_empty() {
                original_string.push(std::path::MAIN_SEPARATOR);
            }
            original_string.push_str(&original_name);
            cactus_index = Some(new_cactus_index);
        }

        Ok(())
    }

    /// Gets the case-sensitive version of the given case-insensitive path from the cache.
    /// The path has to already exist in the cache; you need to use `.regen` to insert paths into
    /// the cache before this can get them.
    fn desensitize(&self, path: impl AsRef<camino::Utf8Path>) -> Option<camino::Utf8PathBuf> {
        let path = path.as_ref();
        if path.as_str().is_empty() {
            return Some(Default::default());
        }
        let mut path = to_lowercase(path);
        let extension = path.extension().unwrap_or_default().to_string();
        let path_clone = path.clone();
        path.set_extension("");

        // Try to search for the given path exactly (case-insensitive)
        let maybe_exact_match =
            self.trie
                .get_file(with_trie_suffix(&path))
                .and_then(|extension_trie| {
                    extension_trie
                        .get_str(&extension)
                        .map(|&index| self.get_path_from_cactus_index(index))
                });

        maybe_exact_match.or_else(|| {
            // If we didn't find anything the first time, try again using a '.*' glob pattern
            // appended to the end (still case-insensitive)
            let path = path_clone;
            self.trie
                .get_file(with_trie_suffix(path))
                .and_then(|extension_trie| {
                    extension_trie
                        .values()
                        .next()
                        .map(|&index| self.get_path_from_cactus_index(index))
                })
        })
    }
}

impl<F> FileSystem<F>
where
    F: FileSystemTrait,
{
    pub fn new(fs: F) -> Result<Self> {
        let this = FileSystem {
            fs,
            cache: Default::default(),
        };
        Ok(this)
    }

    pub fn fs(&self) -> &F {
        &self.fs
    }

    pub fn rebuild(&self) {
        let mut cache = self.cache.write();
        *cache = Default::default(); // FIXME we don't actually bother rebuilding anything, this is just a reset...
    }

    pub fn debug_ui(&self, ui: &mut egui::Ui) {
        let cache = self.cache.read();

        ui.with_layout(
            egui::Layout {
                cross_justify: true,
                ..*ui.layout()
            },
            |ui| {
                egui::ScrollArea::vertical()
                    .id_source("luminol_path_cache_debug_ui")
                    .show_rows(
                        ui,
                        ui.text_style_height(&egui::TextStyle::Body),
                        cache.cactus.len(),
                        |ui, rows| {
                            for (_, (key, cactus_index)) in cache
                                .trie
                                .iter_prefix("")
                                .unwrap()
                                .filter_map(|(mut key, extension_trie)| {
                                    (key.file_name() == Some(TRIE_SUFFIX)).then(|| {
                                        key.pop();
                                        extension_trie
                                            .values()
                                            .map(move |&cactus_index| (key.clone(), cactus_index))
                                    })
                                })
                                .flatten()
                                .enumerate()
                                .filter(|(row, _)| rows.contains(row))
                            {
                                ui.add(
                                    egui::Label::new(format!(
                                        "{key} ➡ {}",
                                        cache.get_path_from_cactus_index(cactus_index),
                                    ))
                                    .truncate(),
                                );
                            }
                        },
                    );
            },
        );
    }

    /// Finds the correct letter casing and file extension for the given RPG Maker-style
    /// case-insensitive path.
    ///
    /// First this function will perform a case-insensitive search for the given path.
    ///
    /// If no file or folder at that path is found, this function searches a second time with a
    /// '.*' glob pattern appended to the end of the path (e.g. if you're looking for "my/path",
    /// this will also find stuff like "my/path.txt" or "my/path.json").
    ///
    /// If no match was found either time, returns `Err(NotExist)`.
    pub fn desensitize(&self, path: impl AsRef<camino::Utf8Path>) -> Result<camino::Utf8PathBuf> {
        let path = path.as_ref();
        let mut cache = self.cache.write();
        cache.regen(&self.fs, path)?;
        cache.desensitize(path).ok_or(Error::NotExist.into())
    }
}

pub fn to_lowercase(p: impl AsRef<camino::Utf8Path>) -> camino::Utf8PathBuf {
    p.as_ref()
        .as_str()
        .to_lowercase()
        .replace(std::path::MAIN_SEPARATOR, "/")
        .into()
}

fn with_trie_suffix(path: impl AsRef<camino::Utf8Path>) -> camino::Utf8PathBuf {
    let path = path.as_ref();
    if path.as_str().is_empty() {
        TRIE_SUFFIX.into()
    } else {
        format!("{path}/{TRIE_SUFFIX}").into()
    }
}

impl<F> FileSystemTrait for FileSystem<F>
where
    F: FileSystemTrait,
{
    type File = F::File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While opening file {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;

        if flags.contains(OpenFlags::Create) && cache.desensitize(path).is_none() {
            // If `OpenFlags::Create` was passed via `flags` and the given path doesn't exist in
            // the cache, then it must be the case that the path doesn't exist because we just
            // called `.regen` to attempt to insert the path into the cache a few lines ago. So we
            // need to create the file.

            // Use the path cache to get the desensitized version of the path to the parent
            // directory of the new file we need to create. If the parent directory doesn't exist
            // in the cache either then the parent directory doesn't exist yet, so error out with a
            // "does not exist" error because we don't recursively create parent directories.
            let parent_path = cache
                .desensitize(
                    path.parent()
                        .ok_or(Error::NotExist)
                        .wrap_err_with(|| c.clone())?,
                )
                .ok_or(Error::NotExist)
                .wrap_err_with(|| c.clone())?;

            // Create the file in the parent directory with the filename at the end of the original
            // path.
            let path = parent_path.join(path.file_name().unwrap());
            let file = self
                .fs
                .open_file(&path, flags)
                .wrap_err_with(|| c.clone())?;

            // Add the new file to the path cache.
            cache.regen(&self.fs, &path).wrap_err_with(|| c.clone())?;

            Ok(file)
        } else {
            self.fs
                .open_file(
                    cache
                        .desensitize(path)
                        .ok_or(Error::NotExist)
                        .wrap_err_with(|| c.clone())?,
                    flags,
                )
                .wrap_err_with(|| c.clone())
        }
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While getting metadata for {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;

        let path = cache
            .desensitize(path)
            .ok_or(Error::NotExist)
            .wrap_err_with(|| c.clone())?;
        self.fs.metadata(path).wrap_err_with(|| c.clone())
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let mut cache = self.cache.write();
        let c = format!(
            "While renaming {:?} to {:?} in a path cache",
            from.as_ref(),
            to.as_ref()
        );
        cache
            .regen(&self.fs, from.as_ref())
            .wrap_err_with(|| c.clone())?;
        let from = cache
            .desensitize(from)
            .ok_or(Error::NotExist)
            .wrap_err_with(|| c.clone())?;

        self.fs.rename(&from, to).wrap_err_with(|| c.clone())?;

        {
            let cache = &mut *cache;
            for extension_trie in cache.trie.iter_prefix(&from).unwrap().map(|(_, t)| t) {
                for index in extension_trie.values().copied() {
                    cache.cactus.remove(index);
                }
            }
            cache.trie.remove_dir(&from);
        }

        Ok(())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While checking if {path:?} exists in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;
        Ok(cache.desensitize(path).is_some())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While creating directory {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;

        let mut lower_path = to_lowercase(path);
        let extension = lower_path.extension().unwrap_or_default().to_string();
        lower_path.set_extension("");
        let prefix = cache.trie.get_dir_prefix(lower_path);
        let cactus_index = (!prefix.as_str().is_empty()).then(|| {
            let extension_trie = cache.trie.get_file(with_trie_suffix(prefix)).unwrap();
            *extension_trie
                .get_str(&extension)
                .unwrap_or(extension_trie.values().next().unwrap())
        });
        let original_prefix =
            cactus_index.map_or_else(Default::default, |i| cache.get_path_from_cactus_index(i));
        let len = original_prefix.iter().count();

        self.fs
            .create_dir(if len == 0 {
                path.to_path_buf()
            } else if len == path.iter().count() {
                original_prefix
            } else {
                std::iter::once(original_prefix.as_str())
                    .chain(path.iter().skip(len))
                    .join(std::path::MAIN_SEPARATOR_STR)
                    .into()
            })
            .wrap_err_with(|| c.clone())?;

        Ok(())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While removing directory {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;
        let path = cache
            .desensitize(path)
            .ok_or(Error::NotExist)
            .wrap_err_with(|| c.clone())?;

        self.fs.remove_dir(&path).wrap_err_with(|| c.clone())?;

        // Remove the directory and its contents from `cache.trie` and `cache.cactus`
        {
            let cache = &mut *cache;

            let mut path = to_lowercase(path);
            path.set_extension("");

            if let Some(iter) = cache.trie.iter_prefix(&path) {
                for extension_trie in iter.map(|(_, t)| t) {
                    for index in extension_trie.values().copied() {
                        cache.cactus.remove(index);
                    }
                }
                cache.trie.remove_dir(&path);
            }
        }

        Ok(())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While removing file {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;
        let path = cache
            .desensitize(path)
            .ok_or(Error::NotExist)
            .wrap_err_with(|| c.clone())?;

        self.fs.remove_file(&path).wrap_err_with(|| c.clone())?;

        // Remove the file from `cache.trie` and `cache.cactus`
        {
            let cache = &mut *cache;

            let mut path = to_lowercase(path);
            let extension = path.extension().unwrap_or_default().to_string();
            let path_clone = path.clone();
            path.set_extension("");

            // Remove by exact match
            if let Some(extension_trie) = cache.trie.get_file_mut(with_trie_suffix(&path)) {
                if let Some(&index) = extension_trie.get_str(&extension) {
                    cache.cactus.remove(index);
                    extension_trie.remove_str(&extension);
                    if extension_trie.is_empty() {
                        cache.trie.remove_dir(&path);
                    }
                    return Ok(());
                }
            }

            // Remove with added '.*' glob pattern
            let path = path_clone;
            if let Some(extension_trie) = cache.trie.get_file_mut(with_trie_suffix(&path)) {
                if let Some((key, &index)) = extension_trie.iter().next() {
                    let key = key.to_owned();
                    cache.cactus.remove(index);
                    extension_trie.remove(&key);
                }
                if extension_trie.is_empty() {
                    cache.trie.remove_dir(&path);
                }
            }
        }

        Ok(())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let mut cache = self.cache.write();
        let path = path.as_ref();
        let c = format!("While reading the contents of the directory {path:?} in a path cache");
        cache.regen(&self.fs, path).wrap_err_with(|| c.clone())?;
        let path = cache
            .desensitize(path)
            .ok_or(Error::NotExist)
            .wrap_err_with(|| c.clone())?;
        self.fs.read_dir(path).wrap_err_with(|| c.clone())
    }
}
