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
#![allow(missing_docs)]
#![allow(unused_variables)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::data::config::RGSSVer;
use crate::UpdateInfo;

use wasm_bindgen::prelude::*;

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    loading_project: RefCell<bool>,
    directory: RefCell<Option<web_sys::FileSystemDirectoryEntry>>,
}

#[async_trait(?Send)]
impl super::filesystem_trait::Filesystem for Filesystem {
    /// Unload the currently loaded project.
    /// Does nothing if none is open.
    fn unload_project(&self) {
        *self.project_path.borrow_mut() = None;
    }

    /// Is there a project loaded?
    fn project_loaded(&self) -> bool {
        self.project_path.borrow().is_some() && !*self.loading_project.borrow()
    }

    /// Get the project path.
    fn project_path(&self) -> Option<PathBuf> {
        self.project_path.borrow().clone()
    }

    /// Get the directory children of a path.
    async fn dir_children(&self, path: impl AsRef<Path>) -> Result<Vec<String>, String> {
        todo!()
    }

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    async fn read_data<T>(&self, path: impl AsRef<Path>) -> Result<T, String> {
        todo!()
    }

    /// Read bytes from a file.
    async fn read_bytes(&self, provided_path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        todo!()
    }

    /// Save some file's data by serializing it with RON.
    async fn save_data(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), String> {
        todo!()
    }

    /// Check if file path exists
    async fn file_exists(&self, path: impl AsRef<Path>) -> bool {
        todo!()
    }

    /// Save all cached files. An alias for [`DataCache::save`];
    async fn save_cached(&self, info: &'static UpdateInfo) -> Result<(), String> {
        todo!()
    }

    /// Try to open a project.
    async fn try_open_project(&self, info: &'static UpdateInfo) -> Result<(), String> {
        let window = web_sys::window().unwrap();

        let dialogue_fn = js_sys::Reflect::get(&window, &JsValue::from_str("showDirectoryPicker"))
            .map_err(|_| "showDirectoryPicker not found")?;
        let dialogue_fn: &js_sys::Function =
            dialogue_fn.dyn_ref().ok_or("could not cast function?")?;

        let future = dialogue_fn
            .call0(&JsValue::NULL)
            .map_err(|_| "failed to call showDirectoryPicker")?;
        let future: js_sys::Promise = future.dyn_into().map_err(|_| "failed to cast future")?;
        let future: wasm_bindgen_futures::JsFuture = future.into();

        let result = future.await.map_err(|_| "awaiting future failed")?;
        let directory: web_sys::FileSystemDirectoryEntry = result
            .dyn_into()
            .map_err(|r| format!("result was not a directory entry {r:?}"))?;

        let path = js_sys::Reflect::get(&window, &JsValue::from_str("name"))
            .map_err(|_| "failed to get path")?;
        let path = path.as_string().ok_or("path was not a string?")?;

        *self.directory.borrow_mut() = Some(directory);
        *self.project_path.borrow_mut() = Some(path.into());

        *self.loading_project.borrow_mut() = true;

        info.data_cache.load(self).await.map_err(|e| {
            *self.project_path.borrow_mut() = None;
            *self.directory.borrow_mut() = None;

            e
        })?;

        *self.loading_project.borrow_mut() = false;

        Ok(())
    }

    /// Create a directory at the specified path.
    async fn create_directory(&self, path: impl AsRef<Path>) -> Result<(), String> {
        todo!()
    }

    /// Try to create a project.
    async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        todo!()
    }
}
