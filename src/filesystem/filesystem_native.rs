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

    pub fn load_project(&self, path: PathBuf, cache: &DataCache) -> Result<(), String> {
        *self.project_path.borrow_mut() = Some(path);
        cache.load(self).map_err(|e| {
            *self.project_path.borrow_mut() = None;
            e
        })
    }

    pub fn dir_children(&self, path: &str) -> Result<fs::ReadDir, String> {
        fs::read_dir(
            self.project_path
                .borrow()
                .as_ref()
                .ok_or_else(|| "Project not open".to_string())?
                .join(path),
        )
        .map_err(|e| e.to_string())
    }

    pub fn bufreader(&self, path: &str) -> Result<BufReader<File>, String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);
        Ok(BufReader::new(File::open(path).map_err(|e| e.to_string())?))
    }

    pub fn read_data<T>(&self, path: &str) -> ron::error::SpannedResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join("Data_RON")
            .join(path);
        println!("Loading {}", path.display());

        let data = fs::read_to_string(path)?;
        ron::from_str(&data)
    }

    pub fn read_bytes(&self, path: &str) -> Result<Vec<u8>, String> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .ok_or_else(|| "Project not open".to_string())?
            .join(path);
        fs::read(path).map_err(|e| e.to_string())
    }

    pub fn save_data<T>(&self, path: &str, data: &T) -> Result<(), String>
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
        println!("saving {}", path.display());

        let contents = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
            .map_err(|e| e.to_string())?;
        fs::write(path, contents).map_err(|e| e.to_string())
    }

    pub fn save_cached(&self, data_cache: &DataCache) -> Result<(), String> {
        data_cache.save(self)
    }

    pub fn try_open_project(&self, cache: &DataCache) -> Result<(), String> {
        if let Some(mut path) = rfd::FileDialog::default()
            .add_filter("project file", &["rxproj", "lum"])
            .pick_file()
        {
            path.pop(); // Pop off filename
            self.load_project(path, cache)
        } else {
            Err("No project loaded".to_string())
        }
    }
}
