use std::cell::RefCell;
use std::fs::{File, self};
use std::io::BufReader;
use std::path::PathBuf;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use super::data_cache::DataCache;

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    fn js_open_project() -> JsValue;
    fn js_filesystem_supported() -> bool;
}

#[derive(Default)]
pub struct Filesystem {
    project_path: RefCell<Option<PathBuf>>,
    handle: RefCell<Option<JsValue>>,
}

impl Filesystem {
    pub fn new() -> Self {
        if !js_filesystem_supported() {
            rfd::MessageDialog::new()
                .set_description("Filesystem not supported on this browser")
                .show();
            panic!("Filesystem not supported on this browser");
        }
        Self {
            project_path: RefCell::new(None),
            handle: RefCell::new(None),
        }
    }

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

    pub fn read_data<T>(&self, _path: &str) -> Result<T, &str>
    where
        T: serde::de::DeserializeOwned,
    {
        Err("NYI for wasm32")
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
        let path = self.project_path
            .borrow()
            .as_ref()
            .expect("Project path not specified")
            .join(path);
        BufReader::new(File::open(path).expect("Failed to open file"))
    }

    pub fn save_data<T>(&self, _path: &str, _data: &T) -> Result<(), ()>
    where
        T: serde::ser::Serialize,
    {
        Ok(())
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save(self);
    }

    pub fn try_open_project(&self, cache: &DataCache) {
        *self.handle.borrow_mut() = Some(js_open_project());
        self.load_project(PathBuf::from("Project"), cache);
    }
}
