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
use itertools::Itertools;

use crate::{DirEntry, Metadata, OpenFlags, Result};

#[derive(Debug, Clone)]
pub struct FileSystem {
    root_path: camino::Utf8PathBuf,
}

#[derive(Debug)]
pub struct File(std::fs::File);

impl FileSystem {
    pub fn new(root_path: impl AsRef<camino::Utf8Path>) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub fn root_path(&self) -> &camino::Utf8Path {
        &self.root_path
    }

    pub async fn from_folder_picker() -> Result<Self> {
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path =
                camino::Utf8Path::from_path(path.path()).ok_or(crate::Error::PathUtf8Error)?;
            Ok(Self::new(path))
        } else {
            Err(crate::Error::CancelledLoading)
        }
    }

    pub async fn from_file_picker() -> Result<Self> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "rvproj", "rvproj2", "lumproj"])
            .pick_file()
            .await
        {
            let path = camino::Utf8Path::from_path(path.path())
                .ok_or(crate::Error::PathUtf8Error)?
                .parent()
                .expect("path does not have parent");
            Ok(Self::new(path))
        } else {
            Err(crate::Error::CancelledLoading)
        }
    }
}

impl crate::FileSystem for FileSystem {
    type File = File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        let path = self.root_path.join(path);
        std::fs::OpenOptions::new()
            .create(flags.contains(OpenFlags::Create))
            .write(flags.contains(OpenFlags::Write))
            .read(flags.contains(OpenFlags::Read))
            .truncate(flags.contains(OpenFlags::Truncate))
            .open(path)
            .map_err(Into::into)
            .map(|f| File(f))
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let path = self.root_path.join(path);
        let metadata = std::fs::metadata(path)?;
        Ok(Metadata {
            is_file: metadata.is_file(),
            size: metadata.len(),
        })
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let from = self.root_path.join(from);
        let to = self.root_path.join(to);
        std::fs::rename(from, to).map_err(Into::into)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        let path = self.root_path.join(path);
        path.try_exists().map_err(Into::into)
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = self.root_path.join(path);
        std::fs::create_dir(path).map_err(Into::into)
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = self.root_path.join(path);
        std::fs::remove_dir_all(path).map_err(Into::into)
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = self.root_path.join(path);
        std::fs::remove_file(path).map_err(Into::into)
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let path = self.root_path.join(path);
        path.read_dir_utf8()?
            .map_ok(|entry| {
                let path = entry.into_path();
                let path = path
                    .strip_prefix(&self.root_path)
                    .unwrap_or(&path)
                    .to_path_buf();

                // i hate windows.
                #[cfg(windows)]
                let path = path.into_string().replace('\\', "/").into();

                let metadata = self.metadata(&path)?;
                Ok(DirEntry::new(path, metadata))
            })
            .flatten()
            .try_collect()
    }
}

impl File {
    /// Attempts to prompt the user to choose a file from their local machine.
    /// Then creates a `File` allowing read-write access to that directory if they chose one
    /// successfully, along with the name of the file including the extension.
    ///
    /// `extensions` should be a list of accepted file extensions for the file, without the leading
    /// `.`
    pub async fn from_file_picker(
        filter_name: &str,
        extensions: &[impl ToString],
    ) -> Result<(Self, String)> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter(filter_name, extensions)
            .pick_file()
            .await
        {
            let f = std::fs::OpenOptions::new()
                .read(true)
                .open(path.path())
                .map_err(|e| crate::Error::IoError(e))?;
            Ok((
                File(f),
                path.path()
                    .iter()
                    .last()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .map_err(|_| crate::Error::PathUtf8Error)?,
            ))
        } else {
            Err(crate::Error::CancelledLoading)
        }
    }
}

impl crate::File for File {
    fn metadata(&self) -> Result<Metadata> {
        let metdata = self.0.metadata()?;
        Ok(Metadata {
            is_file: metdata.is_file(),
            size: metdata.len(),
        })
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        self.0.read_vectored(bufs)
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.0.write_vectored(bufs)
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        self.0.seek(pos)
    }
}
