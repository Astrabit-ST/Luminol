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
use crate::prelude::*;

#[derive(Debug)]
pub struct Archiver {}

impl Archiver {
    pub fn new(project_path: impl AsRef<Path>) -> Self {
        Archiver {}
    }
}

impl vfs::FileSystem for Archiver {
    fn read_dir(&self, path: &str) -> vfs::VfsResult<Box<dyn Iterator<Item = String> + Send>> {
        todo!()
    }

    fn create_dir(&self, path: &str) -> vfs::VfsResult<()> {
        todo!()
    }

    fn open_file(&self, path: &str) -> vfs::VfsResult<Box<dyn vfs::SeekAndRead + Send>> {
        todo!()
    }

    fn create_file(&self, path: &str) -> vfs::VfsResult<Box<dyn std::io::Write + Send>> {
        todo!()
    }

    fn append_file(&self, path: &str) -> vfs::VfsResult<Box<dyn std::io::Write + Send>> {
        todo!()
    }

    fn metadata(&self, path: &str) -> vfs::VfsResult<vfs::VfsMetadata> {
        todo!()
    }

    fn exists(&self, path: &str) -> vfs::VfsResult<bool> {
        todo!()
    }

    fn remove_file(&self, path: &str) -> vfs::VfsResult<()> {
        todo!()
    }

    fn remove_dir(&self, path: &str) -> vfs::VfsResult<()> {
        todo!()
    }
}
