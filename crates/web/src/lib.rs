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

#[cfg(target_arch = "wasm32")]
pub mod bindings;

#[cfg(target_arch = "wasm32")]
use std::future::IntoFuture;

#[cfg(target_arch = "wasm32")]
pub use indexed_db_futures::prelude::IdbTransactionMode;

#[cfg(target_arch = "wasm32")]
pub use indexed_db_futures::IdbQuerySource;

#[cfg(target_arch = "wasm32")]
use indexed_db_futures::prelude::*;

#[cfg(target_arch = "wasm32")]
/// Helper function for performing IndexedDB operations on an `IdbObjectStore` with a given
/// `IdbTransactionMode`.
pub async fn idb<R>(
    store_name: &str,
    mode: IdbTransactionMode,
    f: impl FnOnce(IdbObjectStore<'_>) -> std::result::Result<R, web_sys::DomException>,
) -> std::result::Result<R, web_sys::DomException> {
    let mut db_req = IdbDatabase::open_u32("astrabit.luminol", 2)?;

    db_req.set_on_upgrade_needed(Some(|e: &IdbVersionChangeEvent| {
        if !e
            .db()
            .object_store_names()
            .any(|n| n == "filesystem.dir_handles")
        {
            e.db().create_object_store("filesystem.dir_handles")?;
        }

        if !e.db().object_store_names().any(|n| n == "eframe.storage") {
            e.db().create_object_store("eframe.storage")?;
        }

        Ok(())
    }));

    let db = db_req.into_future().await?;
    let tx = db.transaction_on_one_with_mode(store_name, mode)?;
    let store = tx.object_store(store_name)?;
    let r = f(store);
    tx.await.into_result()?;
    r
}
