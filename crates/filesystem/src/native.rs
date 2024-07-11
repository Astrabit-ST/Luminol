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

use color_eyre::eyre::WrapErr;
use itertools::Itertools;

use crate::{DirEntry, Metadata, OpenFlags, Result, StdIoErrorExt};
use pin_project::pin_project;
use std::{
    io::ErrorKind::{InvalidInput, PermissionDenied},
    pin::Pin,
    task::Poll,
};

#[derive(Debug, Clone)]
pub struct FileSystem {
    root_path: camino::Utf8PathBuf,
}

#[derive(Debug)]
#[pin_project]
pub struct File {
    file: Inner,
    path: camino::Utf8PathBuf,
    stripped_path: Option<camino::Utf8PathBuf>,
    #[pin]
    async_file: async_fs::File,
}

#[derive(Debug)]
enum Inner {
    StdFsFile(std::fs::File),
    NamedTempFile(tempfile::NamedTempFile),
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

    pub async fn from_folder_picker() -> Result<Self> {
        let c = "While picking a folder from the host filesystem";
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path = camino::Utf8Path::from_path(path.path())
                .ok_or(crate::Error::PathUtf8Error)
                .wrap_err(c)?;
            Ok(Self::new(path))
        } else {
            Err(crate::Error::CancelledLoading).wrap_err(c)
        }
    }

    pub async fn from_file_picker() -> Result<Self> {
        let c = "While picking a folder from the host filesystem";
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "rvproj", "rvproj2", "lumproj"])
            .pick_file()
            .await
        {
            let path = camino::Utf8Path::from_path(path.path())
                .ok_or(crate::Error::PathUtf8Error)
                .wrap_err(c)?
                .parent()
                .expect("path does not have parent");
            Ok(Self::new(path))
        } else {
            Err(crate::Error::CancelledLoading).wrap_err(c)
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
        let stripped_path = path.as_ref();
        let path = self.root_path.join(path.as_ref());
        let c = format!("While opening file {path:?} in a host folder");
        std::fs::OpenOptions::new()
            .create(flags.contains(OpenFlags::Create))
            .write(flags.contains(OpenFlags::Write))
            .read(flags.contains(OpenFlags::Read))
            .truncate(flags.contains(OpenFlags::Truncate))
            .open(&path)
            .map(|file| {
                let clone = file.try_clone().wrap_err_with(|| c.clone())?;
                Ok(File {
                    file: Inner::StdFsFile(file),
                    path,
                    stripped_path: Some(stripped_path.to_owned()),
                    async_file: clone.into(),
                })
            })
            .wrap_err_with(|| c.clone())?
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let c = format!(
            "While getting metadata for {:?} in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        let metadata = std::fs::metadata(path).wrap_err_with(|| c.clone())?;
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
        let c = format!(
            "While renaming {:?} to {:?} in a host folder",
            from.as_ref(),
            to.as_ref()
        );
        let from = self.root_path.join(from);
        let to = self.root_path.join(to);
        std::fs::rename(from, to).wrap_err(c)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        let c = format!(
            "While checking if {:?} exists in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        path.try_exists().wrap_err(c)
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let c = format!(
            "While creating a directory at {:?} in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        std::fs::create_dir_all(path).wrap_err(c)
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let c = format!(
            "While removing a directory at {:?} in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        std::fs::remove_dir_all(path).wrap_err(c)
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let c = format!(
            "While removing a file at {:?} in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        std::fs::remove_file(path).wrap_err(c)
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let c = format!(
            "While reading the contents of the directory {:?} in a host folder",
            path.as_ref()
        );
        let path = self.root_path.join(path);
        path.read_dir_utf8()
            .wrap_err_with(|| c.clone())?
            .map_ok(|entry| {
                let path = entry.into_path();
                let path = path
                    .strip_prefix(&self.root_path)
                    .unwrap_or(&path)
                    .to_path_buf();

                // i hate windows.
                #[cfg(windows)]
                let path = path.into_string().replace('\\', "/").into();

                let metadata = self.metadata(&path).wrap_err_with(|| c.clone())?;
                Ok(DirEntry::new(path, metadata))
            })
            .flatten()
            .try_collect()
    }
}

impl File {
    /// Creates a new empty temporary file with read-write permissions.
    pub fn new() -> std::io::Result<Self> {
        let c = "While creating a temporary file on the host filesystem";
        let file = tempfile::NamedTempFile::new()?;
        let path = file
            .path()
            .to_str()
            .ok_or(std::io::Error::new(
                InvalidInput,
                "Tried to create a temporary file, but its path was not valid UTF-8",
            ))
            .wrap_io_err(c)?
            .into();
        let clone = file.as_file().try_clone().wrap_io_err(c)?;
        Ok(Self {
            file: Inner::NamedTempFile(file),
            path,
            stripped_path: None,
            async_file: clone.into(),
        })
    }

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
        let c = "While picking a file on the host filesystem";
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter(filter_name, extensions)
            .pick_file()
            .await
        {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .open(path.path())
                .map_err(crate::Error::IoError)
                .wrap_err(c)?;
            let path = path
                .path()
                .iter()
                .last()
                .unwrap()
                .to_os_string()
                .into_string()
                .map_err(|_| crate::Error::PathUtf8Error)
                .wrap_err(c)?;
            let clone = file.try_clone().wrap_err(c)?;
            Ok((
                File {
                    file: Inner::StdFsFile(file),
                    path: path.clone().into(),
                    stripped_path: Some(
                        camino::Utf8Path::new(&path)
                            .iter()
                            .next_back()
                            .unwrap()
                            .into(),
                    ),
                    async_file: clone.into(),
                },
                path,
            ))
        } else {
            Err(crate::Error::CancelledLoading).wrap_err(c)
        }
    }

    /// Saves this file to a location of the user's choice.
    ///
    /// In native, this will open a file picker dialog, wait for the user to choose a location to
    /// save a file, and then copy this file to the new location. This function will wait for the
    /// user to finish picking a file location before returning.
    ///
    /// In web, this will use the browser's native file downloading method to save the file, which
    /// may or may not open a file picker. Due to platform limitations, this function will return
    /// immediately after making a download request and will not wait for the user to pick a file
    /// location if a file picker is shown.
    ///
    /// You must flush the file yourself before saving. It will not be flushed for you.
    ///
    /// `filename` should be the default filename, with extension, to show in the file picker if
    /// one is shown. `filter_name` should be the name of the file type shown in the part of the
    /// file picker where the user selects a file extension. `filter_name` works only in native
    /// builds; it is ignored in web builds.
    pub async fn save(&self, filename: &str, filter_name: &str) -> Result<()> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While saving the file {:?} in a host folder to disk",
            stripped_path
        );
        let mut dialog = rfd::AsyncFileDialog::default().set_file_name(filename);
        if let Some((_, extension)) = filename.rsplit_once('.') {
            dialog = dialog.add_filter(filter_name, &[extension]);
        }
        let path = dialog
            .save_file()
            .await
            .ok_or(crate::Error::CancelledLoading)
            .wrap_err_with(|| c.clone())?;
        std::fs::copy(&self.path, path.path()).wrap_err_with(|| c.clone())?;
        Ok(())
    }
}

impl crate::File for File {
    fn metadata(&self) -> std::io::Result<Metadata> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While getting metadata for file {:?} in a host folder",
            stripped_path
        );
        let metdata = self.file.as_file().metadata().wrap_io_err(c)?;
        Ok(Metadata {
            is_file: metdata.is_file(),
            size: metdata.len(),
        })
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While setting length of file {:?} in a host folder",
            stripped_path
        );
        self.file.as_file().set_len(new_size).wrap_io_err(c)
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While reading from file {:?} in a host folder",
            stripped_path
        );
        self.file.as_file().read(buf).wrap_io_err(c)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> std::io::Result<usize> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While reading (vectored) from file {:?} in a host folder",
            stripped_path
        );
        self.file.as_file().read_vectored(bufs).wrap_io_err(c)
    }
}

