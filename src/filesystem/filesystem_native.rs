use std::cell::RefCell;
use std::fs;
use std::sync::Mutex;
use std::path::PathBuf;

use super::data_cache::DataCache;

/// Native filesystem implementation.
pub struct Filesystem {
    project_path: Mutex<RefCell<Option<PathBuf>>>,
}

impl Filesystem {
    pub fn new() -> Self {
        Self {
            project_path: Mutex::new(RefCell::new(None))
        }
    }

    pub fn unload_project(&self) {
        *self.project_path.lock().unwrap().borrow_mut() = None;
    }

    pub fn project_loaded(&self) -> bool {
        self.project_path.lock().unwrap().borrow().is_some()
    }

    pub fn project_path(&self) -> Option<PathBuf> {
        self.project_path.lock().unwrap().borrow().clone()
    }

    pub fn load_project(&self, path: PathBuf, cache: &DataCache) {
        *self.project_path.lock().unwrap().borrow_mut() = Some(path);
        cache.load(self);
    }

    pub fn read_data<T>(&self, path: &str) -> ron::error::SpannedResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .lock()
            .unwrap()
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join("Data_RON")
            .join(path);

        let data = fs::read_to_string(path)?;
        ron::from_str(&data)
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save();
    }

    pub async fn try_open_project(&self, cache: &DataCache) {
        if let Some(mut path) = rfd::FileDialog::default()
            .add_filter("project file", &["rxproj", "lum"])
            .pick_file()
        {
            path.pop(); // Pop off filename
            self.load_project(path, cache)
        }
    }
}
