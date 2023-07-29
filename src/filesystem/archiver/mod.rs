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
#![allow(clippy::upper_case_acronyms)]
use super::{DirEntry, Error, FileSystem, Metadata, OpenFlags};
use crate::prelude::*;

mod rgss3a;
mod rgssad;

#[derive(Debug)]
pub enum Archiver {
    RGSSAD(rgssad::Archiver),
    // RGSS3A(rgss3a::Archiver),
}

impl Archiver {
    pub fn new(
        editor_ver: config::RMVer,
        project_path: impl AsRef<camino::Utf8Path>,
    ) -> Result<Self, Error> {
        Ok(match editor_ver {
            config::RMVer::XP | config::RMVer::VX => {
                Archiver::RGSSAD(rgssad::Archiver::new(project_path)?)
            }
            _ => todo!(),
        })
    }
}

#[derive(Debug)]
pub enum File {
    RGSSAD(rgssad::File),
    // RGSS3A(rgss3a::File),
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::RGSSAD(f) => f.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self {
            Self::RGSSAD(f) => f.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            File::RGSSAD(f) => f.flush(),
        }
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::RGSSAD(f) => f.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        match self {
            Self::RGSSAD(f) => f.read_vectored(bufs),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            Self::RGSSAD(f) => f.read_exact(buf),
        }
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::RGSSAD(f) => f.seek(pos),
        }
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        match self {
            Self::RGSSAD(f) => f.stream_position(),
        }
    }
}

impl FileSystem for Archiver {
    type File<'fs> = File where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        match self {
            Archiver::RGSSAD(a) => a.open_file(path, flags).map(File::RGSSAD),
        }
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        match self {
            Archiver::RGSSAD(a) => a.metadata(path),
        }
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> std::result::Result<(), Error> {
        match self {
            Archiver::RGSSAD(a) => a.rename(from, to),
        }
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        match self {
            Archiver::RGSSAD(a) => a.exists(path),
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        match self {
            Archiver::RGSSAD(a) => a.create_dir(path),
        }
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        match self {
            Archiver::RGSSAD(a) => a.remove_dir(path),
        }
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        match self {
            Archiver::RGSSAD(a) => a.remove_file(path),
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        match self {
            Archiver::RGSSAD(a) => a.read_dir(path),
        }
    }
}
