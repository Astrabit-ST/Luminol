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
pub struct FileSystem {
    key: usize,
    name: String,
    tx: mpsc::UnboundedSender<FileSystemCommand>,
}

#[derive(Debug)]
pub struct File {
    key: usize,
    tx: mpsc::UnboundedSender<FileSystemCommand>,
}

#[derive(Debug)]
pub struct FileSystemCommand(FileSystemCommandInner);

#[derive(Debug)]
enum FileSystemCommandInner {
    Supported(oneshot::Sender<bool>),
    Metadata(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<Metadata, Error>>,
    ),
    DirPicker(oneshot::Sender<Option<(usize, String)>>),
    DirOpenFile(
        usize,
        camino::Utf8PathBuf,
        OpenFlags,
        oneshot::Sender<Result<usize, Error>>,
    ),
    DirExists(usize, camino::Utf8PathBuf, oneshot::Sender<bool>),
    DirCreateDir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<(), Error>>,
    ),
    DirRemoveDir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<(), Error>>,
    ),
    DirRemoveFile(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<(), Error>>,
    ),
    DirReadDir(
        usize,
        camino::Utf8PathBuf,
        oneshot::Sender<Result<Vec<DirEntry>, Error>>,
    ),
    DirDrop(usize, oneshot::Sender<bool>),
    FileRead(usize, usize, oneshot::Sender<std::io::Result<Vec<u8>>>),
    FileWrite(usize, Vec<u8>, oneshot::Sender<std::io::Result<()>>),
    FileFlush(usize, oneshot::Sender<std::io::Result<()>>),
    FileSeek(
        usize,
        std::io::SeekFrom,
        oneshot::Sender<std::io::Result<u64>>,
    ),
    FileDrop(usize, oneshot::Sender<bool>),
}

