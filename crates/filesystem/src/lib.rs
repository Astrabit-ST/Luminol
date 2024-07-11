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

pub mod archiver;
pub mod egui_bytes_loader;
pub mod erased;
pub mod list;
pub mod path_cache;
pub mod project;

mod trie;
pub use trie::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod web;

// Re-export platform specific filesystem as "host"
// This means we need can use less #[cfg]s!
#[cfg(not(target_arch = "wasm32"))]
pub use native as host;
#[cfg(target_arch = "wasm32")]
pub use web as host;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("File or directory does not exist")]
    NotExist,
    #[error("Io error {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 Error {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),
    #[error("Path is not valid UTF-8")]
    PathUtf8Error,
    #[error("Project not loaded")]
    NotLoaded,
    #[error("Operation not supported by this filesystem")]
    NotSupported,
    #[error("Archive header is incorrect")]
    InvalidHeader,
    #[error("Invalid archive version: {0} (supported versions are 1, 2 and 3)")]
    InvalidArchiveVersion(u8),
    #[error("No filesystems are loaded to perform this operation")]
    NoFilesystems,
    #[error("Unable to detect the project's RPG Maker version (perhaps you did not open an RPG Maker project?")]
    UnableToDetectRMVer,
    #[error("Cancelled loading project")]
    CancelledLoading,
    #[error("Your browser does not support File System Access API")]
    Wasm32FilesystemNotSupported,
    #[error("Invalid project folder")]
    InvalidProjectFolder,
}

pub use color_eyre::Result;

pub trait StdIoErrorExt {
    // Add additional context to a `std::io::Result`.
    fn wrap_io_err_with<C>(self, c: impl FnOnce() -> C) -> Self
    where
        Self: Sized,
        C: std::fmt::Display + Send + Sync + 'static;

    // Add additional context to a `std::io::Result`.
    fn wrap_io_err<C>(self, c: C) -> Self
    where
        Self: Sized,
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.wrap_io_err_with(|| c)
    }

    // Add additional context to a `std::io::Result`. This is an alias for `.wrap_io_err_with`.
    fn with_io_context<C>(self, c: impl FnOnce() -> C) -> Self
    where
        Self: Sized,
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.wrap_io_err_with(c)
    }

    // Add additional context to a `std::io::Result`. This is an alias for `.wrap_io_err`.
    fn io_context<C>(self, c: C) -> Self
    where
        Self: Sized,
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.wrap_io_err(c)
    }
}

impl<T> StdIoErrorExt for std::io::Result<T> {
    fn wrap_io_err_with<C>(self, c: impl FnOnce() -> C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        self.map_err(|e| std::io::Error::new(e.kind(), color_eyre::eyre::eyre!(e).wrap_err(c())))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Metadata {
    pub is_file: bool,
    pub size: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DirEntry {
    pub path: camino::Utf8PathBuf,
    pub metadata: Metadata,
}

impl DirEntry {
    pub fn new(path: camino::Utf8PathBuf, metadata: Metadata) -> Self {
        Self { path, metadata }
    }

    pub fn path(&self) -> &camino::Utf8Path {
        &self.path
    }

    pub fn metadata(&self) -> Metadata {
        self.metadata
    }

    pub fn file_name(&self) -> &str {
        self.path
            .file_name()
            .expect("path created through DirEntry must have a filename")
    }

    pub fn into_path(self) -> camino::Utf8PathBuf {
        self.path
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct OpenFlags: u8 {
        const Read = 0b00000001;
        const Write = 0b00000010;
        const Truncate = 0b00000100;
        const Create = 0b00001000;
    }
}

pub trait File: std::io::Read + std::io::Write + std::io::Seek + Send + Sync {
    fn metadata(&self) -> std::io::Result<Metadata>;

    /// Truncates or extends the size of the file. If the file is extended, the file will be
    /// null-padded at the end. The file cursor never changes when truncating or extending, even if
    /// the cursor would be put outside the file bounds by this operation.
    fn set_len(&self, new_size: u64) -> std::io::Result<()>;

    /// Casts a mutable reference to this file into `&mut luminol_filesystem::File`.
    fn as_file(&mut self) -> &mut Self
    where
        Self: Sized,
    {
        self
    }
}

impl<T> File for &mut T
where
    T: File + ?Sized,
{
    fn metadata(&self) -> std::io::Result<Metadata> {
        (**self).metadata()
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        (**self).set_len(new_size)
    }
}

pub trait FileSystem: Send + Sync {
    type File: File;

    fn open_file(&self, path: impl AsRef<camino::Utf8Path>, flags: OpenFlags)
        -> Result<Self::File>;

    fn create_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Self::File> {
        self.open_file(path, OpenFlags::Create | OpenFlags::Write)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata>;

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()>;

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool>;

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()>;

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()>;

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()>;

    fn remove(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let metadata = self.metadata(path)?;
        if metadata.is_file {
            self.remove_file(path)
        } else {
            self.remove_dir(path)
        }
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>>;

    /// Corresponds to [`std::fs::read()`].
    /// Will open a file at the path and read the entire file into a buffer.
    fn read(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<u8>> {
        use std::io::Read;

        let path = path.as_ref();

        let mut buf = Vec::with_capacity(self.metadata(path)?.size as usize);
        let mut file = self.open_file(path, OpenFlags::Read)?;
        file.read_to_end(&mut buf)?;

        Ok(buf)
    }

    fn read_to_string(&self, path: impl AsRef<camino::Utf8Path>) -> Result<String> {
        let buf = self.read(path)?;
        String::from_utf8(buf).map_err(Into::into)
    }

    /// Corresponds to [`std::fs::write()`].
    /// Will open a file at the path, create it if it exists (and truncate it) and then write the provided bytes.
    fn write(&self, path: impl AsRef<camino::Utf8Path>, data: impl AsRef<[u8]>) -> Result<()> {
        use std::io::Write;

        let mut file = self.open_file(
            path,
            OpenFlags::Write | OpenFlags::Truncate | OpenFlags::Create,
        )?;
        file.write_all(data.as_ref())?;
        file.flush()?;

        Ok(())
    }
}

pub trait ReadDir {
    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>>;
}

impl<T> ReadDir for T
where
    T: FileSystem,
{
    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        self.read_dir(path)
    }
}
