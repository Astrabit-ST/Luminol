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
use wasm_bindgen_futures::JsFuture;

use crate::data::config::RGSSVer;
use crate::UpdateInfo;

use wasm_bindgen::prelude::*;

use web_sys::{
    File, FileSystemDirectoryHandle, FileSystemFileHandle, FileSystemGetDirectoryOptions,
    FileSystemGetFileOptions, FileSystemHandle, FileSystemWritableFileStream,
};

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
    async fn get_file(&self, path: &Path) -> Result<File, js_sys::Error> {
        let file = self
            .get_parent_dir(path)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)?;

        let promise = file.get_file_handle(path.file_name().unwrap().to_str().unwrap())?;
        let file: FileSystemFileHandle = JsFuture::from(promise).await?.dyn_into()?;

        let promise = FileSystemFileHandle::get_file(&file)?;
        JsFuture::from(promise)
            .await?
            .dyn_into()
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
    }

    async fn get_parent_dir(&self, path: &Path) -> Result<FileSystemDirectoryHandle, JsValue> {
        let mut file = {
            let directory = self.directory.borrow(); // FIXME: USE CELL NOT AWAIT

            directory.clone().ok_or("Project not open".to_string())?
        };

        if let Some(parent) = path.parent() {
            for p in parent.components() {
                let p = p.as_os_str();
                let p = p.to_str().unwrap();

                let promise = file.get_directory_handle(p)?;
                file = JsFuture::from(promise).await?.dyn_into()?;
            }
        }

        Ok(file)
    }

    async fn create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        use super::filesystem_trait::Filesystem;

        self.create_directory("").await?;

        if !self.dir_children(".").await?.is_empty() {
            return Err("Directory not empty".to_string());
        }

        self.create_directory("Data").await?;

        info.data_cache.setup_defaults();
        {
            let mut config = info.data_cache.config();
            let config = config.as_mut().unwrap();
            config.rgss_ver = rgss_ver;
            config.project_name = name;
        }

        self.save_cached(info).await?;

        self.create_directory("Audio").await?;
        self.create_directory("Audio/BGM").await?;
        self.create_directory("Audio/BGS").await?;
        self.create_directory("Audio/SE").await?;
        self.create_directory("Audio/ME").await?;

        self.create_directory("Graphics").await?;
        self.create_directory("Graphics/Animations").await?;
        self.create_directory("Graphics/Autotiles").await?;
        self.create_directory("Graphics/Battlebacks").await?;
        self.create_directory("Graphics/Battlers").await?;
        self.create_directory("Graphics/Characters").await?;
        self.create_directory("Graphics/Fogs").await?;
        self.create_directory("Graphics/Icons").await?;
        self.create_directory("Graphics/Panoramas").await?;
        self.create_directory("Graphics/Pictures").await?;
        self.create_directory("Graphics/Tilesets").await?;
        self.create_directory("Graphics/Titles").await?;
        self.create_directory("Graphics/Transitions").await?;
        self.create_directory("Graphics/Windowskins").await?;

        Ok(())
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
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        let prefix = path.as_ref().file_name().unwrap();
        let prefix = prefix.to_str().unwrap();
        let promise = folder
            .get_directory_handle(prefix)
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;
        let folder: FileSystemDirectoryHandle = JsFuture::from(promise)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?
            .unchecked_into();

        let iter = js_sys::try_iter(&folder).unwrap().unwrap();
        let mut entries = vec![];

        for next in iter {
            let next: js_sys::Promise = next
                .map_err(JsValue::unchecked_into::<js_sys::Error>)
                .map_err(|e| e.to_string().as_string().unwrap())?
                .unchecked_into();
            let next: FileSystemHandle = JsFuture::from(next)
                .await
                .map_err(JsValue::unchecked_into::<js_sys::Error>)
                .map_err(|e| e.to_string().as_string().unwrap())?
                .unchecked_into();

            entries.push(FileSystemHandle::name(&next));
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
        let buf = self.read_bytes(path).await?;

        alox_48::from_bytes(&buf).map_err(|e| e.to_string())
    }

    /// Read bytes from a file.
    async fn read_bytes(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        let file = self
            .get_file(path.as_ref())
            .await
            .map_err(|e| e.to_string().as_string().unwrap())?;

        let buf = JsFuture::from(file.array_buffer());
        let buf = buf
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;
        let buf: js_sys::ArrayBuffer = buf.unchecked_into();
        let buf = js_sys::Uint8Array::new(&buf);

        Ok(buf.to_vec())
    }

    /// Save some file's data by serializing it with RON.
    async fn save_data(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), String> {
        let folder = self.get_parent_dir(path.as_ref()).await.unwrap();

        let mut options = FileSystemGetFileOptions::new();
        options.create(true);
        let promise = folder
            .get_file_handle_with_options(
                path.as_ref().file_name().unwrap().to_str().unwrap(),
                &options,
            )
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        let file: FileSystemFileHandle = JsFuture::from(promise).await.unwrap().unchecked_into();
        let promise = file
            .create_writable()
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        let stream: FileSystemWritableFileStream = JsFuture::from(promise)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?
            .unchecked_into();

        let promise = stream.write_with_buffer_source(&js_sys::Uint8Array::from(data.as_ref()));
        JsFuture::from(promise)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        Ok(())
    }

    /// Check if file path exists
    async fn file_exists(&self, path: impl AsRef<Path>) -> bool {
        self.dir_children(path.as_ref().parent().unwrap())
            .await
            .is_ok_and(|v| {
                v.contains(
                    &path
                        .as_ref()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ) // FIXME: pain
            })
    }

    /// Save all cached files. An alias for [`DataCache::save`];
    async fn save_cached(&self, info: &'static UpdateInfo) -> Result<(), String> {
        info.data_cache.save(self).await
    }

    /// Try to open a project.
    async fn try_open_project(&self, info: &'static UpdateInfo) -> Result<(), String> {
        let window = web_sys::window().unwrap();

        let directory = show_directory_picker(&window)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;
        let directory: FileSystemDirectoryHandle = directory
            .dyn_into()
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

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
        let dir = self
            .get_parent_dir(path.as_ref())
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        let mut options = FileSystemGetDirectoryOptions::new();
        options.create(true);

        let promise = dir
            .get_directory_handle_with_options(
                path.as_ref().file_name().unwrap().to_str().unwrap(),
                &options,
            )
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        JsFuture::from(promise)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        Ok(())
    }

    /// Try to create a project.
    async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        let window = web_sys::window().unwrap();

        let directory = show_directory_picker(&window)
            .await
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;
        let directory: FileSystemDirectoryHandle = directory
            .dyn_into()
            .map_err(JsValue::unchecked_into::<js_sys::Error>)
            .map_err(|e| e.to_string().as_string().unwrap())?;

        *self.project_path.borrow_mut() = Some(directory.name().into());
        *self.directory.borrow_mut() = Some(directory);

        *self.loading_project.borrow_mut() = true;

        self.create_project(name, info, rgss_ver).await?;

        *self.loading_project.borrow_mut() = false;

        Ok(())
    }

    /// Try to open a project.
    async fn spawn_project_file_picker(&self, info: &'static UpdateInfo) -> Result<(), String> {
        self.try_open_project(info).await
    }
}
