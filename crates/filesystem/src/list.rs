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

use crate::{erased::ErasedFilesystem, DirEntry, Error, File, Metadata, OpenFlags, Result};
use color_eyre::eyre::WrapErr;
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
        let c = format!("While opening file {path:?} in a list filesystem");
        let parent = path.parent().unwrap_or(path);
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())?
                || (flags.contains(OpenFlags::Create)
                    && fs.exists(parent).wrap_err_with(|| c.clone())?)
            {
                return fs.open_file(path, flags).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let path = path.as_ref();
        let c = format!("While getting metadata for {path:?} in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                return fs.metadata(path).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        let c = format!("While renaming {from:?} to {to:?} in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(from).wrap_err_with(|| c.clone())? {
                return fs.rename(from, to.as_ref()).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        let path = path.as_ref();
        let c = format!("While checking if {path:?} exists in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While creating a directory at {path:?} in a list filesystem");
        let fs = self
            .filesystems
            .first()
            .ok_or(Error::NoFilesystems)
            .wrap_err_with(|| c.clone())?;
        fs.create_dir(path).wrap_err_with(|| c.clone())
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While removing a directory at {path:?} in a list filesystem");
        let fs = self
            .filesystems
            .first()
            .ok_or(Error::NoFilesystems)
            .wrap_err_with(|| c.clone())?;
        fs.remove_dir(path).wrap_err_with(|| c.clone())
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While removing a file at {path:?} in a list filesystem");
        let fs = self
            .filesystems
            .first()
            .ok_or(Error::NoFilesystems)
            .wrap_err_with(|| c.clone())?;
        fs.remove_file(path).wrap_err_with(|| c.clone())
    }

    fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While removing {path:?} in a list filesystem");
        let fs = self
            .filesystems
            .first()
            .ok_or(Error::NoFilesystems)
            .wrap_err_with(|| c.clone())?;
        fs.remove(path).wrap_err_with(|| c.clone())
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let path = path.as_ref();
        let c =
            format!("While reading the contents of the directory {path:?} in a list filesystem");

        let mut entries = Vec::new();
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                entries.extend(fs.read_dir(path).wrap_err_with(|| c.clone())?)
            }
        }
        // FIXME: remove duplicates in a more efficient manner
        let entries = entries.into_iter().unique().collect_vec();

        Ok(entries)
    }

    fn read(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<u8>> {
        let path = path.as_ref();
        let c = format!("While reading from the file {path:?} in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                return fs.read(path).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }

    fn read_to_string(&self, path: impl AsRef<camino::Utf8Path>) -> Result<String> {
        let path = path.as_ref();
        let c = format!("While reading from the file {path:?} in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                return fs.read_to_string(path).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }

    fn write(&self, path: impl AsRef<camino::Utf8Path>, data: impl AsRef<[u8]>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While writing to the file {path:?} in a list filesystem");
        for fs in self.filesystems.iter() {
            if fs.exists(path).wrap_err_with(|| c.clone())? {
                return fs.write(path, data.as_ref()).wrap_err_with(|| c.clone());
            }
        }
        Err(Error::NotExist).wrap_err_with(|| c.clone())
    }
}
