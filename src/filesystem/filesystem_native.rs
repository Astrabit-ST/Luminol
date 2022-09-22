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

use super::data_cache::DataCache;

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

    pub fn load_project(&self, path: PathBuf, cache: &DataCache) {
        *self.project_path.borrow_mut() = Some(path);
        cache.load(self);
    }

    pub fn dir_children(&self, path: &str) -> fs::ReadDir {
        fs::read_dir(
            self.project_path
                .borrow()
                .as_ref()
                .expect("Project path not specified")
                .join(path),
        )
        .expect("Directory missing")
    }

    pub fn bufreader(&self, path: &str) -> BufReader<File> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join(path);
        BufReader::new(File::open(path).expect("Failed to open file"))
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

    pub fn read_bytes(&self, path: &str) -> Result<Vec<u8>, std::io::Error> {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join(path);
        fs::read(path)
    }

    pub fn save_data<T>(&self, path: &str, data: &T) -> Result<(), std::io::Error>
    where
        T: serde::ser::Serialize,
    {
        let path = self
            .project_path
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join("Data_RON")
            .join(path);
        println!("saving {}", path.display());

        let contents = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
            .expect("Failed to serialize data");
        fs::write(path, contents)
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save(self);
    }

    pub fn try_open_project(&self, cache: &DataCache) {
        if let Some(mut path) = rfd::FileDialog::default()
            .add_filter("project file", &["rxproj", "lum"])
            .pick_file()
        {
            path.pop(); // Pop off filename
            self.load_project(path, cache)
        }
    }
}
