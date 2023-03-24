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
use std::path::{Path, PathBuf};

use crate::data::config::RGSSVer;
use crate::UpdateInfo;
use async_trait::async_trait;

/// Native filesystem implementation.
#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    loading_project: RefCell<bool>,
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

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    async fn read_data<T>(&self, path: impl AsRef<Path>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        let data = async_fs::read(&path).await.map_err(|e| e.to_string())?;

        // let de = &mut alox_48::Deserializer::new(&data).unwrap();
        // let result = serde_path_to_error::deserialize(de);
        //
        // result.map_err(|e| format!("{}: {:?}", e.path(), e.inner()))
        alox_48::from_bytes(&data).map_err(|e| format!("Loading {path:?}: {e}"))
    }

    /// Read bytes from a file.
    async fn read_bytes(&self, provided_path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(provided_path);
        async_fs::read(&path)
            .await
            .map_err(|e| format!("Loading {path:?}: {e}"))
    }

    async fn save_data(
        &self,
        path: impl AsRef<Path>,
        data: impl AsRef<[u8]>,
    ) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        async_fs::write(path, data).await.map_err(|e| e.to_string())
    }

    /// Check if file path exists
    async fn path_exists(&self, path: impl AsRef<Path>) -> bool {
        let path = self.project_path.borrow().as_ref().unwrap().join(path);

        async_fs::metadata(path).await.is_ok()
    }

    /// Save all cached files. An alias for [`DataCache::save`];
    async fn save_cached(&self, info: &'static UpdateInfo) -> Result<(), String> {
        info.data_cache.save(self).await
    }

    async fn try_open_project(
        &self,
        info: &'static UpdateInfo,
        path: impl ToString,
    ) -> Result<(), String> {
        let mut path = PathBuf::from(path.to_string());
        let original_path = path.clone().to_string_lossy().to_string();

        path.pop(); // Pop off filename

        *self.project_path.borrow_mut() = Some(path);

        *self.loading_project.borrow_mut() = true;

        info.data_cache.load(self).await.map_err(|e| {
            *self.project_path.borrow_mut() = None;
            e
        })?;

        *self.loading_project.borrow_mut() = false;

        {
            let projects = &mut info.saved_state.borrow_mut().recent_projects;

            *projects = projects
                .iter()
                .filter_map(|p| {
                    if *p == original_path {
                        None
                    } else {
                        Some(p.clone())
                    }
                })
                .collect();
            projects.push_front(original_path);
            projects.truncate(10);
        }

        Ok(())
    }

    /// Try to open a project.
    async fn spawn_project_file_picker(&self, info: &'static UpdateInfo) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "lumproj"])
            .pick_file()
            .await
        {
            self.try_open_project(info, path.path().to_str().unwrap())
                .await
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    /// Create a directory at the specified path.
    async fn create_directory(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        async_fs::create_dir(path).await.map_err(|e| e.to_string())
    }

    /// Try to create a project.
    async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path = path.path().to_path_buf().join(name.clone());

            self.create_project(name.clone(), path, info, rgss_ver)
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

            self.save_data(format!("{name}.lumproj"), "").await
        } else {
            Err("Cancelled opening a folder".to_owned())
        }
    }
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
