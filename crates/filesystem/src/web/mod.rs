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

mod events;
mod util;
pub use events::setup_main_thread_hooks;

use super::FileSystem as FileSystemTrait;
use super::{DirEntry, Error, Metadata, OpenFlags, Result};
use util::{send_and_await, send_and_recv};

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
}

#[derive(Debug)]
enum FileSystemCommand {
    Supported(oneshot::Sender<bool>),
    DirEntryMetadata(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<Metadata>>,
    ),
    DirPicker(oneshot::Sender<Option<(usize, String, Option<String>)>>),
    DirFromIdb(String, oneshot::Sender<Option<(usize, String)>>),
    DirSubdir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<(usize, String, Option<String>)>>,
    ),
    DirIdbDrop(String, oneshot::Sender<bool>),
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
    FileRead(usize, usize, oneshot::Sender<std::io::Result<Vec<u8>>>),
    FileWrite(usize, Vec<u8>, oneshot::Sender<std::io::Result<()>>),
    FileFlush(usize, oneshot::Sender<std::io::Result<()>>),
    FileSeek(
        usize,
        std::io::SeekFrom,
        oneshot::Sender<std::io::Result<u64>>,
    ),
    FileSize(usize, oneshot::Sender<std::io::Result<u64>>),
    FileDrop(usize, oneshot::Sender<bool>),
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
        send_and_recv(|tx| FileSystemCommand::Supported(tx))
    }

    /// Attempts to prompt the user to choose a directory from their local machine using the
    /// JavaScript File System API.
    /// Then creates a `FileSystem` allowing read-write access to that directory if they chose one
    /// successfully.
    /// If the File System API is not supported, this always returns `None` without doing anything.
    pub async fn from_folder_picker() -> Result<Self> {
        if !Self::filesystem_supported() {
            return Err(Error::Wasm32FilesystemNotSupported);
        }
        send_and_await(|tx| FileSystemCommand::DirPicker(tx))
            .await
            .map(|(key, name, idb_key)| FileSystem { key, name, idb_key })
            .ok_or(Error::CancelledLoading)
    }

    /// Attempts to restore a previously created `FileSystem` using its `.idb_key()`.
    pub async fn from_idb_key(idb_key: String) -> Result<Self> {
        if !Self::filesystem_supported() {
            return Err(Error::Wasm32FilesystemNotSupported);
        }
        send_and_await(|tx| FileSystemCommand::DirFromIdb(idb_key.clone(), tx))
            .await
            .map(|(key, name)| FileSystem {
                key,
                name,
                idb_key: Some(idb_key),
            })
            .ok_or(Error::MissingIDB)
    }

    /// Creates a new `FileSystem` from a subdirectory of this one.
    pub fn subdir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Self> {
        send_and_recv(|tx| FileSystemCommand::DirSubdir(self.key, path.as_ref().to_path_buf(), tx))
            .map(|(key, name, idb_key)| FileSystem { key, name, idb_key })
    }

    /// Drops the directory with the given key from IndexedDB if it exists in there.
    pub fn idb_drop(idb_key: String) -> bool {
        send_and_recv(|tx| FileSystemCommand::DirIdbDrop(idb_key, tx))
    }

    /// Returns a path consisting of a single element: the name of the root directory of this
    /// filesystem.
    pub fn root_path(&self) -> &camino::Utf8Path {
        self.name.as_str().into()
    }

    /// Returns the key needed to restore this `FileSystem` using `FileSystem::from_idb()`.
    pub fn idb_key(&self) -> Option<&str> {
        self.idb_key.as_deref()
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
            idb_key: self.idb_key.clone(),
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
        send_and_recv(|tx| {
            FileSystemCommand::DirOpenFile(self.key, path.as_ref().to_path_buf(), flags, tx)
        })
        .map(|key| File { key })
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata> {
        send_and_recv(|tx| {
            FileSystemCommand::DirEntryMetadata(self.key, path.as_ref().to_path_buf(), tx)
        })
    }

    fn rename(
        &self,
        _from: impl AsRef<camino::Utf8Path>,
        _to: impl AsRef<camino::Utf8Path>,
    ) -> Result<()> {
        Err(Error::NotSupported)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool> {
        Ok(send_and_recv(|tx| {
            FileSystemCommand::DirEntryExists(self.key, path.as_ref().to_path_buf(), tx)
        }))
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        send_and_recv(|tx| {
            FileSystemCommand::DirCreateDir(self.key, path.as_ref().to_path_buf(), tx)
        })
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        send_and_recv(|tx| {
            FileSystemCommand::DirRemoveDir(self.key, path.as_ref().to_path_buf(), tx)
        })
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<()> {
        send_and_recv(|tx| {
            FileSystemCommand::DirRemoveFile(self.key, path.as_ref().to_path_buf(), tx)
        })
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>> {
        send_and_recv(|tx| FileSystemCommand::DirReadDir(self.key, path.as_ref().to_path_buf(), tx))
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let _ = send_and_recv(|tx| FileSystemCommand::FileDrop(self.key, tx));
    }
}

impl crate::File for File {
    fn metadata(&self) -> crate::Result<Metadata> {
        let size = send_and_recv(|tx| FileSystemCommand::FileSize(self.key, tx))?;
        Ok(Metadata {
            is_file: true,
            size,
        })
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let vec = send_and_recv(|tx| FileSystemCommand::FileRead(self.key, buf.len(), tx))?;
        let length = vec.len();
        buf[..length].copy_from_slice(&vec[..]);
        Ok(length)
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        send_and_recv(|tx| FileSystemCommand::FileWrite(self.key, buf.to_vec(), tx))?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        send_and_recv(|tx| FileSystemCommand::FileFlush(self.key, tx))
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        send_and_recv(|tx| FileSystemCommand::FileSeek(self.key, pos, tx))
    }
}
