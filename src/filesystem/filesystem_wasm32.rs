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

use std::borrow::Borrow;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use js_sys::JsString;

use crate::data::config::RGSSVer;
use crate::UpdateInfo;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[derive(Clone, Debug)]
    type FileSystemHandle;

    #[wasm_bindgen(method, getter)]
    fn name(this: &FileSystemHandle) -> String;

    #[wasm_bindgen(extends = FileSystemHandle)]
    #[derive(Clone, Debug)]
    type FileSystemFileHandle;

    #[wasm_bindgen(method, js_name = getFile)]
    async fn get_file(this: &FileSystemFileHandle) -> JsValue;

    #[wasm_bindgen(extends = FileSystemHandle)]
    #[derive(Clone, Debug)]
    type FileSystemDirectoryHandle;

    #[wasm_bindgen(method, js_name = getFileHandle, catch)]
    async fn get_file_handle(
        this: &FileSystemDirectoryHandle,
        path: JsString,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method, js_name = getDirectoryHandle, catch)]
    async fn get_directory_handle(
        this: &FileSystemDirectoryHandle,
        path: JsString,
    ) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(method)]
    fn entries(this: &FileSystemDirectoryHandle) -> FileSystemDirectoryIterator;

    type FileSystemDirectoryIterator;

    #[wasm_bindgen(method)]
    async fn next(this: &FileSystemDirectoryIterator) -> JsValue;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_class = Window, js_name = showDirectoryPicker)]
    #[wasm_bindgen(catch)]
    async fn show_directory_picker(this: &web_sys::Window) -> Result<JsValue, JsValue>;
}

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    loading_project: RefCell<bool>,
    directory: RefCell<Option<FileSystemDirectoryHandle>>, // FIXME: Use cell
}

impl Filesystem {
    async fn get_file(&self, path: &Path) -> Result<web_sys::File, JsValue> {
        let file = self.get_parent_dir(path).await?;

        let file: FileSystemFileHandle = file
            .get_file_handle(JsString::from(
                path.file_name().unwrap().to_string_lossy().borrow(),
            ))
            .await?
            .dyn_into()?;

        file.get_file().await.dyn_into()
    }

    async fn get_parent_dir(&self, path: &Path) -> Result<FileSystemDirectoryHandle, JsValue> {
        let mut file = {
            let directory = self.directory.borrow(); // FIXME: USE CELL NOT AWAIT

            directory.clone().ok_or("Project not open".to_string())?
        };

        if let Some(parent) = path.parent() {
            for p in parent.components() {
                let p = p.as_os_str();
                let p = p.to_string_lossy();

                file = file
                    .get_directory_handle(JsString::from(p.borrow()))
                    .await?
                    .dyn_into()?;
            }
        }

        Ok(file)
    }
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
        let folder = self
            .get_parent_dir(path.as_ref())
            .await
            .map_err(|e| format!("error getting_directories {e:?}"))?;

        let prefix = path.as_ref().file_name().unwrap();
        let prefix = prefix.to_string_lossy();
        let folder = folder
            .get_directory_handle(JsString::from(prefix.borrow()))
            .await
            .unwrap();
        let folder: FileSystemDirectoryHandle = folder.dyn_into().unwrap();

        let iter = folder.entries();
        let mut entries = vec![];

        loop {
            let next = iter.next().await;

            if js_sys::Reflect::get(&next, &"done".into())
                .unwrap()
                .is_truthy()
            {
                break;
            }

            let value: js_sys::Array = js_sys::Reflect::get(&next, &"value".into())
                .unwrap()
                .dyn_into()
                .unwrap();

            entries.push(value.get(0).as_string().unwrap());
        }

        Ok(entries)
    }

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    async fn read_data<T>(&self, path: impl AsRef<Path>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let file = self
            .get_file(path.as_ref())
            .await
            .map_err(|e| format!("js error {e:?}"))?;

        let buf: wasm_bindgen_futures::JsFuture = file.array_buffer().into();
        let buf = buf
            .await
            .map_err(|e| format!("i am rolling in my grave {e:?}"))?;
        let buf: js_sys::ArrayBuffer = buf
            .dyn_into()
            .map_err(|e| format!("fuck javascript. {e:?}"))?;
        let buf = js_sys::Uint8Array::new(&buf);
        let buf = buf.to_vec();

        alox_48::from_bytes(&buf).map_err(|e| e.to_string())
    }

    /// Read bytes from a file.
    async fn read_bytes(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        let file = self
            .get_file(path.as_ref())
            .await
            .map_err(|e| format!("js error {e:?}"))?;

        let buf: wasm_bindgen_futures::JsFuture = file.array_buffer().into();
        let buf = buf
            .await
            .map_err(|e| format!("i am rolling in my grave {e:?}"))?;
        let buf: js_sys::ArrayBuffer = buf
            .dyn_into()
            .map_err(|e| format!("fuck javascript. {e:?}"))?;
        let buf = js_sys::Uint8Array::new(&buf);

        Ok(buf.to_vec())
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

        let directory = show_directory_picker(&window).await.expect("???");
        let directory: FileSystemDirectoryHandle = directory.dyn_into().expect("????");

        *self.project_path.borrow_mut() = Some(directory.name().into());
        *self.directory.borrow_mut() = Some(directory);

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
