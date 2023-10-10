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
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/bindings.js")]
extern "C" {
    pub fn worker() -> Option<web_sys::DedicatedWorkerGlobalScope>;
    pub fn performance(worker: &web_sys::DedicatedWorkerGlobalScope) -> web_sys::Performance;
    pub fn filesystem_supported() -> bool;
    #[wasm_bindgen(catch)]
    async fn _show_directory_picker() -> Result<JsValue, JsValue>;
    pub fn dir_values(dir: &web_sys::FileSystemDirectoryHandle) -> js_sys::AsyncIterator;
    async fn _request_permission(handle: &web_sys::FileSystemHandle) -> JsValue;
}

pub async fn show_directory_picker() -> Result<web_sys::FileSystemDirectoryHandle, js_sys::Error> {
    _show_directory_picker()
        .await
        .map(|o| o.unchecked_into())
        .map_err(|e| e.unchecked_into())
}

pub async fn request_permission(handle: &web_sys::FileSystemHandle) -> bool {
    _request_permission(handle).await.is_truthy()
}