impl futures_lite::AsyncRead for File {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously reading from file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_read(cx, buf)
            .map(|p| p.wrap_io_err(c))
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &mut [std::io::IoSliceMut<'_>],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously reading (vectored) from file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_read_vectored(cx, bufs)
            .map(|p| p.wrap_io_err(c))
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While writing to file {:?} in a host folder", stripped_path);
        self.file.as_file().write(buf).wrap_io_err(c)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While flushing file {:?} in a host folder", stripped_path);
        self.file.as_file().flush().wrap_io_err(c)
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While writing (vectored) to file {:?} in a host folder",
            stripped_path
        );
        self.file.as_file().write_vectored(bufs).wrap_io_err(c)
    }
}

impl futures_lite::AsyncWrite for File {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously writing to file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_write(cx, buf)
            .map(|r| r.wrap_io_err(c))
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously writing (vectored) to file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_write_vectored(cx, bufs)
            .map(|r| r.wrap_io_err(c))
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously flushing file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_flush(cx)
            .map(|r| r.wrap_io_err(c))
    }

    fn poll_close(
        self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously closing file {:?} in a host folder",
            stripped_path
        );
        Poll::Ready(Err(std::io::Error::new(PermissionDenied, "Attempted to asynchronously close a `luminol_filesystem::host::File`, which is not allowed")).wrap_io_err(c))
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While seeking file {:?} in a host folder", stripped_path);
        self.file.as_file().seek(pos).wrap_io_err(c)
    }
}

impl futures_lite::AsyncSeek for File {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: std::io::SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        let stripped_path = self
            .stripped_path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously seeking file {:?} in a host folder",
            stripped_path
        );
        self.project()
            .async_file
            .poll_seek(cx, pos)
            .map(|r| r.wrap_io_err(c))
    }
}

impl Inner {
    fn as_file(&self) -> &std::fs::File {
        match self {
            Inner::StdFsFile(file) => file,
            Inner::NamedTempFile(file) => file.as_file(),
        }
    }
}
