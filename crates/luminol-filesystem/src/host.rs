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

use crate::{DirEntry, File, Metadata, OpenFlags, Result};

#[derive(Debug, Clone)]
pub struct FileSystem {
    root_path: camino::Utf8PathBuf,
}

impl FileSystem {
    pub fn new(root_path: impl AsRef<camino::Utf8Path>) -> Self {
        Self {
            root_path: root_path.as_ref().to_path_buf(),
        }
    }

    pub fn root_path(&self) -> &camino::Utf8Path {
        &self.root_path
    }

    pub async fn from_pile_picker() -> Result<Self> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "rvproj", "rvproj2", "lumproj"])
            .pick_file()
            .await
        {
            let path = camino::Utf8Path::from_path(path.path())
                .expect("path not utf-8")
                .parent()
                .expect("path does not have parent");
            Ok(Self::new(path))
        } else {
            Err(crate::Error::CancelledLoading)
        }
    }
}

impl crate::FileSystem for FileSystem {
    type File = std::fs::File;

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

impl File for std::fs::File {
    fn metadata(&self) -> Result<Metadata> {
        let metdata = self.metadata()?;
        Ok(Metadata {
            is_file: metdata.is_file(),
            size: metdata.len(),
        })
    }
}