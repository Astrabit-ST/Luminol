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
use wasm_bindgen::prelude::*;

use super::FileSystem as FileSystemTrait;
use super::{DirEntry, Error, Metadata, OpenFlags};

#[derive(Debug)]
pub struct FileSystem<'tx> {
    tx: &'tx mpsc::UnboundedSender<FileSystemCommand>,
    key: usize,
}

#[derive(Debug)]
pub struct File<'tx> {
    tx: &'tx mpsc::UnboundedSender<FileSystemCommand>,
    key: usize,
}

pub enum FileSystemCommand {
    DirPicker(oneshot::Sender<Option<usize>>),
    DropDir(usize, oneshot::Sender<bool>),
}

impl FileSystem<'_> {
    /// Returns whether or not the user's browser supports the JavaScript File System API.
    pub fn filesystem_supported() -> bool {
        todo!();
    }

    /// Attempts to prompt the user to choose a directory from their local machine using the
    /// JavaScript File System API.
    /// Then creates a `FileSystem` allowing read-write access to that directory if they chose one
    /// successfully.
    /// If the File System API is not supported, this always returns `None` without doing anything.
    pub async fn from_directory_picker(
        filesystem_tx: &mpsc::UnboundedSender<FileSystemCommand>,
    ) -> Option<FileSystem<'_>> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        filesystem_tx
            .send(FileSystemCommand::DirPicker(oneshot_tx))
            .unwrap();
        if let Ok(Some(key)) = oneshot_rx.await {
            Some(FileSystem {
                tx: filesystem_tx,
                key,
            })
        } else {
            None
        }
    }
}

impl Drop for FileSystem<'_> {
    fn drop(&mut self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand::DropDir(self.key, oneshot_tx))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap();
    }
}

impl<'tx> FileSystemTrait for FileSystem<'tx> {
    type File<'fs> = File<'tx> where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        todo!();
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        todo!();
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<(), Error> {
        todo!();
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        todo!();
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        todo!();
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        todo!();
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        todo!();
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        todo!();
    }
}

impl Drop for File<'_> {
    fn drop(&mut self) {
        todo!();
    }
}

impl std::io::Read for File<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        todo!();
    }
}

impl std::io::Write for File<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        todo!();
    }

    fn flush(&mut self) -> std::io::Result<()> {
        todo!();
    }
}

impl std::io::Seek for File<'_> {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        todo!();
    }
}

#[wasm_bindgen(
    inline_js = "export async function show_directory_picker() { return await showDirectoryPicker({ mode: 'readwrite' }); }"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn show_directory_picker() -> Result<JsValue, JsValue>;
}

pub fn setup_main_thread_hooks(mut filesystem_rx: mpsc::UnboundedReceiver<FileSystemCommand>) {
    wasm_bindgen_futures::spawn_local(async move {
        let window = web_sys::window()
            .expect("cannot run `setup_main_thread_hooks()` outside of main thread");

        let mut dirs = slab::Slab::new();

        loop {
            let Some(command) = filesystem_rx.recv().await else {
                return;
            };

            match command {
                FileSystemCommand::DirPicker(oneshot_tx) => {
                    if let Ok(dir) = show_directory_picker().await {
                        let dir = dir.unchecked_into::<web_sys::FileSystemDirectoryHandle>();
                        oneshot_tx.send(Some(dirs.insert(dir))).unwrap();
                    } else {
                        oneshot_tx.send(None).unwrap();
                    }
                }

                FileSystemCommand::DropDir(key, oneshot_tx) => {
                    if dirs.contains(key) {
                        dirs.remove(key);
                        oneshot_tx.send(true).unwrap();
                    } else {
                        oneshot_tx.send(false).unwrap();
                    }
                }

                _ => todo!(),
            }
        }
    });
}
