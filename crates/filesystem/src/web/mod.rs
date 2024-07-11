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

mod events;
mod util;
pub use events::setup_main_thread_hooks;

use crate::StdIoErrorExt;

use super::FileSystem as FileSystemTrait;
use super::{DirEntry, Error, Metadata, OpenFlags, Result};
use std::io::ErrorKind::PermissionDenied;
use std::task::Poll;
use util::{generate_key, send_and_await, send_and_recv, send_and_wake};

static WORKER_CHANNELS: once_cell::sync::OnceCell<WorkerChannels> =
    once_cell::sync::OnceCell::new();

#[derive(Debug)]
pub struct WorkerChannels {
    command_tx: flume::Sender<FileSystemCommand>,
}

impl WorkerChannels {
    fn send(&self, command: FileSystemCommand) {
        self.command_tx.send(command).unwrap();
    }
}

#[derive(Debug)]
pub struct MainChannels {
    command_rx: flume::Receiver<FileSystemCommand>,
}

/// Creates a new connected `(WorkerChannels, MainChannels)` pair for initializing filesystems.
pub fn channels() -> (WorkerChannels, MainChannels) {
    let (command_tx, command_rx) = flume::unbounded();
    (WorkerChannels { command_tx }, MainChannels { command_rx })
}

#[derive(Debug)]
pub struct FileSystem {
    key: usize,
    name: String,
    idb_key: Option<String>,
}

#[derive(Debug)]
pub struct File {
    key: usize,
    path: Option<camino::Utf8PathBuf>,
    temp_file_name: Option<String>,
    futures: parking_lot::Mutex<FileFutures>,
}

#[derive(Debug, Default)]
pub struct FileFutures {
    read: Option<oneshot::Receiver<std::io::Result<Vec<u8>>>>,
    write: Option<oneshot::Receiver<std::io::Result<()>>>,
    flush: Option<oneshot::Receiver<std::io::Result<()>>>,
    seek: Option<oneshot::Receiver<std::io::Result<u64>>>,
}

#[derive(Debug)]
enum FileSystemCommand {
    Supported(oneshot::Sender<bool>),
    DirEntryMetadata(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<Metadata>>,
    ),
    DirPicker(oneshot::Sender<Option<(usize, String)>>),
    DirFromIdb(String, oneshot::Sender<Result<(usize, String)>>),
    DirToIdb(usize, String, oneshot::Sender<bool>),
    DirSubdir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<(usize, String)>>,
    ),
    DirOpenFile(
        usize,
        camino::Utf8PathBuf,
        OpenFlags,
        oneshot::Sender<Result<usize>>,
    ),
    DirEntryExists(usize, camino::Utf8PathBuf, oneshot::Sender<bool>),
    DirCreateDir(usize, camino::Utf8PathBuf, oneshot::Sender<Result<()>>),
    DirRemoveDir(usize, camino::Utf8PathBuf, oneshot::Sender<Result<()>>),
    DirRemoveFile(usize, camino::Utf8PathBuf, oneshot::Sender<Result<()>>),
    DirReadDir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<Vec<DirEntry>>>,
    ),
    DirDrop(usize, oneshot::Sender<bool>),
    DirClone(usize, oneshot::Sender<usize>),
    FileCreateTemp(oneshot::Sender<std::io::Result<(usize, String)>>),
    FileSetLength(usize, u64, oneshot::Sender<std::io::Result<()>>),
    FilePicker(
        String,
        Vec<String>,
        oneshot::Sender<Option<(usize, String)>>,
    ),
    FileSave(usize, String, oneshot::Sender<Result<()>>),
    FileRead(usize, usize, oneshot::Sender<std::io::Result<Vec<u8>>>),
    FileWrite(usize, Vec<u8>, oneshot::Sender<std::io::Result<()>>),
    FileFlush(usize, oneshot::Sender<std::io::Result<()>>),
    FileSeek(
        usize,
        std::io::SeekFrom,
        oneshot::Sender<std::io::Result<u64>>,
    ),
    FileSize(usize, oneshot::Sender<std::io::Result<u64>>),
    FileDrop(usize, Option<String>, oneshot::Sender<bool>),
}

