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
use super::{DirEntry, Error, FileSystem, Metadata, OpenFlags};
use itertools::Itertools;
use std::io::prelude::*;

#[derive(Debug, Clone)]
pub struct Overlay<P, S> {
    primary: P,
    secondary: S,
}

impl<P, S> Overlay<P, S> {
    pub fn new(primary: P, secondary: S) -> Self {
        Self { primary, secondary }
    }

    pub fn primary(&self) -> &P {
        &self.primary
    }

    pub fn secondary(&self) -> &S {
        &self.secondary
    }

    pub fn swap(self) -> Overlay<S, P> {
        Overlay {
            secondary: self.primary,
            primary: self.secondary,
        }
    }
}

#[derive(Debug)]
pub enum File<'fs, P, S>
where
    P: FileSystem + 'fs,
    S: FileSystem + 'fs,
{
    Primary(P::File<'fs>),
    Secondary(S::File<'fs>),
}

impl<'fs, P, S> Write for File<'fs, P, S>
where
    P: FileSystem,
    S: FileSystem,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Primary(f) => f.write(buf),
            Self::Secondary(f) => f.write(buf),
        }
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        match self {
            Self::Primary(f) => f.write_vectored(bufs),
            Self::Secondary(f) => f.write_vectored(bufs),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Primary(f) => f.flush(),
            Self::Secondary(f) => f.flush(),
        }
    }
}

impl<'fs, P, S> Read for File<'fs, P, S>
where
    P: FileSystem,
    S: FileSystem,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Primary(f) => f.read(buf),
            Self::Secondary(f) => f.read(buf),
        }
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        match self {
            Self::Primary(f) => f.read_vectored(bufs),
            Self::Secondary(f) => f.read_vectored(bufs),
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> std::io::Result<()> {
        match self {
            Self::Primary(f) => f.read_exact(buf),
            Self::Secondary(f) => f.read_exact(buf),
        }
    }
}

impl<'fs, P, S> Seek for File<'fs, P, S>
where
    P: FileSystem,
    S: FileSystem,
{
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        match self {
            Self::Primary(f) => f.seek(pos),
            Self::Secondary(f) => f.seek(pos),
        }
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        match self {
            Self::Primary(f) => f.stream_position(),
            Self::Secondary(f) => f.stream_position(),
        }
    }
}

impl<P, S> FileSystem for Overlay<P, S>
where
    P: FileSystem,
    S: FileSystem,
{
    type File<'fs> = File<'fs, P, S> where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        let path = path.as_ref();
        if self.primary.exists(path)? || flags.contains(OpenFlags::Create) {
            return self.primary.open_file(path, flags).map(File::Primary);
        }

        if self.secondary.exists(path)? {
            return self.secondary.open_file(path, flags).map(File::Secondary);
        }

        self.primary.open_file(path, flags).map(File::Primary)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        let path = path.as_ref();
        if self.primary.exists(path)? {
            self.primary.metadata(path)
        } else {
            self.secondary.metadata(path)
        }
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<(), Error> {
        self.primary.rename(from, to)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        let path = path.as_ref();
        if self.primary.exists(path)? {
            Ok(true)
        } else {
            self.secondary.exists(path)
        }
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.primary.create_dir(path)
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.primary.remove_dir(path)
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        self.primary.remove_file(path)
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let path = path.as_ref();

        let mut entries = vec![]; // FIXME: inefficient
        if self.primary.exists(path)? {
            entries.extend(self.primary.read_dir(path)?);
        }
        if self.secondary.exists(path)? {
            entries.extend(self.secondary.read_dir(path)?);
        }
        // FIXME: remove duplicates in a more efficient manner
        let entries = entries.into_iter().unique().collect_vec();

        Ok(entries)
    }
}
