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

use crate::data::config::RGSSVer;
use js_sys::Uint8Array;
use std::cell::RefCell;
use std::io::Cursor;
use std::path::PathBuf;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use crate::{data::data_cache::DataCache, UpdateInfo};

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    #[wasm_bindgen(catch)]
    async fn js_open_project() -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_read_file(path: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_read_bytes(path: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_dir_children(path: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_save_data(path: JsValue, data: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_create_directory(path: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch)]
    async fn js_create_project_dir(path: JsValue) -> Result<JsValue, JsValue>;

    fn js_filesystem_supported() -> bool;
}

pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
}

impl Default for Filesystem {
    fn default() -> Self {
        if !js_filesystem_supported() {
            panic!("Filesystem not supported on this browser");
        }
        Self {
            project_path: RefCell::new(None),
        }
    }
}

impl Filesystem {
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

    pub async fn dir_children(&self, path: &str) -> Result<Vec<String>, String> {
        js_dir_children(JsValue::from_str(path))
            .await
            .map(|ref children| {
                js_sys::Array::from(children)
                    .iter()
                    .map(|child| child.as_string().unwrap())
                    .collect()
            })
            .map_err(|s| format!("JS Error {:#?}", s))
    }

    pub async fn bufreader(&self, path: &str) -> Result<Cursor<Vec<u8>>, String> {
        Ok(Cursor::new(self.read_bytes(path).await?))
    }

    pub async fn read_data<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let str = js_read_file(JsValue::from_str(&format!("Data_RON/{}", path)))
            .await
            .map(|s| s.as_string().unwrap())
            .map_err(|s| format!("JS Error {:#?}", s))?;

        ron::from_str(&str).map_err(|e| e.to_string())
    }

    pub async fn read_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        js_read_bytes(JsValue::from_str(path))
            .await
            .map(|bytes| js_sys::Uint8Array::try_from(bytes).unwrap().to_vec())
            .map_err(|s| format!("JS Error {:#?}", s))
    }

    pub async fn save_data(&self, path: &str, data: &str) -> Result<(), String> {
        js_save_data(
            JsValue::from_str(&format!("Data_RON/{}", path)),
            JsValue::from_str(data),
        )
        .await
        .map(|_| ())
        .map_err(|s| format!("JS Error {:#?}", s))
    }

    /// Save data at a specific directory
    pub async fn save_data_at(&self, path: &str, data: &str) -> Result<(), String> {
        js_save_data(JsValue::from_str(path), JsValue::from_str(data))
            .await
            .map(|_| ())
            .map_err(|s| format!("JS Error {:#?}", s))
    }

    /// Save some bytes
    pub async fn save_bytes_at(&self, path: &str, data: Vec<u8>) -> Result<(), String> {
        js_save_data(
            JsValue::from_str(path),
            Uint8Array::from(data.as_slice()).into(),
        )
        .await
        .map(|_| ())
        .map_err(|s| format!("JS Error {:#?}", s))
    }

    pub async fn create_directory(&self, path: &str) -> Result<(), String> {
        js_create_directory(JsValue::from_str(path))
            .await
            .map(|_| ())
            .map_err(|s| format!("JS Error {:#?}", s))
    }

    /// Check if file path exists
    pub async fn file_exists(&self, path: &str) -> bool {
        let split: Vec<_> = path.split('/').map(|s| s.to_string()).collect();

        self.dir_children(&split[..(split.len() - 2)].join("/"))
            .await
            .is_ok_and(|v| v.contains(split.last().unwrap()))
    }

    pub async fn save_cached(&self, data_cache: &'static DataCache) -> Result<(), String> {
        data_cache.save(self).await
    }

    pub async fn try_open_project(&self, info: &'static UpdateInfo) -> Result<(), String> {
        let handle = js_open_project()
            .await
            .map_err(|_| "Cancelled loading project".to_string())?;

        self.load_project(handle, &info.data_cache).await
    }

    async fn create_project(
        &self,
        name: String,
        path: PathBuf,
        cache: &'static DataCache,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        js_create_project_dir(JsValue::from_str(&name))
            .await
            .map(|_| ())
            .map_err(|s| format!("JS Error {:#?}", s))?;
        *self.project_path.borrow_mut() = Some(path);

        if !self.dir_children("").await?.is_empty() {
            return Err("Directory not empty".to_string());
        }

        self.create_directory("Data_RON").await?;

        cache.setup_defaults();
        {
            let mut config = cache.config();
            let config = config.as_mut().unwrap();
            config.rgss_ver = rgss_ver;
            config.project_name = name;
        }

        self.save_cached(cache).await?;

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

    /// Try to create a project.
    pub async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        let handle = js_open_project()
            .await
            .map_err(|_| "Cancelled opening a folder".to_string())?;

        let path = PathBuf::from(handle.as_string().unwrap());

        self.create_project(name.clone(), path, &info.data_cache, rgss_ver)
            .await
            .map_err(|e| {
                *self.project_path.borrow_mut() = None;
                e
            })?;

        {
            let projects = &mut info.saved_state.borrow_mut().recent_projects;

            let path = self.project_path().unwrap().display().to_string();
            *projects = projects
                .iter()
                .filter_map(|p| if *p != path { Some(p.clone()) } else { None })
                .collect();
            projects.push_front(path);
            projects.truncate(10);
        }

        self.save_data_at(&format!("{name}.lumproj"), "").await
    }
}