fn worker_channels_or_die() -> &'static WorkerChannels {
    WORKER_CHANNELS.get().expect("FileSystem worker channels have not been initialized! Please call `FileSystem::setup_worker_channels` before calling this function.")
}

impl FileSystem {
    /// Initializes the channels that we use to send filesystem commands to the main thread.
    /// This must be called before performing any filesystem operations.
    pub fn setup_worker_channels(worker_channels: WorkerChannels) {
        WORKER_CHANNELS
            .set(worker_channels)
            .expect("FileSystem worker channels cannot be initialized twice");
    }

    /// Returns whether or not the user's browser supports the JavaScript File System API.
    pub fn filesystem_supported() -> bool {
        send_and_recv(FileSystemCommand::Supported)
    }

    /// Attempts to prompt the user to choose a directory from their local machine using the
    /// JavaScript File System API.
    /// Then creates a `FileSystem` allowing read-write access to that directory if they chose one
    /// successfully.
    /// If the File System API is not supported, this always returns `None` without doing anything.
    pub async fn from_folder_picker() -> Result<Self> {
        let c = "While picking a folder from the host filesystem";
        if !Self::filesystem_supported() {
            return Err(Error::Wasm32FilesystemNotSupported).wrap_err(c);
        }
        send_and_await(FileSystemCommand::DirPicker)
            .await
            .map(|(key, name)| Self {
                key,
                name,
                idb_key: None,
            })
            .ok_or(Error::CancelledLoading)
            .wrap_err(c)
    }

    /// Attempts to restore a previously created `FileSystem` using its IndexedDB key returned by
    /// `.save_to_idb()`.
    pub async fn from_idb_key(idb_key: String) -> Result<Self> {
        let c = "While restoring a directory handle from IndexedDB";
        if !Self::filesystem_supported() {
            return Err(Error::Wasm32FilesystemNotSupported).wrap_err(c);
        }
        send_and_await(|tx| FileSystemCommand::DirFromIdb(idb_key.clone(), tx))
            .await
            .map(|(key, name)| FileSystem {
                key,
                name,
                idb_key: Some(idb_key),
            })
            .wrap_err(c)
    }

    /// Creates a new `FileSystem` from a subdirectory of this one.
    pub fn subdir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Self> {
        let path = path.as_ref();
        let c = format!("While getting a subdirectory {path:?} of a host folder");
        send_and_recv(|tx| FileSystemCommand::DirSubdir(self.key, path.to_path_buf(), tx))
            .map(|(key, name)| FileSystem {
                key,
                name,
                idb_key: None,
            })
            .wrap_err(c)
    }

    /// Stores this `FileSystem` to IndexedDB. If successful, consumes this `Filesystem` and
    /// returns the key needed to restore this `FileSystem` using `FileSystem::from_idb()`.
    /// Otherwise, returns ownership of this `FileSystem`.
    pub fn save_to_idb(mut self) -> std::result::Result<String, Self> {
        let idb_key_is_some = self.idb_key.is_some();
        let idb_key = self.idb_key.take().unwrap_or_else(generate_key);
        if send_and_recv(|tx| FileSystemCommand::DirToIdb(self.key, idb_key.clone(), tx)) {
            Ok(idb_key)
        } else {
            self.idb_key = idb_key_is_some.then_some(idb_key);
            Err(self)
        }
    }

    /// Returns a path consisting of a single element: the name of the root directory of this
    /// filesystem.
    pub fn root_path(&self) -> &camino::Utf8Path {
        self.name.as_str().into()
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        let _ = send_and_recv(|tx| FileSystemCommand::DirDrop(self.key, tx));
    }
}

impl Clone for FileSystem {
    fn clone(&self) -> Self {
        Self {
            key: send_and_recv(|tx| FileSystemCommand::DirClone(self.key, tx)),
            name: self.name.clone(),
            idb_key: None,
        }
    }
}

