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

use async_trait::async_trait;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use crate::data::config::RGSSVer;
use crate::UpdateInfo;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    fn js_filesystem_supported() -> bool;

    #[wasm_bindgen(js_name = tryOpenFolder, catch)]
    async fn js_try_open_folder() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = readFile, catch)]
    async fn js_read_file(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = writeFile, catch)]
    async fn js_write_file(path: &str, data: js_sys::Uint8Array) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = dirChildren, catch)]
    async fn js_dir_children(path: &str) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = createDir, catch)]
    async fn js_create_dir(path: &str) -> Result<JsValue, JsValue>;
}

async fn try_open_folder() -> Result<String, js_sys::Error> {
    let path = js_try_open_folder()
        .await
        .map_err(JsValue::unchecked_into::<js_sys::Error>)?;

    Ok(path.as_string().unwrap())
}

async fn read_file(path: &str) -> Result<Vec<u8>, js_sys::Error> {
    let data = js_read_file(path)
        .await
        .map_err(JsValue::unchecked_into::<js_sys::Error>)?;
    let data: js_sys::Uint8Array = data.unchecked_into();

    Ok(data.to_vec())
}

async fn write_file(path: &str, data: &[u8]) -> Result<(), js_sys::Error> {
    let buf = js_sys::Uint8Array::from(data);
    js_write_file(path, buf)
        .await
        .map_err(JsValue::unchecked_into::<js_sys::Error>)?;

    Ok(())
}

async fn dir_children(path: &str) -> Result<Vec<String>, js_sys::Error> {
    let children = js_dir_children(path)
        .await
        .map_err(JsValue::unchecked_into::<js_sys::Error>)?;
    let children: js_sys::Array = children.unchecked_into();

    Ok(children.iter().map(|v| v.as_string().unwrap()).collect())
}

async fn create_dir(path: &str) -> Result<(), js_sys::Error> {
    js_create_dir(path)
        .await
        .map_err(JsValue::unchecked_into::<js_sys::Error>)?;

    Ok(())
}

macro_rules! gaurd_supported {
    () => {
        if !js_filesystem_supported() {
            return Err("Filesystem Access API is not supported in your browser.".to_string());
        }
    };
}

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    loading_project: RefCell<bool>,
}

impl Filesystem {
    async fn create_project(
        &self,
        name: String,
        path: PathBuf,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        use super::filesystem_trait::Filesystem;

        *self.project_path.borrow_mut() = Some(path);
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
        let children = dir_children(path.as_ref().to_str().unwrap())
            .await
            .map_err(|e| e.to_string())?;

        Ok(children)
    }

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    async fn read_data<T>(&self, path: impl AsRef<Path>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let data = self.read_bytes(path).await?;
        let data = alox_48::from_bytes(&data).map_err(|e| e.to_string())?;

        Ok(data)
    }

    /// Read bytes from a file.
    async fn read_bytes(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        let data = read_file(path.as_ref().to_str().unwrap())
            .await
            .map_err(|e| e.to_string())?;

        Ok(data)
    }

    /// Save some bytes to a file.
    async fn save_data(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), String> {
        write_file(path.as_ref().to_str().unwrap(), data.as_ref())
            .await
            .map_err(|e| e.to_string())?;

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
        gaurd_supported!();

        let path = try_open_folder().await.map_err(|e| e.to_string())?;
        self.project_path.replace(Some(PathBuf::from(path)));
        self.loading_project.replace(true);

        info.data_cache.load(self).await.map_err(|e| {
            self.project_path.replace(None);
            self.loading_project.replace(false);
            e
        })?;

        self.loading_project.replace(false);

        Ok(())
    }

    /// Create a directory at the specified path.
    async fn create_directory(&self, path: impl AsRef<Path>) -> Result<(), String> {
        create_dir(path.as_ref().to_str().unwrap())
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Try to create a project.
    async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        gaurd_supported!();

        let path = try_open_folder().await.map_err(|e| e.to_string())?;

        self.create_project(name, path.into(), info, rgss_ver)
            .await
            .map_err(|e| {
                self.project_path.replace(None);
                self.loading_project.replace(false);
                e
            })?;

        Ok(())
    }

    /// Try to open a project.
    async fn spawn_project_file_picker(&self, info: &'static UpdateInfo) -> Result<(), String> {
        self.try_open_project(info).await
    }
}
