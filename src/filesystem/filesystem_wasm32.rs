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

use parking_lot::Mutex;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::data::data_cache::DataCache;

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_open_project() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    async fn js_read_file(handle: JsValue, _path: String) -> Result<JsValue, JsValue>;
    fn js_filesystem_supported() -> bool;
}

#[derive(Default)]
pub struct Filesystem {
    project_path: Mutex<Option<PathBuf>>,
    handle: Mutex<Option<JsValue>>,
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
        *self.project_path.lock() = None;
    }

    pub fn project_loaded(&self) -> bool {
        self.project_path.lock().is_some()
    }

    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.lock().clone()
    }

    pub fn load_project(&self, handle: JsValue, cache: Arc<DataCache>) -> Result<(), String> {
        *self.project_path.lock() = Some(PathBuf::from(
            js_sys::Reflect::get(&handle, &JsValue::from("name"))
                .unwrap()
                .as_string()
                .unwrap(),
        ));
        *self.handle.lock() = Some(handle);
        cache.load(self).map_err(|e| {
            *self.handle.lock() = None;
            *self.project_path.lock() = None;
            e
        })
    }

    pub fn dir_children(&self, _path: &str) -> Result<fs::ReadDir, String> {
        Err("Not implemented".to_string())
    }

    pub fn bufreader(&self, _path: &str) -> Result<BufReader<File>, String> {
        Err("Not implemented".to_string())
    }

    pub fn read_data<T>(&self, _path: &str) -> ron::error::SpannedResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        Err(ron::error::SpannedError {
            code: ron::error::Error::Eof,
            position: ron::error::Position { line: 0, col: 0 },
        })
    }

    pub fn read_bytes(&self, _path: &str) -> Result<Vec<u8>, String> {
        Err("Not implemented".to_string())
    }

    pub fn save_data<T>(&self, _path: &str, _data: &T) -> Result<(), String>
    where
        T: serde::ser::Serialize,
    {
        Err("Not implemented".to_string())
    }

    pub fn save_cached(&self, data_cache: Arc<DataCache>) -> Result<(), String> {
        data_cache.save(self)
    }

    pub async fn try_open_project(&self, cache: Arc<DataCache>) -> Result<(), String> {
        let handle = js_open_project()
            .await
            .map_err(|_| "No project loaded".to_string())?;

        self.load_project(handle, cache)
    }
}
