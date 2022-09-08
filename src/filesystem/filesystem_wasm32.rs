use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::Mutex;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

use super::data_cache::DataCache;

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    async fn js_open_project() -> JsValue;
    fn js_filesystem_supported() -> bool;
}

pub struct Filesystem {
    project_path: Mutex<RefCell<Option<PathBuf>>>,
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
            project_path: Mutex::new(RefCell::new(None)),
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

    pub fn read_data<T>(&self, path: &str) -> Result<T, &str>
    where
        T: serde::de::DeserializeOwned,
    {
        Err("NYI for wasm32")
    }

    pub fn save_cached(&self, data_cache: &super::data_cache::DataCache) {
        data_cache.save();
    }

    pub async fn try_open_project(&self, cache: &DataCache) {
        let handle = js_open_project().await;
        // Should have this field
        let path = PathBuf::from(
            js_sys::Reflect::get(&handle, &JsValue::from("name"))
                .unwrap()
                .as_string()
                .unwrap(),
        );
        *self.project_path.lock().unwrap().borrow_mut() = Some(path);

        //path.pop(); // Pop off filename
        //self.load_project(path)
    }
}
