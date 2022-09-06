use std::path::PathBuf;
use wasm_bindgen::prelude::{wasm_bindgen};
use wasm_bindgen::{JsValue};

// Javascript interface for filesystem
#[wasm_bindgen(module = "/assets/filesystem.js")]
extern "C" {
    async fn js_open_project() -> JsValue;
    fn js_filesystem_supported() -> bool;
}

pub struct Filesystem {
    pub project_path: Option<PathBuf>,
    data_cache: Option<super::data_cache::DataCache>,
}

impl Filesystem {
    pub fn new() -> Self {
        if !js_filesystem_supported() {
            rfd::MessageDialog::new().set_description("Filesystem not supported on this browser").show();
            panic!("Filesystem not supported on this browser");
        }
        Self {
            project_path: None,
            data_cache: None,
        }
    }

    pub fn unload_project(&mut self) {
        self.project_path = None;
        self.data_cache = None;
    }

    pub fn project_loaded(&self) -> bool {
        self.project_path.is_some()
    }

    pub fn project_path(&self) -> &Option<PathBuf> {
        &self.project_path
    }

    pub fn load_project(&mut self, path: PathBuf) {
        self.project_path = Some(path);
        self.data_cache = Some(super::data_cache::DataCache::load(self));
    }

    pub fn read_data<T>(&self, path: &str) -> Result<T, &str>
    where
        T: serde::de::DeserializeOwned,
    {
        Err("NYI for wasm32")
    }

    pub fn data_cache(&mut self) -> Option<&mut super::data_cache::DataCache> {
        self.data_cache.as_mut()
    }

    pub fn save_cached(&self) {
        self.data_cache
            .as_ref()
            .expect("No Data Cache Loaded")
            .save();
    }

    pub async fn try_open_project(&mut self) {
        let path = js_open_project().await;
        
        //path.pop(); // Pop off filename
        //self.load_project(path)
    }
}