impl FileSystemTrait for FileSystem {
    type File = File;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File> {
        let path = path.as_ref();
        let c = format!("While opening file {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirOpenFile(self.key, path.to_path_buf(), flags, tx))
            .map(|key| File {
                key,
                path: Some(path.to_owned()),
                temp_file_name: None,
                futures: Default::default(),
            })
            .wrap_err(c)
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        let path = path.as_ref();
        let c = format!("While getting metadata for {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirEntryMetadata(self.key, path.to_path_buf(), tx))
            .wrap_err(c)
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();
        let c = format!("While renaming {from:?} to {to:?} in a host folder");
        Err(Error::NotSupported).wrap_err(c)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        Ok(send_and_recv(|tx| {
            FileSystemCommand::DirEntryExists(self.key, path.as_ref().to_path_buf(), tx)
        }))
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While creating a directory at {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirCreateDir(self.key, path.to_path_buf(), tx))
            .wrap_err(c)
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While removing a directory at {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirRemoveDir(self.key, path.to_path_buf(), tx))
            .wrap_err(c)
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        let path = path.as_ref();
        let c = format!("While removing a file at {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirRemoveFile(self.key, path.to_path_buf(), tx))
            .wrap_err(c)
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        let path = path.as_ref();
        let c = format!("While reading the contents of the directory {path:?} in a host folder");
        send_and_recv(|tx| FileSystemCommand::DirReadDir(self.key, path.to_path_buf(), tx))
            .wrap_err(c)
    }
}

impl File {
    /// Creates a new empty temporary file with read-write permissions.
    pub fn new() -> std::io::Result<Self> {
        let c = "While creating a temporary file on a host filesystem";
        send_and_recv(FileSystemCommand::FileCreateTemp)
            .map(|(key, temp_file_name)| Self {
                key,
                path: None,
                temp_file_name: Some(temp_file_name),
                futures: Default::default(),
            })
            .wrap_io_err(c)
    }

    /// Attempts to prompt the user to choose a file from their local machine using the
    /// JavaScript File System API.
    /// Then creates a `File` allowing read access to that file if they chose one
    /// successfully.
    /// If the File System API is not supported, this always returns `None` without doing anything.
    ///
    /// `extensions` should be a list of accepted file extensions for the file, without the leading
    /// `.`
    pub async fn from_file_picker(
        filter_name: &str,
        extensions: &[impl ToString],
    ) -> Result<(Self, String)> {
        let c = "While picking a file on a host filesystem";
        if !FileSystem::filesystem_supported() {
            return Err(Error::Wasm32FilesystemNotSupported).wrap_err(c);
        }
        send_and_await(|tx| {
            FileSystemCommand::FilePicker(
                filter_name.to_string(),
                extensions.iter().map(|e| e.to_string()).collect_vec(),
                tx,
            )
        })
        .await
        .map(|(key, name)| {
            (
                Self {
                    key,
                    path: Some(name.clone().into()),
                    temp_file_name: None,
                    futures: Default::default(),
                },
                name,
            )
        })
        .ok_or(Error::CancelledLoading)
        .wrap_err(c)
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
    pub async fn save(&self, filename: &str, _filter_name: &str) -> Result<()> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While saving the file {:?} in a host folder to disk",
            stripped_path
        );
        send_and_await(|tx| FileSystemCommand::FileSave(self.key, filename.to_string(), tx))
            .await
            .wrap_err(c)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = send_and_recv(|tx| {
            FileSystemCommand::FileDrop(self.key, self.temp_file_name.take(), tx)
        });
    }
}

impl crate::File for File {
    fn metadata(&self) -> std::io::Result<Metadata> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While getting metadata for file {:?} in a host folder",
            stripped_path
        );
        let size = send_and_recv(|tx| FileSystemCommand::FileSize(self.key, tx)).wrap_io_err(c)?;
        Ok(Metadata {
            is_file: true,
            size,
        })
    }

