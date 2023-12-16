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

use indexed_db_futures::prelude::*;
use rand::Rng;
use std::future::IntoFuture;
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

/// Generates a random string suitable for use as a unique identifier.
pub(super) fn generate_key() -> String {
    rand::thread_rng()
        .sample_iter(rand::distributions::Alphanumeric)
        .take(42) // This should be enough to avoid collisions
        .map(char::from)
        .collect()
}

/// Helper function for performing IndexedDB operations on an `IdbObjectStore` with a given
/// `IdbTransactionMode`.
pub(super) async fn idb<R>(
    mode: IdbTransactionMode,
    f: impl FnOnce(IdbObjectStore<'_>) -> std::result::Result<R, web_sys::DomException>,
) -> std::result::Result<R, web_sys::DomException> {
    let mut db_req = IdbDatabase::open_u32("astrabit.luminol", 1)?;

    // Create store for our directory handles if it doesn't exist
    db_req.set_on_upgrade_needed(Some(|e: &IdbVersionChangeEvent| {
        if e.db()
            .object_store_names()
            .find(|n| n == "filesystem.dir_handles")
            .is_none()
        {
            e.db().create_object_store("filesystem.dir_handles")?;
        }
        Ok(())
    }));

    let db = db_req.into_future().await?;
    let tx = db.transaction_on_one_with_mode("filesystem.dir_handles", mode)?;
    let store = tx.object_store("filesystem.dir_handles")?;
    let r = f(store);
    tx.await.into_result()?;
    r
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
