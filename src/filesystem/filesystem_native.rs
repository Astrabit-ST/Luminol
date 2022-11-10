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

use crate::data::config::RGSSVer;
use crate::data::data_cache::DataCache;
use crate::UpdateInfo;

/// Native filesystem implementation.
#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    loading_project: RefCell<bool>,
}

impl Filesystem {
    /// Unload the currently loaded project.
    /// Does nothing if none is open.
    pub fn unload_project(&self) {
        *self.project_path.borrow_mut() = None;
    }

    /// Is there a project loaded?
    pub fn project_loaded(&self) -> bool {
        self.project_path.borrow().is_some() && !*self.loading_project.borrow()
    }

    /// Get the project path.
    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.borrow().clone()
    }

    /// Load a project and setup the Data Cache.
    pub async fn load_project(
        &self,
        path: PathBuf,
        cache: &'static DataCache,
    ) -> Result<(), String> {
        *self.project_path.borrow_mut() = Some(path);

        *self.loading_project.borrow_mut() = true;
        let result = cache.load(self).await.map_err(|e| {
            *self.project_path.borrow_mut() = None;
            e
        });
        *self.loading_project.borrow_mut() = false;

        result
    }

    /// Get the directory children of a path.
    pub async fn dir_children(&self, path: &str) -> Result<Vec<String>, String> {
        // I am too lazy to make this actually async.
        // It'd take an external library or some hacking that I'm not up for currently.
        std::fs::read_dir(
            self.project_path
                .borrow()
                .as_ref()
                .ok_or_else(|| "Project not open".to_string())?
                .join(path),
        )
        .map_err(|e| e.to_string())
        .map(|rd| {
            rd.into_iter()
                .map(|e| e.unwrap().file_name().into_string().unwrap())
                .collect()
        })
    }

    /// Aquire a Cursor to a file.
    /// FIXME: Rename
    pub async fn bufreader(&self, path: &str) -> Result<Cursor<Vec<u8>>, String> {
        Ok(Cursor::new(self.read_bytes(path).await?))
    }

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    pub async fn read_data<T>(&self, path: &str) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join("Data_RON")
            .join(path);

        let data = async_fs::read_to_string(path)
            .await
            .map_err(|e| e.to_string())?;
        ron::from_str(&data).map_err(|e| e.to_string())
    }

    /// Read bytes from a file.
    pub async fn read_bytes(&self, provided_path: &str) -> Result<Vec<u8>, String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(provided_path);
        async_fs::read(path)
            .await
            .map_err(|e| format!("Loading {provided_path}: {e}"))
    }

    /// Save some file's data by serializing it with RON.
    pub async fn save_data(&self, path: &str, data: &str) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join("Data_RON")
            .join(path);

        async_fs::write(path, data).await.map_err(|e| e.to_string())
    }

    /// Save some bytes
    pub async fn save_bytes_at(&self, path: &str, data: Vec<u8>) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        async_fs::write(path, data).await.map_err(|e| e.to_string())
    }

    /// Save data at a specific directory
    pub async fn save_data_at(&self, path: &str, data: &str) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        async_fs::write(path, data).await.map_err(|e| e.to_string())
    }

    /// Check if file path exists
    pub async fn file_exists(&self, path: &str) -> bool {
        let path = self.project_path.borrow().as_ref().unwrap().join(path);

        async_fs::metadata(path).await.is_ok()
    }

    /// Save all cached files. An alias for [`DataCache::save`];
    pub async fn save_cached(&self, data_cache: &'static DataCache) -> Result<(), String> {
        data_cache.save(self).await
    }

    /// Try to open a project.
    pub async fn try_open_project(&self, info: &'static UpdateInfo) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "lumproj"])
            .pick_file()
            .await
        {
            let mut path = path.path().to_path_buf();

            path.pop(); // Pop off filename
            self.load_project(path, &info.data_cache).await?;

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

            Ok(())
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    /// Create a directory at the specified path.
    pub async fn create_directory(&self, path: &str) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        async_fs::create_dir(path).await.map_err(|e| e.to_string())
    }

    async fn create_project(
        &self,
        name: String,
        path: PathBuf,
        cache: &'static DataCache,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        *self.project_path.borrow_mut() = Some(path);
        self.create_directory("").await?;

        if !self.dir_children(".").await?.is_empty() {
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
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path = path.path().to_path_buf().join(name.clone());

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
        } else {
            Err("Cancelled opening a folder".to_owned())
        }
    }
}
