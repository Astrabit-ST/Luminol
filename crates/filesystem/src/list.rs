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

use crate::{erased::ErasedFilesystem, DirEntry, Error, File, Metadata, OpenFlags, Result};
use itertools::Itertools;

#[derive(Default)]
pub struct FileSystem {
    filesystems: Vec<Box<dyn ErasedFilesystem>>,
}

impl FileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, fs: impl crate::FileSystem + 'static) {
        self.filesystems.push(Box::new(fs))
    }
}

impl crate::FileSystem for FileSystem {
    type File = Box<dyn File>;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        let path = path.as_ref();
        let parent = path.parent().unwrap_or(path);
        for fs in self.filesystems.iter() {
            if fs.exists(path)? || (flags.contains(OpenFlags::Create) && fs.exists(parent)?) {
                return fs.open_file(path, flags);
            }
        }
        Err(Error::NotExist.into())
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let path = path.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                return fs.metadata(path);
            }
        }
        Err(Error::NotExist.into())
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let from = from.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(from)? {
                return fs.rename(from, to.as_ref());
            }
        }
        Err(Error::NotExist.into())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        let path = path.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let fs = self.filesystems.first().ok_or(Error::NoFilesystems)?;
        fs.create_dir(path.as_ref())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let fs = self.filesystems.first().ok_or(Error::NoFilesystems)?;
        fs.remove_dir(path.as_ref())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let fs = self.filesystems.first().ok_or(Error::NoFilesystems)?;
        fs.remove_file(path.as_ref())
    }

    fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let fs = self.filesystems.first().ok_or(Error::NoFilesystems)?;
        fs.remove(path.as_ref())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let path = path.as_ref();

        let mut entries = Vec::new();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                entries.extend(fs.read_dir(path)?)
            }
        }
        // FIXME: remove duplicates in a more efficient manner
        let entries = entries.into_iter().unique().collect_vec();

        Ok(entries)
    }

    fn read(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<u8>> {
        let path = path.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                return fs.read(path);
            }
        }
        Err(Error::NotExist.into())
    }

    fn read_to_string(&self, path: impl AsRef<camino::Utf8Path>) -> Result<String> {
        let path = path.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                return fs.read_to_string(path);
            }
        }
        Err(Error::NotExist.into())
    }

    fn write(&self, path: impl AsRef<camino::Utf8Path>, data: impl AsRef<[u8]>) -> Result<()> {
        let path = path.as_ref();
        for fs in self.filesystems.iter() {
            if fs.exists(path)? {
                return fs.write(path, data.as_ref());
            }
        }
        Err(Error::NotExist.into())
    }
}
