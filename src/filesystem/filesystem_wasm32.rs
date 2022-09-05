use std::path::PathBuf;

pub struct Filesystem {
    pub project_path: Option<PathBuf>,
    data_cache: Option<super::data_cache::DataCache>,
}

impl Filesystem {
    pub fn new() -> Self {
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
}
