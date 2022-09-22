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
use std::fs::{self, File};
use std::io::BufReader;
use std::path::PathBuf;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use super::data_cache::DataCache;

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    fn js_open_project() -> JsValue;
    fn js_filesystem_supported() -> bool;
}

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    handle: RefCell<Option<JsValue>>,
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
            project_path: RefCell::new(None),
            handle: RefCell::new(None),
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

    pub fn load_project(&self, path: PathBuf, cache: &DataCache) {
        *self.project_path.borrow_mut() = Some(path);
        cache.load(self);
    }

    pub fn read_data<T>(&self, _path: &str) -> Result<T, &str>
    where
        T: serde::de::DeserializeOwned,
    {
        todo!()
    }

    pub fn read_bytes(&self, _path: &str) -> Result<Vec<u8>, std::io::Error> {
        todo!()
    }

    pub fn dir_children(&self, path: &str) -> fs::ReadDir {
        todo!()
    }

    pub fn bufreader(&self, path: &str) -> BufReader<File> {
        todo!()
    }

    pub fn save_data<T>(&self, _path: &str, _data: &T) -> Result<(), ()>
    where
        T: serde::ser::Serialize,
    {
        todo!()
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save(self);
    }

    pub fn try_open_project(&self, cache: &DataCache) {
        todo!()
    }
}