    fn set_len(&self, new_size: u64) -> std::io::Result<()> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While setting length of file {:?} in a host folder",
            stripped_path
        );
        send_and_recv(|tx| FileSystemCommand::FileSetLength(self.key, new_size, tx)).wrap_io_err(c)
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While reading from file {:?} in a host folder",
            stripped_path
        );
        let vec = send_and_recv(|tx| FileSystemCommand::FileRead(self.key, buf.len(), tx))
            .wrap_io_err(c)?;
        let length = vec.len();
        buf[..length].copy_from_slice(&vec[..]);
        Ok(length)
    }
}

impl futures_lite::AsyncRead for File {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously reading from file {:?} in a host folder",
            stripped_path
        );
        let mut futures = self.futures.lock();
        if futures.read.is_none() {
            futures.read = Some(send_and_wake(cx, |tx| {
                FileSystemCommand::FileRead(self.key, buf.len(), tx)
            }));
        }
        match futures.read.as_mut().unwrap().try_recv() {
            Ok(Ok(vec)) => {
                futures.read = None;
                let length = vec.len();
                buf[..length].copy_from_slice(&vec[..]);
                Poll::Ready(Ok(length))
            }
            Ok(Err(e)) => {
                futures.read = None;
                Poll::Ready(Err(e).wrap_io_err(c))
            }
            Err(_) => Poll::Pending,
        }
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While writing to file {:?} in a host folder", stripped_path);
        send_and_recv(|tx| FileSystemCommand::FileWrite(self.key, buf.to_vec(), tx))
            .wrap_io_err(c)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While flushing file {:?} in a host folder", stripped_path);
        send_and_recv(|tx| FileSystemCommand::FileFlush(self.key, tx)).wrap_io_err(c)
    }
}

impl futures_lite::AsyncWrite for File {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously writing to file {:?} in a host folder",
            stripped_path
        );
        let mut futures = self.futures.lock();
        if futures.write.is_none() {
            futures.write = Some(send_and_wake(cx, |tx| {
                FileSystemCommand::FileWrite(self.key, buf.to_vec(), tx)
            }));
        }
        match futures.write.as_mut().unwrap().try_recv() {
            Ok(Ok(())) => {
                futures.write = None;
                Poll::Ready(Ok(buf.len()))
            }
            Ok(Err(e)) => {
                futures.write = None;
                Poll::Ready(Err(e).wrap_io_err(c))
            }
            Err(_) => Poll::Pending,
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously flushing file {:?} in a host folder",
            stripped_path
        );
        let mut futures = self.futures.lock();
        if futures.flush.is_none() {
            futures.flush = Some(send_and_wake(cx, |tx| {
                FileSystemCommand::FileFlush(self.key, tx)
            }));
        }
        match futures.flush.as_mut().unwrap().try_recv() {
            Ok(Ok(())) => {
                futures.flush = None;
                Poll::Ready(Ok(()))
            }
            Ok(Err(e)) => {
                futures.flush = None;
                Poll::Ready(Err(e).wrap_io_err(c))
            }
            Err(_) => Poll::Pending,
        }
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<std::io::Result<()>> {
        let stripped_path = self
            .path
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
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!("While seeking file {:?} in a host folder", stripped_path);
        send_and_recv(|tx| FileSystemCommand::FileSeek(self.key, pos, tx)).wrap_io_err(c)
    }
}

impl futures_lite::AsyncSeek for File {
    fn poll_seek(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        pos: std::io::SeekFrom,
    ) -> Poll<std::io::Result<u64>> {
        let stripped_path = self
            .path
            .as_ref()
            .map(|p| p.as_str())
            .unwrap_or("<temporary file>");
        let c = format!(
            "While asynchronously seeking file {:?} in a host folder",
            stripped_path
        );
        let mut futures = self.futures.lock();
        if futures.seek.is_none() {
            futures.seek = Some(send_and_wake(cx, |tx| {
                FileSystemCommand::FileSeek(self.key, pos, tx)
            }));
        }
        match futures.seek.as_mut().unwrap().try_recv() {
            Ok(Ok(offset)) => {
                futures.seek = None;
                Poll::Ready(Ok(offset))
            }
            Ok(Err(e)) => {
                futures.seek = None;
                Poll::Ready(Err(e).wrap_io_err(c))
            }
            Err(_) => Poll::Pending,
        }
    }
}
