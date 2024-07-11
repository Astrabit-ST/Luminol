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

use super::util::{
    generate_key, get_subdir, get_subdir_create, get_tmp_dir, handle_event, to_future,
};
use super::FileSystemCommand;
use crate::{DirEntry, Error, Metadata, OpenFlags};
use luminol_web::IdbQuerySource;
use std::io::ErrorKind::{InvalidInput, PermissionDenied};
use wasm_bindgen::prelude::*;

pub fn setup_main_thread_hooks(main_channels: super::MainChannels) {
    wasm_bindgen_futures::spawn_local(async move {
        let window = web_sys::window()
            .expect("cannot run `setup_main_thread_hooks()` outside of main thread");
        let document = window
            .document()
            .expect("cannot run `setup_main_thread_hooks()` outside of main thread");
        let storage = window.navigator().storage();

        struct FileHandle {
            offset: usize,
            file_handle: web_sys::FileSystemFileHandle,
            read_allowed: bool,
            write_handle: Option<web_sys::FileSystemWritableFileStream>,
        }

        let mut dirs: slab::Slab<web_sys::FileSystemDirectoryHandle> = slab::Slab::new();
        let mut files: slab::Slab<FileHandle> = slab::Slab::new();

        loop {
            let Ok(command) = main_channels.command_rx.recv_async().await else {
                tracing::warn!(
                    "FileSystem main thread loop is stopping! This is not supposed to happen."
                );
                return;
            };
            tracing::debug!("Main thread received FS command: {:?}", command);

            match command {
                FileSystemCommand::Supported(tx) => {
                    handle_event(tx, async { luminol_web::bindings::filesystem_supported() }).await;
                }

                FileSystemCommand::DirEntryMetadata(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let Some(name) = iter.next_back() else {
                            return Ok(Metadata {
                                is_file: false,
                                size: 0,
                            });
                        };
                        let subdir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;

                        if let Ok(file) =
                            to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(name))
                                .await
                        {
                            // If the path is a file
                            to_future::<web_sys::File>(file.get_file())
                                .await
                                .map(|blob| Metadata {
                                    is_file: true,
                                    size: blob.size() as u64,
                                })
                                .map_err(|e| {
                                    Error::IoError(std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                            "Failed to get read handle to file: {}",
                                            e.to_string()
                                        ),
                                    ))
                                    .into()
                                })
                        } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(name),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a directory
                            Ok(Metadata {
                                is_file: false,
                                size: 0,
                            })
                        } else {
                            // If the path is neither a file nor a directory
                            Err(Error::NotExist.into())
                        }
                    })
                    .await;
                }

                FileSystemCommand::DirPicker(tx) => {
                    handle_event(tx, async {
                        let dir = luminol_web::bindings::show_directory_picker().await.ok()?;
                        let name = dir.name();
                        Some((dirs.insert(dir), name))
                    })
                    .await;
                }

                FileSystemCommand::DirFromIdb(idb_key, tx) => {
                    handle_event(tx, async {
                        let dir = luminol_web::idb("filesystem.dir_handles", luminol_web::IdbTransactionMode::Readonly, |store| {
                            store.get_owned(&idb_key)
                        })
                        .await
                        .map_err(|e| color_eyre::eyre::eyre!(format!("Failed to restore directory handle from IndexedDB: {}", e.to_string())))?
                        .await
                        .map_err(|e| color_eyre::eyre::eyre!(format!("Failed to restore directory handle from IndexedDB: {}", e.to_string())))?
                        .ok_or(color_eyre::eyre::eyre!("Failed to restore directory handle from IndexedDB: No directory handle found for the given IndexedDB key"))?;
                        let dir = dir.unchecked_into::<web_sys::FileSystemDirectoryHandle>();
                        let key =
                            luminol_web::bindings::request_permission(&dir)
                                .await
                                .map_err(|e| color_eyre::eyre::eyre!("Failed to request permission for directory handle restored from IndexedDB: {}", e.to_string()))?
                                .then(|| {
                                    let name = dir.name();
                                    (dirs.insert(dir), name)
                                }).ok_or(color_eyre::eyre::eyre!("Failed to request permission for directory handle restored from IndexedDB"))?;
                        luminol_web::idb("filesystem.dir_handles", luminol_web::IdbTransactionMode::Readwrite, |store| {
                            store.delete_owned(&idb_key)
                        })
                        .await
                        .map_err(|e| color_eyre::eyre::eyre!(format!("Failed to remove directory handle from IndexedDB: {}", e.to_string())))?;
                        Ok(key)
                    })
                    .await;
                }

                FileSystemCommand::DirToIdb(key, idb_key, tx) => {
                    handle_event(tx, async {
                        let dir = dirs.get(key).unwrap();
                        luminol_web::idb(
                            "filesystem.dir_handles",
                            luminol_web::IdbTransactionMode::Readwrite,
                            |store| store.put_key_val_owned(idb_key, dir),
                        )
                        .await
                        .is_ok()
                    })
                    .await;
                }

                FileSystemCommand::DirSubdir(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let dir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;
                        let name = dir.name();
                        Ok((dirs.insert(dir), name))
                    })
                    .await;
                }

                FileSystemCommand::DirOpenFile(key, path, flags, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let filename = iter
                            .next_back()
                            .ok_or(Error::IoError(std::io::Error::new(PermissionDenied, "Tried to open a file but the given path corresponded to a directory")))?;
                        let subdir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;

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
                            // If the path is a file

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
                            if flags.contains(OpenFlags::Truncate) && handle.write_handle.is_some()
                            {
                                handle.write_handle = to_future(
                                    handle.file_handle.create_writable_with_options(&options),
                                )
                                .await
                                .ok()
                            }

                            if (flags.contains(OpenFlags::Write) && handle.write_handle.is_none())
                                || !close_result
                            {
                                Err(Error::IoError(std::io::Error::new(PermissionDenied, "Failed to obtain write permissions for file"))
                                    .into())
                            } else {
                                Ok(files.insert(handle))
                            }
                        } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(filename),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a directory
                            Err(std::io::Error::new(PermissionDenied, "Tried to open a file but the given path corresponded to a directory").into())
                        } else {
                            // If the path is neither a file nor a directory
                            Err(Error::NotExist.into())
                        }
                    })
                    .await;
                }

                FileSystemCommand::DirEntryExists(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let Some(name) = iter.next_back() else {
                            return true;
                        };
                        let Some(subdir) = get_subdir(dirs.get(key).unwrap(), &mut iter).await
                        else {
                            return false;
                        };
                        to_future::<web_sys::FileSystemFileHandle>(subdir.get_file_handle(name))
                            .await
                            .is_ok()
                            || to_future::<web_sys::FileSystemDirectoryHandle>(
                                subdir.get_directory_handle(name),
                            )
                            .await
                            .is_ok()
                    })
                    .await;
                }

                FileSystemCommand::DirCreateDir(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        get_subdir_create(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::IoError(std::io::Error::new(
                                PermissionDenied,
                                "Failed to create directory",
                            )))?;
                        Ok(())
                    })
                    .await;
                }

                FileSystemCommand::DirRemoveDir(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let dirname = iter
                            .next_back()
                            .ok_or(Error::IoError(std::io::Error::new(PermissionDenied, "Failed to remove directory")))?;
                        let subdir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;

                        if to_future::<web_sys::FileSystemFileHandle>(
                            subdir.get_file_handle(dirname),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a file
                            Err(std::io::Error::new(PermissionDenied, "Tried to remove a directory but the given path corresponded to a file").into())
                        } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(dirname),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a directory
                            let mut options = web_sys::FileSystemRemoveOptions::new();
                            options.recursive(true);
                            to_future::<JsValue>(
                                subdir.remove_entry_with_options(dirname, &options),
                            )
                            .await
                            .map(|_| ())
                            .map_err(|e| std::io::Error::new(PermissionDenied, format!("Failed to remove directory: {}", e.to_string())).into())
                        } else {
                            // If the path is neither a file nor a directory
                            Err(Error::NotExist.into())
                        }
                    })
                    .await;
                }

                FileSystemCommand::DirRemoveFile(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let filename = iter
                            .next_back()
                            .ok_or(std::io::Error::new(PermissionDenied, "Tried to remove a file but the given path corresponded to a directory"))?;
                        let subdir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;

                        if to_future::<web_sys::FileSystemFileHandle>(
                            subdir.get_file_handle(filename),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a file
                            to_future::<JsValue>(subdir.remove_entry(filename))
                                .await
                                .map(|_| ())
                                .map_err(|e| std::io::Error::new(PermissionDenied, format!("Failed to remove file: {}", e.to_string())).into())
                        } else if to_future::<web_sys::FileSystemDirectoryHandle>(
                            subdir.get_directory_handle(filename),
                        )
                        .await
                        .is_ok()
                        {
                            // If the path is a directory
                            Err(std::io::Error::new(PermissionDenied, "Tried to remove a file but the given path corresponded to a directory").into())
                        } else {
                            // If the path is neither a file nor a directory
                            Err(Error::NotExist.into())
                        }
                    })
                    .await;
                }

                FileSystemCommand::DirReadDir(key, path, tx) => {
                    handle_event(tx, async {
                        let mut iter = path.iter();
                        let subdir = get_subdir(dirs.get(key).unwrap(), &mut iter)
                            .await
                            .ok_or(Error::NotExist)?;
                        let entry_iter = luminol_web::bindings::dir_values(&subdir);

                        let mut vec = Vec::new();
                        loop {
                            let Ok(entry) = to_future::<js_sys::IteratorNext>(
                                entry_iter.next().map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                            "Failed to read directory contents: {}",
                                            e.unchecked_into::<js_sys::Error>().to_string()
                                        ),
                                    )
                                })?,
                            )
                            .await
                            else {
                                break;
                            };
                            if entry.done() {
                                break;
                            }

                            let entry = entry.value().unchecked_into::<web_sys::FileSystemHandle>();
                            match entry.kind() {
                                web_sys::FileSystemHandleKind::File => {
                                    let entry =
                                        entry.unchecked_into::<web_sys::FileSystemFileHandle>();
                                    if let Ok(blob) =
                                        to_future::<web_sys::File>(entry.get_file()).await
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

                        Ok(vec)
                    })
                    .await;
                }

                FileSystemCommand::DirDrop(key, tx) => {
                    handle_event(tx, async {
                        if dirs.contains(key) {
                            dirs.remove(key);
                            true
                        } else {
                            false
                        }
                    })
                    .await;
                }

                FileSystemCommand::DirClone(key, tx) => {
                    handle_event(tx, async { dirs.insert(dirs.get(key).unwrap().clone()) }).await;
                }

                FileSystemCommand::FileCreateTemp(tx) => {
                    handle_event(tx, async {
                        let tmp_dir = get_tmp_dir(&storage).await?;

                        let filename = generate_key();

                        let mut options = web_sys::FileSystemGetFileOptions::new();
                        options.create(true);
                        let file_handle = to_future::<web_sys::FileSystemFileHandle>(
                            tmp_dir.get_file_handle_with_options(&filename, &options),
                        )
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                PermissionDenied,
                                format!("Failed to create temporary file: {}", e.to_string()),
                            )
                        })?;

                        let write_handle =
                            to_future(file_handle.create_writable())
                                .await
                                .map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                        "Failed to obtain write permissions for temporary file: {}",
                                        e.to_string()
                                    ),
                                    )
                                })?;

                        Ok((
                            files.insert(FileHandle {
                                offset: 0,
                                file_handle,
                                read_allowed: true,
                                write_handle: Some(write_handle),
                            }),
                            filename,
                        ))
                    })
                    .await;
                }

                FileSystemCommand::FileSetLength(key, new_size, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();
                        let write_handle =
                            file.write_handle.as_ref().ok_or(std::io::Error::new(
                                PermissionDenied,
                                "Attempted to write to file with no write permissions",
                            ))?;
                        to_future::<JsValue>(
                            write_handle
                                .truncate_with_f64(new_size as f64)
                                .map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                            "Failed to set file length: {}",
                                            e.unchecked_into::<js_sys::Error>().to_string()
                                        ),
                                    )
                                })?,
                        )
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                PermissionDenied,
                                format!(
                                    "Failed to set file length: {}",
                                    e.unchecked_into::<js_sys::Error>().to_string()
                                ),
                            )
                        })?;
                        Ok(())
                    })
                    .await;
                }

                FileSystemCommand::FilePicker(filter_name, extensions, tx) => {
                    handle_event(tx, async {
                        let file_handle =
                            luminol_web::bindings::show_file_picker(&filter_name, &extensions)
                                .await
                                .ok()?;
                        let name = file_handle.name();

                        Some((
                            files.insert(FileHandle {
                                offset: 0,
                                file_handle,
                                read_allowed: true,
                                write_handle: None,
                            }),
                            name,
                        ))
                    })
                    .await;
                }

                FileSystemCommand::FileSave(key, filename, tx) => {
                    handle_event(tx, async {
                        let file = files.get(key).unwrap();
                        let file_handle = file.read_allowed.then_some(&file.file_handle).ok_or(
                            std::io::Error::new(
                                PermissionDenied,
                                "Attempted to read from file with no read permissions",
                            ),
                        )?;

                        let blob = to_future::<web_sys::File>(file_handle.get_file())
                            .await
                            .map_err(|e| {
                                Error::IoError(std::io::Error::new(
                                    PermissionDenied,
                                    format!("Failed to get read handle to file: {}", e.to_string()),
                                ))
                            })?;
                        let url =
                            web_sys::Url::create_object_url_with_blob(&blob).map_err(|e| {
                                Error::IoError(std::io::Error::new(
                                    PermissionDenied,
                                    format!(
                                        "Failed to create object URL from blob: {}",
                                        e.unchecked_into::<js_sys::Error>().to_string()
                                    ),
                                ))
                            })?;

                        let anchor = document
                            .create_element("a")
                            .map_err(|e| {
                                Error::IoError(std::io::Error::new(
                                    PermissionDenied,
                                    format!(
                                        "Failed to create anchor element for downloading files: {}",
                                        e.unchecked_into::<js_sys::Error>().to_string()
                                    ),
                                ))
                            })?
                            .unchecked_into::<web_sys::HtmlAnchorElement>();
                        anchor.set_href(&url);
                        anchor.set_download(&filename);
                        anchor.click();
                        anchor.remove();
                        let _ = web_sys::Url::revoke_object_url(&url);

                        Ok(())
                    })
                    .await;
                }

                FileSystemCommand::FileRead(key, max_length, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();

                        let read_handle = (if file.read_allowed {
                            to_future::<web_sys::File>(file.file_handle.get_file())
                                .await
                                .ok()
                        } else {
                            None
                        })
                        .ok_or(std::io::Error::new(PermissionDenied, "Attempted to read from file with no read permissions"))?;

                        let blob = read_handle
                            .slice_with_f64_and_f64(
                                file.offset as f64,
                                (file.offset + max_length) as f64,
                            )
                            .map_err(|e| std::io::Error::new(PermissionDenied, format!("Failed to read from file: {}", e.unchecked_into::<js_sys::Error>().to_string())))?;

                        let buffer = to_future::<js_sys::ArrayBuffer>(blob.array_buffer())
                            .await
                            .map_err(|e| std::io::Error::new(PermissionDenied, format!("Failed to convert file contents into array buffer while reading file: {}", e.unchecked_into::<js_sys::Error>().to_string())))?;

                        let u8_array = js_sys::Uint8Array::new(&buffer);
                        let vec = u8_array.to_vec();
                        file.offset += vec.len();
                        Ok(vec)
                    })
                    .await;
                }

                FileSystemCommand::FileWrite(key, vec, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();
                        let write_handle =
                            file.write_handle.as_ref().ok_or(std::io::Error::new(
                                PermissionDenied,
                                "Attempted to write to file with no write permissions",
                            ))?;

                        // We can't use `write_handle.write_with_u8_array()` when shared memory is enabled
                        let u8_array =
                            js_sys::Uint8Array::new(&JsValue::from_f64(vec.len() as f64));
                        u8_array.copy_from(&vec[..]);
                        to_future::<JsValue>(
                            write_handle
                                .seek_with_f64(file.offset as f64)
                                .map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                            "Failed to seek file handle: {}",
                                            e.unchecked_into::<js_sys::Error>().to_string()
                                        ),
                                    )
                                })?,
                        )
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                PermissionDenied,
                                format!(
                                    "Failed to seek file handle: {}",
                                    e.unchecked_into::<js_sys::Error>().to_string()
                                ),
                            )
                        })?;
                        to_future::<JsValue>(
                            write_handle
                                .write_with_buffer_source(&u8_array)
                                .map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                            "Failed to write to file: {}",
                                            e.unchecked_into::<js_sys::Error>().to_string()
                                        ),
                                    )
                                })?,
                        )
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                PermissionDenied,
                                format!(
                                    "Failed to write to file: {}",
                                    e.unchecked_into::<js_sys::Error>().to_string()
                                ),
                            )
                        })?;

                        file.offset += vec.len();
                        Ok(())
                    })
                    .await;
                }

                FileSystemCommand::FileFlush(key, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();

                        // Closing and reopening the handle is the only way to flush
                        if file.write_handle.is_none() {
                            return Err(std::io::Error::new(
                                PermissionDenied,
                                "Attempted to write to file with no write permissions",
                            ));
                        }
                        to_future::<JsValue>(
                            file.write_handle
                                .as_ref()
                                .ok_or(std::io::Error::new(
                                    PermissionDenied,
                                    "Attempted to write to file with no write permissions",
                                ))?
                                .close(),
                        )
                        .await
                        .map_err(|e| {
                            std::io::Error::new(
                                PermissionDenied,
                                format!("Failed to flush file: {}", e.to_string()),
                            )
                        })?;
                        let mut options = web_sys::FileSystemCreateWritableOptions::new();
                        options.keep_existing_data(true);
                        let write_handle =
                            to_future(file.file_handle.create_writable_with_options(&options))
                                .await
                                .map_err(|e| {
                                    std::io::Error::new(
                                        PermissionDenied,
                                        format!(
                                    "Failed to reopen write handle for file after flushing: {}",
                                    e.to_string()
                                ),
                                    )
                                })?;
                        file.write_handle = Some(write_handle);
                        Ok(())
                    })
                    .await;
                }

                FileSystemCommand::FileSeek(key, seek_from, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();
                        let read_handle = (if file.read_allowed {
                            to_future::<web_sys::File>(file.file_handle.get_file())
                                .await
                                .ok()
                        } else {
                            None
                        })
                        .ok_or(std::io::Error::new(
                            PermissionDenied,
                            "Attempted to read from file with no read permissions",
                        ))?;

                        let size = read_handle.size();
                        let new_offset = match seek_from {
                            std::io::SeekFrom::Start(i) => i as i64,
                            std::io::SeekFrom::End(i) => i + size as i64,
                            std::io::SeekFrom::Current(i) => i + file.offset as i64,
                        };
                        if new_offset >= 0 {
                            file.offset = new_offset as usize;
                            Ok(new_offset as u64)
                        } else {
                            Err(std::io::Error::new(
                                InvalidInput,
                                "Tried to seek to negative offset in file",
                            ))
                        }
                    })
                    .await;
                }

                FileSystemCommand::FileSize(key, tx) => {
                    handle_event(tx, async {
                        let file = files.get_mut(key).unwrap();
                        to_future::<web_sys::File>(file.file_handle.get_file())
                            .await
                            .map(|file| file.size() as u64)
                            .map_err(|e| {
                                std::io::Error::new(
                                    PermissionDenied,
                                    format!("Failed to get file size: {}", e.to_string()),
                                )
                            })
                    })
                    .await;
                }

                FileSystemCommand::FileDrop(key, temp_file_name, tx) => {
                    handle_event(tx, async {
                        if files.contains(key) {
                            let file = files.remove(key);
                            // We need to close the write handle to flush any changes that the user
                            // made to the file
                            if let Some(write_handle) = &file.write_handle {
                                let _ = to_future::<JsValue>(write_handle.close()).await;
                            }

                            if let Some(temp_file_name) = temp_file_name {
                                if let Ok(tmp_dir) = get_tmp_dir(&storage).await {
                                    let _ =
                                        to_future::<JsValue>(tmp_dir.remove_entry(&temp_file_name))
                                            .await;
                                }
                            }
                            true
                        } else {
                            false
                        }
                    })
                    .await;
                }
            }
        }
    });
}
