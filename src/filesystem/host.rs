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
use super::{File, FileSystem};
use itertools::Itertools;
use std::fs::File as HostFile;

#[derive(Debug, Clone)]
pub struct HostFS {
    root_path: camino::Utf8PathBuf,
}

impl HostFS {
    pub fn root_path(&self) -> &camino::Utf8Path {
        &self.root_path
    }
}

impl FileSystem for HostFS {
    type File = HostFile;
    type Error = std::io::Error;

    fn open_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Self::File, Self::Error> {
        let path = self.root_path.join(path);
        std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Self::Error> {
        let path = self.root_path.join(path);
        path.try_exists()
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error> {
        let path = self.root_path.join(path);
        std::fs::create_dir(path)
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error> {
        let path = self.root_path.join(path);
        std::fs::remove_dir_all(path)
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error> {
        let path = self.root_path.join(path);
        std::fs::remove_file(path)
    }

    fn read_dir(
        &self,
        path: impl AsRef<camino::Utf8Path>,
    ) -> Result<Vec<camino::Utf8PathBuf>, Self::Error> {
        let path = self.root_path.join(path);
        path.read_dir_utf8()?
            .map(|e| e.map(|e| e.into_path()))
            .try_collect()
    }
}

impl File for HostFile {}
