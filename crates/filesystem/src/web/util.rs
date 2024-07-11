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

use rand::Rng;
use wasm_bindgen::prelude::*;

/// Casts a `js_sys::Promise` into a future.
pub(super) async fn to_future<T>(promise: js_sys::Promise) -> std::result::Result<T, js_sys::Error>
where
    T: JsCast,
{
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map(|t| t.unchecked_into())
        .map_err(|e| e.unchecked_into())
}

/// Returns a subdirectory of a given directory given the relative path of the subdirectory.
/// If one of the parent directories along the path does not exist, this will return `None`.
/// Use `get_subdir_create` if you want to create the missing parent directories.
pub(super) async fn get_subdir(
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

/// Returns a subdirectory of a given directory given the relative path of the subdirectory.
/// If one of the parent directories along the path does not exist, this will create the missing
/// directories.
pub(super) async fn get_subdir_create(
    dir: &web_sys::FileSystemDirectoryHandle,
    path_iter: &mut camino::Iter<'_>,
) -> Option<web_sys::FileSystemDirectoryHandle> {
    let mut dir = dir.clone();
    loop {
        let Some(path_element) = path_iter.next() else {
            return Some(dir);
        };
        let mut options = web_sys::FileSystemGetDirectoryOptions::new();
        options.create(true);
        if let Ok(subdir) =
            to_future(dir.get_directory_handle_with_options(path_element, &options)).await
        {
            dir = subdir;
        } else {
            return None;
        }
    }
}

/// Returns a handle to a directory for temporary files in the Origin Private File System.
pub(super) async fn get_tmp_dir(
    storage: &web_sys::StorageManager,
) -> std::io::Result<web_sys::FileSystemDirectoryHandle> {
    let opfs_root = to_future::<web_sys::FileSystemDirectoryHandle>(storage.get_directory())
        .await
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                format!("Failed to get handle to OPFS root: {}", e.to_string()),
            )
        })?;
    let mut iter = camino::Utf8Path::new("astrabit.luminol/tmp").iter();
    get_subdir_create(&opfs_root, &mut iter)
        .await
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Failed to get handle to temporary directory",
        ))
}

/// Generates a random string suitable for use as a unique identifier.
pub(super) fn generate_key() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(42) // This should be enough to avoid collisions
        .map(char::from)
        .collect()
}

/// Wrapper function for handling filesystem events on the worker thread.
/// You can insert logging into this function for debug purposes.
pub(super) async fn handle_event<R>(
    tx: oneshot::Sender<R>,
    f: impl std::future::Future<Output = R>,
) {
    tx.send(f.await).unwrap();
}

fn send<R>(f: impl FnOnce(oneshot::Sender<R>) -> super::FileSystemCommand) -> oneshot::Receiver<R> {
    let (oneshot_tx, oneshot_rx) = oneshot::channel();
    super::worker_channels_or_die().send(f(oneshot_tx));
    oneshot_rx
}

/// Helper function to send a filesystem command from the worker thread to the main thread and then
/// block the worker thread to wait for the result.
pub(super) fn send_and_recv<R>(
    f: impl FnOnce(oneshot::Sender<R>) -> super::FileSystemCommand,
) -> R {
    send(f).recv().unwrap()
}

/// Helper function to send a filesystem command from the worker thread to the main thread and then
/// wait asynchronously on the worker thread for the result.
pub(super) async fn send_and_await<R>(
    f: impl FnOnce(oneshot::Sender<R>) -> super::FileSystemCommand,
) -> R {
    send(f).await.unwrap()
}

/// Helper function to send a filesystem command from the worker thread to the main thread, wait
/// asynchronously for the response, send the response to the receiver returned by this function
/// and then wake up a task.
pub(super) fn send_and_wake<R>(
    cx: &std::task::Context<'_>,
    f: impl FnOnce(oneshot::Sender<R>) -> super::FileSystemCommand,
) -> oneshot::Receiver<R>
where
    R: 'static,
{
    let command_rx = send(f);
    let (task_tx, task_rx) = oneshot::channel();

    let waker = cx.waker().clone();

    wasm_bindgen_futures::spawn_local(async move {
        let response = command_rx.await.unwrap();
        let _ = task_tx.send(response);
        waker.wake();
    });

    task_rx
}
