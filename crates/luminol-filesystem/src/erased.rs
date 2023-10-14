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
use crate::{DirEntry, Error, Metadata, OpenFlags};

pub trait File: std::io::Read + std::io::Write + std::io::Seek + Send + Sync {}
impl<T> File for T where T: std::io::Read + std::io::Write + std::io::Seek + Send + Sync {}

pub trait ErasedFilesystem: Send + Sync {
    fn open_file(
        &self,
        path: &camino::Utf8Path,
        flags: OpenFlags,
    ) -> Result<Box<dyn File + '_>, Error>;

    fn metadata(&self, path: &camino::Utf8Path) -> Result<Metadata, Error>;

    fn rename(&self, from: &camino::Utf8Path, to: &camino::Utf8Path) -> Result<(), Error>;

    fn exists(&self, path: &camino::Utf8Path) -> Result<bool, Error>;

    fn create_dir(&self, path: &camino::Utf8Path) -> Result<(), Error>;

    fn remove_dir(&self, path: &camino::Utf8Path) -> Result<(), Error>;

    fn remove_file(&self, path: &camino::Utf8Path) -> Result<(), Error>;

    fn remove(&self, path: &camino::Utf8Path) -> Result<(), Error>;

    fn read_dir(&self, path: &camino::Utf8Path) -> Result<Vec<DirEntry>, Error>;

    fn read(&self, path: &camino::Utf8Path) -> Result<Vec<u8>, Error>;

    fn read_to_string(&self, path: &camino::Utf8Path) -> Result<String, Error>;

    fn write(&self, path: &camino::Utf8Path, data: &[u8]) -> Result<(), Error>;
}

impl<T> ErasedFilesystem for T
where
    T: crate::FileSystem,
{
    fn open_file(
        &self,
        path: &camino::Utf8Path,
        flags: OpenFlags,
    ) -> Result<Box<dyn File + '_>, Error> {
        let file = self.open_file(path, flags)?;
        Ok(Box::new(file))
    }

    fn metadata(&self, path: &camino::Utf8Path) -> Result<Metadata, Error> {
        self.metadata(path)
    }

    fn rename(&self, from: &camino::Utf8Path, to: &camino::Utf8Path) -> Result<(), Error> {
        self.rename(from, to)
    }

    fn exists(&self, path: &camino::Utf8Path) -> Result<bool, Error> {
        self.exists(path)
    }

    fn create_dir(&self, path: &camino::Utf8Path) -> Result<(), Error> {
        self.create_dir(path)
    }

    fn remove_dir(&self, path: &camino::Utf8Path) -> Result<(), Error> {
        self.remove_dir(path)
    }

    fn remove_file(&self, path: &camino::Utf8Path) -> Result<(), Error> {
        self.remove_file(path)
    }

    fn remove(&self, path: &camino::Utf8Path) -> Result<(), Error> {
        self.remove(path)
    }

    fn read_dir(&self, path: &camino::Utf8Path) -> Result<Vec<DirEntry>, Error> {
        self.read_dir(path)
    }

    fn read(&self, path: &camino::Utf8Path) -> Result<Vec<u8>, Error> {
        self.read(path)
    }

    fn read_to_string(&self, path: &camino::Utf8Path) -> Result<String, Error> {
        self.read_to_string(path)
    }

    fn write(&self, path: &camino::Utf8Path, data: &[u8]) -> Result<(), Error> {
        self.write(path, data)
    }
}

impl crate::FileSystem for dyn ErasedFilesystem {
    type File<'fs> = Box<dyn File + 'fs> where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        self.open_file(path.as_ref(), flags)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        self.metadata(path.as_ref())
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<(), Error> {
        self.rename(from.as_ref(), to.as_ref())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        self.exists(path.as_ref())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.create_dir(path.as_ref())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.remove_dir(path.as_ref())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.remove_file(path.as_ref())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        self.read_dir(path.as_ref())
    }

    fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.remove(path.as_ref())
    }

    fn read(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<u8>, Error> {
        self.read(path.as_ref())
    }

    fn read_to_string(&self, path: impl AsRef<camino::Utf8Path>) -> Result<String, Error> {
        self.read_to_string(path.as_ref())
    }

    fn write(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), Error> {
        self.write(path.as_ref(), data.as_ref())
    }
}
