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

use crate::data::data_cache::DataCache;

/// Native filesystem implementation.
#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
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
        path: PathBuf,
        cache: &'static DataCache,
    ) -> Result<(), String> {
        *self.project_path.borrow_mut() = Some(path);
        cache.load(self).await.map_err(|e| {
            *self.project_path.borrow_mut() = None;
            e
        })
    }

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

    pub async fn bufreader(&self, path: &str) -> Result<Cursor<Vec<u8>>, String> {
        Ok(Cursor::new(self.read_bytes(path).await?))
    }

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

    pub async fn read_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);
        async_fs::read(path).await.map_err(|e| e.to_string())
    }

    pub async fn save_data<T>(&self, path: &str, data: &T) -> Result<(), String>
    where
        T: serde::ser::Serialize,
    {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join("Data_RON")
            .join(path);

        let contents = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?;
        async_fs::write(path, contents)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn save_cached(&self, data_cache: &'static DataCache) -> Result<(), String> {
        data_cache.save(self).await
    }

    pub async fn try_open_project(&self, cache: &'static DataCache) -> Result<(), String> {
        if let Some(mut path) = rfd::FileDialog::default()
            .add_filter("project file", &["rxproj", "lum"])
            .pick_file()
        {
            path.pop(); // Pop off filename
            self.load_project(path, cache).await
        } else {
            Err("No project loaded".to_string())
        }
    }
}
