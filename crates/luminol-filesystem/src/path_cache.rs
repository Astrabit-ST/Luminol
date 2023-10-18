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

use crate::{DirEntry, Error, Metadata, OpenFlags, Result};

#[derive(Debug, Clone)]
pub struct FileSystem<F> {
    fs: F,
    cache: dashmap::DashMap<camino::Utf8PathBuf, camino::Utf8PathBuf>,
}

impl<F> FileSystem<F>
where
    F: crate::FileSystem,
{
    pub fn new(fs: F) -> Result<Self> {
        let this = FileSystem {
            fs,
            cache: dashmap::DashMap::new(),
        };
        this.regen_cache()?;
        Ok(this)
    }

    pub fn fs(&self) -> &F {
        &self.fs
    }

    pub fn regen_cache(&self) -> Result<()> {
        fn read_dir_recursive(
            fs: &(impl crate::FileSystem + ?Sized),
            path: impl AsRef<camino::Utf8Path>,
            mut f: impl FnMut(&camino::Utf8Path),
        ) -> Result<()> {
            fn internal(
                fs: &(impl crate::FileSystem + ?Sized),
                path: impl AsRef<camino::Utf8Path>,
                f: &mut impl FnMut(&camino::Utf8Path),
            ) -> Result<()> {
                // In web builds, RTPs are currently to be placed in the "RTP" subdirectory of
                // the project root directory, so this is to avoid loading the contents of
                // those directories twice
                let skip = matches!(path.as_ref().iter().next_back(), Some("RTP"));

                for entry in fs.read_dir(path)? {
                    f(entry.path());
                    if !skip && !entry.metadata().is_file {
                        internal(fs, entry.path(), f)?;
                    }
                }
                Ok(())
            }
            internal(fs, path, &mut f)
        }

        self.cache.clear();
        read_dir_recursive(&self.fs, "", |path| {
            let mut lowercase = to_lowercase(path);
            lowercase.set_extension("");

            self.cache.insert(lowercase, path.to_path_buf());
        })?;
        Ok(())
    }

    pub fn debug_ui(&self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical()
            .id_source("luminol_path_cache_debug_ui")
            .show_rows(
                ui,
                ui.text_style_height(&egui::TextStyle::Body),
                self.cache.len(),
                |ui, rows| {
                    for (_, item) in self
                        .cache
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| rows.contains(index))
                    {
                        ui.horizontal(|ui| {
                            ui.label(item.key().as_str());
                            ui.label("➡");
                            ui.label(item.value().as_str());
                        });
                    }
                },
            );
    }

    pub fn desensitize(&self, path: impl AsRef<camino::Utf8Path>) -> Option<camino::Utf8PathBuf> {
        let mut path = to_lowercase(path);
        path.set_extension("");
        self.cache.get(&path).as_deref().cloned()
    }
}

pub fn to_lowercase(p: impl AsRef<camino::Utf8Path>) -> camino::Utf8PathBuf {
    p.as_ref().as_str().to_lowercase().into()
}

impl<F> crate::FileSystem for FileSystem<F>
where
    F: crate::FileSystem,
{
    type File = F::File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        let path = path.as_ref();
        if flags.contains(OpenFlags::Create) {
            let mut lower_path = to_lowercase(path);
            lower_path.set_extension("");
            self.cache.insert(lower_path, path.to_path_buf());
        }
        let path = self.desensitize(path).ok_or(Error::NotExist)?;
        self.fs.open_file(path, flags)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let path = self.desensitize(path).ok_or(Error::NotExist)?;
        self.fs.metadata(path)
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let from = self.desensitize(from).ok_or(Error::NotExist)?;
        let to = to.as_ref().to_path_buf();

        self.fs.rename(&from, &to)?;

        self.cache.remove(&from);
        self.cache.insert(to_lowercase(&to), to);

        Ok(())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        Ok(self.desensitize(path).is_some())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref().to_path_buf();

        self.fs.create_dir(&path)?;

        self.cache.insert(to_lowercase(&path), path);

        Ok(())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = self.desensitize(path).ok_or(Error::NotExist)?;

        self.fs.remove_dir(&path)?;

        self.cache.remove(&to_lowercase(path));

        Ok(())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = self.desensitize(path).ok_or(Error::NotExist)?;

        self.fs.remove_file(&path)?;

        self.cache.remove(&to_lowercase(path));

        Ok(())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let path = self.desensitize(path).ok_or(Error::NotExist)?;
        self.fs.read_dir(path)
    }
}
