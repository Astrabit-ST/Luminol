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

use super::util::{generate_key, get_subdir, idb, to_future};
use super::{FileSystemCommand, FileSystemCommandInner};
use crate::{DirEntry, Error, Metadata, OpenFlags, Result};
use indexed_db_futures::prelude::*;
use wasm_bindgen::prelude::*;

pub fn setup_main_thread_hooks(mut filesystem_rx: flume::Receiver<FileSystemCommand>) {
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

        loop {
            let Ok(command) = filesystem_rx.recv_async().await else {
                tracing::warn!(
                    "FileSystem main thread loop is stopping! This is not supposed to happen."
                );
                return;
            };
            tracing::debug!("Main thread received FS command: {:?}", command.0);

            match command.0 {
                FileSystemCommandInner::Supported(oneshot_tx) => {
                    oneshot_tx
                        .send(luminol_web::bindings::filesystem_supported())
                        .unwrap();
                }

                FileSystemCommandInner::DirEntryMetadata(key, path, oneshot_tx) => {
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
                    if let Ok(dir) = luminol_web::bindings::show_directory_picker().await {
                        // Try to insert the handle into IndexedDB
                        let idb_key = generate_key();
                        let idb_ok = {
                            let idb_key = idb_key.as_str();
                            super::util::idb(IdbTransactionMode::Readwrite, |store| {
                                store.put_key_val_owned(idb_key, &dir)
                            })
                            .await
                            .is_ok()
                        };

                        let name = dir.name();
                        oneshot_tx
                            .send(Some((
                                dirs.insert(dir),
                                name,
                                if idb_ok { Some(idb_key) } else { None },
                            )))
                            .unwrap();
                    } else {
                        oneshot_tx.send(None).unwrap();
                    }
                }

                FileSystemCommandInner::DirFromIdb(idb_key, oneshot_tx) => {
                    let idb_key = idb_key.as_str();
                    if let Ok(future) = super::util::idb(IdbTransactionMode::Readonly, |store| {
                        store.get_owned(idb_key)
                    })
                    .await
                    {
                        if let Some(dir) = future.await.ok().flatten() {
                            let dir = dir.unchecked_into::<web_sys::FileSystemDirectoryHandle>();
                            if luminol_web::bindings::request_permission(&dir).await {
                                let name = dir.name();
                                oneshot_tx.send(Some((dirs.insert(dir), name))).unwrap();
                            } else {
                                oneshot_tx.send(None).unwrap();
                            }
                        } else {
                            oneshot_tx.send(None).unwrap();
                        }
                    } else {
                        oneshot_tx.send(None).unwrap();
                    }
                }

                FileSystemCommandInner::DirSubdir(key, path, oneshot_tx) => {
                    let mut iter = path.iter();
                    let Some(dir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await else {
                        oneshot_tx.send(Err(Error::NotExist)).unwrap();
                        continue;
                    };

                    // Try to insert the handle into IndexedDB
                    let idb_key = generate_key();
                    let idb_ok = {
                        let idb_key = idb_key.as_str();
                        super::util::idb(IdbTransactionMode::Readwrite, |store| {
                            store.put_key_val_owned(idb_key, &dir)
                        })
                        .await
                        .is_ok()
                    };

                    let name = dir.name();
                    oneshot_tx
                        .send(Ok((
                            dirs.insert(dir),
                            name,
                            if idb_ok { Some(idb_key) } else { None },
                        )))
                        .unwrap();
                }

                FileSystemCommandInner::DirIdbDrop(idb_key, oneshot_tx) => {
                    let idb_key = idb_key.as_str();
                    oneshot_tx
                        .send(
                            super::util::idb(IdbTransactionMode::Readwrite, |store| {
                                store.delete_owned(idb_key)
                            })
                            .await
                            .is_ok(),
                        )
                        .unwrap();
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

                FileSystemCommandInner::DirEntryExists(key, path, oneshot_tx) => {
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
                        let mut options = web_sys::FileSystemRemoveOptions::new();
                        options.recursive(true);
                        if to_future::<JsValue>(subdir.remove_entry_with_options(dirname, &options))
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
                        if to_future::<JsValue>(subdir.remove_entry(filename))
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
                    let entry_iter = luminol_web::bindings::dir_values(&subdir);
                    let mut vec = Vec::new();
                    loop {
                        let Ok(entry) =
                            to_future::<js_sys::IteratorNext>(entry_iter.next().unwrap()).await
                        else {
                            break;
                        };
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

                FileSystemCommandInner::DirClone(key, oneshot_tx) => {
                    oneshot_tx
                        .send(dirs.insert(dirs.get(key).unwrap().clone()))
                        .unwrap();
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

                FileSystemCommandInner::FileWrite(key, vec, oneshot_tx) => {
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
                    let u8_array = js_sys::Uint8Array::new(&JsValue::from_f64(vec.len() as f64));
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
                FileSystemCommandInner::FileSize(key, oneshot_tx) => {
                    let file = files.get_mut(key).unwrap();
                    if let Ok(file) = to_future::<web_sys::File>(file.file_handle.get_file()).await
                    {
                        oneshot_tx.send(file.size() as u64).unwrap();
                    }
                }
            }
        }
    });
}