impl FileSystem {
    /// Returns whether or not the user's browser supports the JavaScript File System API.
    pub fn filesystem_supported(filesystem_tx: mpsc::UnboundedSender<FileSystemCommand>) -> bool {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        filesystem_tx
            .send(FileSystemCommand(FileSystemCommandInner::Supported(
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    /// Attempts to prompt the user to choose a directory from their local machine using the
    /// JavaScript File System API.
    /// Then creates a `FileSystem` allowing read-write access to that directory if they chose one
    /// successfully.
    /// If the File System API is not supported, this always returns `None` without doing anything.
    pub async fn from_directory_picker(
        filesystem_tx: mpsc::UnboundedSender<FileSystemCommand>,
    ) -> Option<Self> {
        if !Self::filesystem_supported(filesystem_tx.clone()) {
            return None;
        }
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        filesystem_tx
            .send(FileSystemCommand(FileSystemCommandInner::DirPicker(
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.await.unwrap().map(|(key, name)| FileSystem {
            key,
            name,
            tx: filesystem_tx,
        })
    }

    /// Returns a path consisting of a single element: the name of the root directory of this
    /// filesystem.
    pub fn root_path(&self) -> &camino::Utf8Path {
        self.name.as_str().into()
    }
}

impl Drop for FileSystem {
    fn drop(&mut self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirDrop(
                self.key, oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap();
    }
}

impl FileSystemTrait for FileSystem {
    type File<'fs> = File where Self: 'fs;

    fn open_file(
        &self,
        path: impl AsRef<camino::Utf8Path>,
        flags: OpenFlags,
    ) -> Result<Self::File<'_>, Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirOpenFile(
                self.key,
                path.as_ref().to_path_buf(),
                flags,
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap().map(|key| File {
            key,
            tx: self.tx.clone(),
        })
    }

    fn metadata(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Metadata, Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::Metadata(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    fn rename(
        &self,
        from: impl AsRef<camino::Utf8Path>,
        to: impl AsRef<camino::Utf8Path>,
    ) -> Result<(), Error> {
        Err(Error::NotSupported)
    }

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirExists(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        Ok(oneshot_rx.blocking_recv().unwrap())
    }

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirCreateDir(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirRemoveDir(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirRemoveFile(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }

    fn read_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Vec<DirEntry>, Error> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::DirReadDir(
                self.key,
                path.as_ref().to_path_buf(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }
}

impl Drop for File {
    fn drop(&mut self) {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::FileDrop(
                self.key, oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap();
    }
}

impl std::io::Read for File {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::FileRead(
                self.key,
                buf.len(),
                oneshot_tx,
            )))
            .unwrap();
        let vec = oneshot_rx.blocking_recv().unwrap()?;
        let length = vec.len();
        buf[..length].copy_from_slice(&vec[..]);
        Ok(length)
    }
}

impl std::io::Write for File {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::FileWrite(
                self.key,
                buf.to_vec(),
                oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::FileFlush(
                self.key, oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }
}

impl std::io::Seek for File {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        let (oneshot_tx, oneshot_rx) = oneshot::channel();
        self.tx
            .send(FileSystemCommand(FileSystemCommandInner::FileSeek(
                self.key, pos, oneshot_tx,
            )))
            .unwrap();
        oneshot_rx.blocking_recv().unwrap()
    }
}

#[wasm_bindgen(inline_js = "
export function filesystem_supported() {
    return typeof window?.showOpenFilePicker === 'function'
        && typeof window?.showDirectoryPicker === 'function'
        && typeof FileSystemFileHandle === 'function'
        && typeof FileSystemWritableFileStream === 'function'
        && typeof FileSystemFileHandle?.prototype?.remove === 'function'
        && typeof FileSystemDirectoryHandle?.prototype?.remove === 'function';
}")]
extern "C" {
    fn filesystem_supported() -> bool;
}

#[wasm_bindgen(
    inline_js = "export async function show_directory_picker() { return await showDirectoryPicker({ mode: 'readwrite' }); }"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn show_directory_picker() -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(
    inline_js = "export async function move_file(file, dir, filename) { await file.move(dir, filename); }"
)]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn move_file(
        file: &web_sys::FileSystemFileHandle,
        dir: &web_sys::FileSystemDirectoryHandle,
        filename: &str,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(inline_js = "export async function remove_file(file) { await file.remove(); }")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn remove_file(file: &web_sys::FileSystemFileHandle) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(inline_js = "export async function remove_dir(dir) { await dir.remove(); }")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn remove_dir(dir: &web_sys::FileSystemDirectoryHandle) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(inline_js = "export function dir_values(dir) { return dir.values(); }")]
extern "C" {
    fn dir_values(dir: &web_sys::FileSystemDirectoryHandle) -> js_sys::AsyncIterator;
}

pub fn setup_main_thread_hooks(mut filesystem_rx: mpsc::UnboundedReceiver<FileSystemCommand>) {
    wasm_bindgen_futures::spawn_local(async move {
        web_sys::window().expect("cannot run `setup_main_thread_hooks()` outside of main thread");

        struct FileHandle {
            offset: usize,
            file_handle: web_sys::FileSystemFileHandle,
            read_allowed: bool,
            write_handle: Option<web_sys::FileSystemWritableFileStream>,
        }

        let mut dirs: slab::Slab<web_sys::FileSystemDirectoryHandle> = slab::Slab::new();
        let mut files: slab::Slab<FileHandle> = slab::Slab::new();

        async fn to_future<T>(promise: js_sys::Promise) -> Result<T, js_sys::Error>
        where
            T: JsCast,
        {
            wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map(|t| t.unchecked_into())
                .map_err(|e| e.unchecked_into())
        }

        async fn get_subdir(
            dir: &web_sys::FileSystemDirectoryHandle,
            path_iter: &mut camino::Iter<'_>,
        ) -> Option<web_sys::FileSystemDirectoryHandle> {
            let mut dir = dir.clone();
            loop {
                let Some(path_element) = path_iter.next() else {
                    return Some(dir);
                };
                if let Ok(subdir) = to_future(dir.get_directory_handle(path_element)).await {
                    dir = subdir;
                } else {
                    return None;
                }
            }
        }

        loop {
            let Some(command) = filesystem_rx.recv().await else {
                return;
            };
            web_sys::console::debug_1(&JsValue::from(format!(
                "Main thread received FS command: {:?}",
                command
            )));

            match command.0 {
                FileSystemCommandInner::Supported(oneshot_tx) => {
                    oneshot_tx.send(filesystem_supported()).unwrap();
                }

                FileSystemCommandInner::Metadata(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(name) = iter.next_back() else {
                        oneshot_tx
                            .send(Ok(Metadata {
                                is_file: false,
                                size: 0,
                            }))
                            .unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    if let Ok(file) =
                        to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(name))
                            .await
                    {
                        if let Ok(blob) = to_future::<web_sys::File>(file.get_file()).await {
                            oneshot_tx
                                .send(Ok(Metadata {
                                    is_file: true,
                                    size: blob.size() as u64,
                                }))
                                .unwrap();
                        } else {
                            oneshot_tx
                                .send(Err(Error::IoError(
                                    std::io::ErrorKind::PermissionDenied.into(),
                                )))
                                .unwrap();
                        }
                    } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                        subdir.get_directory_handle(name),
                    )
                    .await
                    .is_ok()
                    {
                        oneshot_tx
                            .send(Ok(Metadata {
                                is_file: false,
                                size: 0,
                            }))
                            .unwrap();
                    } else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                    }
                }

                FileSystemCommandInner::DirPicker(oneshot_tx) => {
                    if let Ok(dir) = show_directory_picker()
                        .await
                        .map(|x| x.unchecked_into::<web_sys::FileSystemDirectoryHandle>())
                    {
                        let name = dir.name();
                        oneshot_tx.send(Some((dirs.insert(dir), name))).unwrap();
                    } else {
                        oneshot_tx.send(None).unwrap();
                    }
                }

                FileSystemCommandInner::DirOpenFile(key, path, flags, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(filename) = iter.next_back() else {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    // If write and create permissions were both requested, then the file should be
                    // created if the file does not exist but all the parent directories do
                    let mut options = web_sys::FileSystemGetFileOptions::new();
                    if flags.contains(OpenFlags::Write) && flags.contains(OpenFlags::Create) {
                        options.create(true);
                    }
                    if let Ok(file_handle) = to_future::<web_sys::FileSystemFileHandle>(
                        subdir.get_file_handle_with_options(filename, &options),
                    )
                    .await
                    {
                        let mut handle = FileHandle {
                            offset: 0,
                            file_handle,
                            read_allowed: flags.contains(OpenFlags::Read),
                            write_handle: None,
                        };
                        // If write permissions were requested, try to get a write handle on the
                        // file, with truncation if requested
                        let mut options = web_sys::FileSystemCreateWritableOptions::new();
                        options.keep_existing_data(!flags.contains(OpenFlags::Truncate));
                        handle.write_handle = if flags.contains(OpenFlags::Write) {
                            to_future(handle.file_handle.create_writable_with_options(&options))
                                .await
                                .ok()
                        } else {
                            None
                        };
                        // If write and truncate permissions were both requested, try to flush the
                        // write handle (by closing and reopening) to perform the truncation
                        // immediately
                        let close_result = !flags.contains(OpenFlags::Truncate)
                            || if let Some(write_handle) = &handle.write_handle {
                                to_future::<JsValue>(write_handle.close()).await.is_ok()
                            } else {
                                true
                            };
                        let mut options = web_sys::FileSystemCreateWritableOptions::new();
                        options.keep_existing_data(true);
                        if flags.contains(OpenFlags::Truncate) && handle.write_handle.is_some() {
                            handle.write_handle =
                                to_future(handle.file_handle.create_writable_with_options(&options))
                                    .await
                                    .ok()
                        }

                        if (flags.contains(OpenFlags::Write) && handle.write_handle.is_none())
                            || !close_result
                        {
                            oneshot_tx
                                .send(Err(Error::IoError(
                                    std::io::ErrorKind::PermissionDenied.into(),
                                )))
                                .unwrap();
                        } else {
                            oneshot_tx.send(Ok(files.insert(handle))).unwrap();
                        }
                    } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                        subdir.get_directory_handle(filename),
                    )
                    .await
                    .is_ok()
                    {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                    } else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                    }
                }

                FileSystemCommandInner::DirExists(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(name) = iter.next_back() else {
                        oneshot_tx.send(true).unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(false).unwrap();
                        continue;
                    };
                    if to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(name))
                        .await
                        .is_ok()
                        || to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(name),
                        )
                        .await
                        .is_ok()
                    {
                        oneshot_tx.send(true).unwrap();
                    } else {
                        oneshot_tx.send(false).unwrap();
                    }
                }

                FileSystemCommandInner::DirCreateDir(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(dirname) = iter.next_back() else {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::AlreadyExists.into(),
                            )))
                            .unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    if to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(dirname))
                        .await
                        .is_ok()
                        || to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(dirname),
                        )
                        .await
                        .is_ok()
                    {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                    } else {
                        let mut options = web_sys::FileSystemGetDirectoryOptions::new();
                        options.create(true);
                        if to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle_with_options(dirname, &options),
                        )
                        .await
                        .is_ok()
                        {
                            oneshot_tx.send(Ok(())).unwrap();
                        } else {
                            oneshot_tx
                                .send(Err(Error::IoError(
                                    std::io::ErrorKind::PermissionDenied.into(),
                                )))
                                .unwrap();
                        }
                    }
                }

                FileSystemCommandInner::DirRemoveDir(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(dirname) = iter.next_back() else {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    if to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(dirname))
                        .await
                        .is_ok()
                    {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                    } else if let Ok(dir) = to_future::<web_sys::FileSystemDirectoryHandle>(
                        subdir.get_directory_handle(dirname),
                    )
                    .await
                    {
                        if remove_dir(&dir).await.is_ok() {
                            oneshot_tx.send(Ok(())).unwrap();
                        } else {
                            oneshot_tx
                                .send(Err(Error::IoError(
                                    std::io::ErrorKind::PermissionDenied.into(),
                                )))
                                .unwrap();
                        }
                    } else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                    }
                }

                FileSystemCommandInner::DirRemoveFile(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(filename) = iter.next_back() else {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                        continue;
                    };
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    if let Ok(file) =
                        to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(filename))
                            .await
                    {
                        if remove_file(&file).await.is_ok() {
                            oneshot_tx.send(Ok(())).unwrap();
                        } else {
                            oneshot_tx
                                .send(Err(Error::IoError(
                                    std::io::ErrorKind::PermissionDenied.into(),
                                )))
                                .unwrap();
                        }
                    } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                        subdir.get_directory_handle(filename),
                    )
                    .await
                    .is_ok()
                    {
                        oneshot_tx
                            .send(Err(Error::IoError(
                                std::io::ErrorKind::PermissionDenied.into(),
                            )))
                            .unwrap();
                    } else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                    }
                }

                FileSystemCommandInner::DirReadDir(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };
                    let entry_iter = dir_values(&subdir);
                    let mut vec = Vec::new();
                    loop {
                        let entry = to_future::<js_sys::IteratorNext>(entry_iter.next().unwrap())
                            .await
                            .unwrap();
                        if entry.done() {
                            break;
                        }
                        let entry = entry.value().unchecked_into::<web_sys::FileSystemHandle>();
                        match entry.kind() {
                            web_sys::FileSystemHandleKind::File => {
                                let entry = entry.unchecked_into::<web_sys::FileSystemFileHandle>();
                                if let Ok(blob) = to_future::<web_sys::File>(entry.get_file()).await
                                {
                                    vec.push(DirEntry::new(
                                        path.join(entry.name()),
                                        Metadata {
                                            is_file: true,
                                            size: blob.size() as u64,
                                        },
                                    ));
                                }
                            }
                            web_sys::FileSystemHandleKind::Directory => {
                                vec.push(DirEntry::new(
                                    path.join(entry.name()),
                                    Metadata {
                                        is_file: false,
                                        size: 0,
                                    },
                                ));
                            }
                            _ => (),
                        }
                    }
                    oneshot_tx.send(Ok(vec)).unwrap();
                }

                FileSystemCommandInner::DirDrop(key, oneshot_tx) => {
                    if dirs.contains(key) {
                        dirs.remove(key);
                        oneshot_tx.send(true).unwrap();
                    } else {
                        oneshot_tx.send(false).unwrap();
                    }
                }

                FileSystemCommandInner::FileRead(key, max_length, oneshot_tx) => {
                    let file = files.get_mut(key).unwrap();
                    let Some(read_handle) = (if file.read_allowed {
                        to_future::<web_sys::File>(file.file_handle.get_file())
                            .await
                            .ok()
                    } else {
                        None
                    }) else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    };
                    let blob = read_handle
                        .slice_with_f64_and_f64(
                            file.offset as f64,
                            (file.offset + max_length) as f64,
                        )
                        .unwrap();
                    let Ok(buffer) = to_future::<js_sys::ArrayBuffer>(blob.array_buffer()).await
                    else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    };
                    let u8_array = js_sys::Uint8Array::new(&buffer);
                    let vec = u8_array.to_vec();
                    file.offset += vec.len();
                    oneshot_tx.send(Ok(vec)).unwrap();
                }

                FileSystemCommandInner::FileWrite(key, mut vec, oneshot_tx) => {
                    let file = files.get_mut(key).unwrap();
                    let Some(write_handle) = &file.write_handle else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    };
                    // TODO: `write_handle.write_with_u8_array()` will not work here when
                    // theading is enabled. Possible wasm_bindgen bug?
                    // We are using `write_handle.write_with_buffer_source()` here as a workaround
                    // that does the same thing but with an extra memory allocation.
                    // Check if this is fixed in newer versions of wasm_bindgen.
                    let mut u8_array =
                        js_sys::Uint8Array::new(&JsValue::from_f64(vec.len() as f64));
                    u8_array.copy_from(&vec[..]);
                    if to_future::<JsValue>(write_handle.seek_with_f64(file.offset as f64).unwrap())
                        .await
                        .is_ok()
                        && to_future::<JsValue>(
                            write_handle.write_with_buffer_source(&u8_array).unwrap(),
                        )
                        .await
                        .is_ok()
                    {
                        file.offset += vec.len();
                        oneshot_tx.send(Ok(())).unwrap();
                    } else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                    }
                }

                FileSystemCommandInner::FileFlush(key, oneshot_tx) => {
                    let file = files.get_mut(key).unwrap();
                    if file.write_handle.is_none() {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    }
                    // Closing and reopening the handle is the only way to flush
                    if to_future::<JsValue>(file.write_handle.as_ref().unwrap().close())
                        .await
                        .is_err()
                    {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    }
                    let mut options = web_sys::FileSystemCreateWritableOptions::new();
                    options.keep_existing_data(true);
                    if let Ok(write_handle) =
                        to_future(file.file_handle.create_writable_with_options(&options)).await
                    {
                        file.write_handle = Some(write_handle);
                        oneshot_tx.send(Ok(())).unwrap();
                    } else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                    }
                }

                FileSystemCommandInner::FileSeek(key, seek_from, oneshot_tx) => {
                    let file = files.get_mut(key).unwrap();
                    let Some(read_handle) = (if file.read_allowed {
                        to_future::<web_sys::File>(file.file_handle.get_file())
                            .await
                            .ok()
                    } else {
                        None
                    }) else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::PermissionDenied.into()))
                            .unwrap();
                        continue;
                    };
                    let size = read_handle.size();
                    let new_offset = match seek_from {
                        std::io::SeekFrom::Start(i) => i as i64,
                        std::io::SeekFrom::End(i) => i + size as i64,
                        std::io::SeekFrom::Current(i) => i + file.offset as i64,
                    };
                    if new_offset >= 0 {
                        file.offset = new_offset as usize;
                        oneshot_tx.send(Ok(new_offset as u64)).unwrap();
                    } else {
                        oneshot_tx
                            .send(Err(std::io::ErrorKind::InvalidInput.into()))
                            .unwrap();
                    }
                }

                FileSystemCommandInner::FileDrop(key, oneshot_tx) => {
                    if files.contains(key) {
                        let file = files.remove(key);
                        // We need to close the write handle to flush any changes that the user
                        // made to the file
                        if let Some(write_handle) = &file.write_handle {
                            let _ = to_future::<JsValue>(write_handle.close()).await;
                        }
                        oneshot_tx.send(true).unwrap();
                    } else {
                        oneshot_tx.send(false).unwrap();
                    }
                }
            }
        }
    });
}
