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
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/js/bindings.js")]
extern "C" {
    pub fn worker() -> Option<web_sys::DedicatedWorkerGlobalScope>;
    pub fn filesystem_supported() -> bool;
    #[wasm_bindgen(catch)]
    async fn _show_directory_picker() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn _show_file_picker(
        filter_name: &str,
        extensions: &js_sys::Array,
    ) -> Result<JsValue, JsValue>;
    pub fn dir_values(dir: &web_sys::FileSystemDirectoryHandle) -> js_sys::AsyncIterator;
    #[wasm_bindgen(catch)]
    async fn _request_permission(handle: &web_sys::FileSystemHandle) -> Result<JsValue, JsValue>;
    pub fn cross_origin_isolated() -> bool;
}

pub async fn show_directory_picker() -> Result<web_sys::FileSystemDirectoryHandle, js_sys::Error> {
    _show_directory_picker()
        .await
        .map(|o| o.unchecked_into())
        .map_err(|e| e.unchecked_into())
}

pub async fn show_file_picker(
    filter_name: &str,
    extensions: &[impl AsRef<str>],
) -> Result<web_sys::FileSystemFileHandle, js_sys::Error> {
    let array = js_sys::Array::new();
    for extension in extensions {
        array.push(&JsValue::from(format!(".{}", extension.as_ref())));
    }
    _show_file_picker(filter_name, &array)
        .await
        .map(|o| o.unchecked_into())
        .map_err(|e| e.unchecked_into())
}

pub async fn request_permission(handle: &web_sys::FileSystemHandle) -> Result<bool, js_sys::Error> {
    _request_permission(handle)
        .await
        .map(|o| o.is_truthy())
        .map_err(|e| e.unchecked_into())
}
