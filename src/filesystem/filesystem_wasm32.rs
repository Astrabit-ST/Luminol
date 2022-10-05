// Copyright (C) 2022 Lily Lyons
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

use std::cell::RefCell;
use std::io::Cursor;
use std::path::PathBuf;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::data::data_cache::DataCache;

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_open_project() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn js_read_file(path: JsValue) -> Result<JsValue, JsValue>;
    fn js_filesystem_supported() -> bool;
}

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
}

impl Filesystem {
    pub fn new() -> Self {
        if !js_filesystem_supported() {
            rfd::MessageDialog::new()
                .set_description("Filesystem not supported on this browser")
                .show();
            panic!("Filesystem not supported on this browser");
        }
        Self {
            ..Default::default()
        }
    }

    pub fn unload_project(&self) {
        *self.project_path.borrow_mut() = None;
    }

    pub fn project_loaded(&self) -> bool {
        self.project_path.borrow().is_some()
    }

    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.borrow().clone()
    }

    pub async fn load_project(
        &self,
        path: JsValue,
        cache: &'static DataCache,
    ) -> Result<(), String> {
        *self.project_path.borrow_mut() = Some(PathBuf::from(path.as_string().unwrap()));
        cache.load(self).await.map_err(|e| {
            *self.project_path.borrow_mut() = None;
            e
        })
    }

    pub async fn dir_children(&self, _path: &str) -> Result<Vec<String>, String> {
        Err("Not implemented".to_string())
    }

    pub async fn bufreader(&self, _path: &str) -> Result<Cursor<Vec<u8>>, String> {
        Err("Not implemented".to_string())
    }

    pub async fn read_data<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let str = js_read_file(JsValue::from_str(&format!("Data_RON/{}", path)))
            .await
            .map(|s| s.as_string().unwrap())
            .map_err(|s| "JS error".to_string())?;

        ron::from_str(&str).map_err(|e| e.to_string())
    }

    pub async fn read_bytes(&self, _path: &str) -> Result<Vec<u8>, String> {
        Err("Not implemented".to_string())
    }

    pub async fn save_data<T>(&self, _path: &str, _data: &T) -> Result<(), String>
    where
        T: serde::ser::Serialize,
    {
        Err("Not implemented".to_string())
    }

    pub async fn save_cached(&self, data_cache: &'static DataCache) -> Result<(), String> {
        data_cache.save(self).await
    }

    pub async fn try_open_project(&self, cache: &'static DataCache) -> Result<(), String> {
        let handle = js_open_project()
            .await
            .map_err(|_| "No project loaded".to_string())?;

        self.load_project(handle, cache).await
    }
}
