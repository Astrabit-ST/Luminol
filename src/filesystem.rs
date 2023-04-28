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

pub use crate::prelude::*;
use std::sync::atomic::{AtomicBool, Ordering};

/// Native filesystem implementation.
#[derive(Default)]
pub struct Filesystem {
    project_path: RwLock<Option<PathBuf>>,
    loading_project: AtomicBool,
}

impl Filesystem {
    /// Unload the currently loaded project.
    /// Does nothing if none is open.
    pub fn unload_project(&self) {
        *self.project_path.write() = None;
    }

    /// Is there a project loaded?
    pub fn project_loaded(&self) -> bool {
        self.project_path.read().is_some() && !self.loading_project.load(Ordering::Relaxed)
    }

    /// Get the project path.
    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.read().clone()
    }

    /// Get the directory children of a path.
    pub fn dir_children(&self, path: impl AsRef<Path>) -> Result<Vec<String>, String> {
        // I am too lazy to make this actually .
        // It'd take an external library or some hacking that I'm not up for currently.
        std::fs::read_dir(
            self.project_path
                .read()
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
    pub fn read_data<T>(&self, path: impl AsRef<Path>) -> Result<T, String>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .read()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        let data = std::fs::read(&path).map_err(|e| e.to_string())?;

        // let de = &mut alox_48::Deserializer::new(&data).unwrap();
        // let result = serde_path_to_error::deserialize(de);
        //
        // result.map_err(|e| format!("{}: {:?}", e.path(), e.inner()))
        alox_48::from_bytes(&data).map_err(|e| format!("Loading {path:?}: {e}"))
    }

    /// Read bytes from a file.
    pub fn read_bytes(&self, provided_path: impl AsRef<Path>) -> Result<Vec<u8>, String> {
        let path = self
            .project_path
            .read()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(provided_path);
        std::fs::read(&path).map_err(|e| format!("Loading {path:?}: {e}"))
    }

    pub fn reader(&self, provided_path: impl AsRef<Path>) -> Result<std::fs::File, String> {
        let path = self
            .project_path
            .read()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(provided_path);
        std::fs::File::open(&path).map_err(|e| format!("Loading {path:?}: {e}"))
    }

    pub fn save_data(&self, path: impl AsRef<Path>, data: impl AsRef<[u8]>) -> Result<(), String> {
        let path = self
            .project_path
            .read()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        std::fs::write(path, data).map_err(|e| e.to_string())
    }

    /// Check if file path exists
    pub fn path_exists(&self, path: impl AsRef<Path>) -> bool {
        let path = self.project_path.read().as_ref().unwrap().join(path);

        std::fs::metadata(path).is_ok()
    }

    /// Save all cached files. An alias for [`DataCache::save`];
    pub fn save_cached(&self) -> Result<(), String> {
        state!().data_cache.save(self)
    }

    pub fn try_open_project(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let mut path = path.as_ref().to_path_buf();
        let original_path = path.to_string_lossy().to_string();

        path.pop(); // Pop off filename

        *self.project_path.write() = Some(path);
        self.loading_project.store(true, Ordering::Relaxed);

        state!().data_cache.load().map_err(|e| {
            *self.project_path.write() = None;
            self.loading_project.store(false, Ordering::Relaxed);
            e
        })?;

        self.loading_project.store(false, Ordering::Relaxed);

        {
            let projects = &mut state!().saved_state.borrow_mut().recent_projects;

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

        std::env::set_current_dir(self.project_path().unwrap()).map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Try to open a project.
    pub async fn spawn_project_file_picker(&self) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default()
            .add_filter("project file", &["rxproj", "lumproj"])
            .pick_file()
            .await
        {
            self.try_open_project(path.path())
        } else {
            Err("Cancelled loading project".to_string())
        }
    }

    /// Create a directory at the specified path.
    pub fn create_directory(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let path = self
            .project_path
            .read()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);

        std::fs::create_dir(path).map_err(|e| e.to_string())
    }

    /// Try to create a project.
    pub async fn try_create_project(&self, name: String, rgss_ver: RGSSVer) -> Result<(), String> {
        if let Some(path) = rfd::AsyncFileDialog::default().pick_folder().await {
            let path: PathBuf = path.into();
            let path = path.join(name.clone());

            self.create_project(name.clone(), path, rgss_ver)
                .map_err(|e| {
                    *self.project_path.write() = None;
                    e
                })?;

            {
                let projects = &mut state!().saved_state.borrow_mut().recent_projects;

                let path = self.project_path().unwrap().display().to_string();
                *projects = projects
                    .iter()
                    .filter_map(|p| if *p != path { Some(p.clone()) } else { None })
                    .collect();
                projects.push_front(path);
                projects.truncate(10);
            }

            self.save_data(format!("{name}.lumproj"), "")
        } else {
            Err("Cancelled opening a folder".to_owned())
        }
    }

    pub fn create_project(
        &self,
        name: String,
        path: PathBuf,
        rgss_ver: RGSSVer,
    ) -> Result<(), String> {
        *self.project_path.write() = Some(path);
        self.create_directory("")?;

        if !self.dir_children(".")?.is_empty() {
            return Err("Directory not empty".to_string());
        }

        self.create_directory("Data")?;

        state!().data_cache.setup_defaults();
        {
            let mut config = state!().data_cache.config();
            config.rgss_ver = rgss_ver;
            config.project_name = name;
        }

        self.save_cached()?;

        self.create_directory("Audio")?;
        self.create_directory("Audio/BGM")?;
        self.create_directory("Audio/BGS")?;
        self.create_directory("Audio/SE")?;
        self.create_directory("Audio/ME")?;

        self.create_directory("Graphics")?;
        self.create_directory("Graphics/Animations")?;
        self.create_directory("Graphics/Autotiles")?;
        self.create_directory("Graphics/Battlebacks")?;
        self.create_directory("Graphics/Battlers")?;
        self.create_directory("Graphics/Characters")?;
        self.create_directory("Graphics/Fogs")?;
        self.create_directory("Graphics/Icons")?;
        self.create_directory("Graphics/Panoramas")?;
        self.create_directory("Graphics/Pictures")?;
        self.create_directory("Graphics/Tilesets")?;
        self.create_directory("Graphics/Titles")?;
        self.create_directory("Graphics/Transitions")?;
        self.create_directory("Graphics/Windowskins")?;

        Ok(())
    }
}
