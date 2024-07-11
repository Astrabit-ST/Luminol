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
use crate::File;
use crate::{DirEntry, Metadata, OpenFlags, Result};

pub trait ErasedFilesystem: Send + Sync {
    fn open_file(&self, path: &camino::Utf8Path, flags: OpenFlags) -> Result<Box<dyn File>>;

    fn metadata(&self, path: &camino::Utf8Path) -> Result<Metadata>;

    fn rename(&self, from: &camino::Utf8Path, to: &camino::Utf8Path) -> Result<()>;

    fn exists(&self, path: &camino::Utf8Path) -> Result<bool>;

    fn create_dir(&self, path: &camino::Utf8Path) -> Result<()>;

    fn remove_dir(&self, path: &camino::Utf8Path) -> Result<()>;

    fn remove_file(&self, path: &camino::Utf8Path) -> Result<()>;

    fn remove(&self, path: &camino::Utf8Path) -> Result<()>;

    fn read_dir(&self, path: &camino::Utf8Path) -> Result<Vec<DirEntry>>;

    fn read(&self, path: &camino::Utf8Path) -> Result<Vec<u8>>;

    fn read_to_string(&self, path: &camino::Utf8Path) -> Result<String>;

    fn write(&self, path: &camino::Utf8Path, data: &[u8]) -> Result<()>;
}

impl<T> ErasedFilesystem for T
where
    T: crate::FileSystem,
    T::File: 'static,
{
    fn open_file(&self, path: &camino::Utf8Path, flags: OpenFlags) -> Result<Box<dyn File>> {
        let file = self.open_file(path, flags)?;
        Ok(Box::new(file))
    }

    fn metadata(&self, path: &camino::Utf8Path) -> Result<Metadata> {
        self.metadata(path)
    }

    fn rename(&self, from: &camino::Utf8Path, to: &camino::Utf8Path) -> Result<()> {
        self.rename(from, to)
    }

    fn exists(&self, path: &camino::Utf8Path) -> Result<bool> {
        self.exists(path)
    }

    fn create_dir(&self, path: &camino::Utf8Path) -> Result<()> {
        self.create_dir(path)
    }

    fn remove_dir(&self, path: &camino::Utf8Path) -> Result<()> {
        self.remove_dir(path)
    }

    fn remove_file(&self, path: &camino::Utf8Path) -> Result<()> {
        self.remove_file(path)
    }

    fn remove(&self, path: &camino::Utf8Path) -> Result<()> {
        self.remove(path)
    }

    fn read_dir(&self, path: &camino::Utf8Path) -> Result<Vec<DirEntry>> {
        self.read_dir(path)
    }

    fn read(&self, path: &camino::Utf8Path) -> Result<Vec<u8>> {
        self.read(path)
    }

    fn read_to_string(&self, path: &camino::Utf8Path) -> Result<String> {
        self.read_to_string(path)
    }

    fn write(&self, path: &camino::Utf8Path, data: &[u8]) -> Result<()> {
        self.write(path, data)
    }
}

impl File for Box<dyn File> {
    fn metadata(&self) -> std::io::Result<Metadata> {
        self.as_ref().metadata()
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        self.as_ref().set_len(new_size)
    }
}

impl crate::FileSystem for dyn ErasedFilesystem {
    type File = Box<dyn File>;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        self.open_file(path.as_ref(), flags)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        self.metadata(path.as_ref())
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        self.rename(from.as_ref(), to.as_ref())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        self.exists(path.as_ref())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        self.create_dir(path.as_ref())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        self.remove_dir(path.as_ref())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        self.remove_file(path.as_ref())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        self.read_dir(path.as_ref())
    }

    fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        self.remove(path.as_ref())
    }

    fn read(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<u8>> {
        self.read(path.as_ref())
    }

    fn read_to_string(&self, path: impl AsRef<camino::Utf8Path>) -> Result<String> {
        self.read_to_string(path.as_ref())
    }

    fn write(&self, path: impl AsRef<camino::Utf8Path>, data: impl AsRef<[u8]>) -> Result<()> {
        self.write(path.as_ref(), data.as_ref())
    }
}
