use std::fs;
use std::path::PathBuf;

/// Native filesystem implementation.
#[derive(Default)]
pub struct Filesystem {
    project_path: Option<PathBuf>,
    data_cache: Option<super::data_cache::DataCache>,
}

impl Filesystem {
    pub fn new() -> Self {
        Self {
            ..Default::default()
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

    pub fn read_data<T>(&self, path: &str) -> ron::error::SpannedResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let path = self
            .project_path
            .as_ref()
            .expect("Project path not specified")
            .join("Data_RON")
            .join(path);

        let data = fs::read_to_string(path)?;
        ron::from_str(&data)
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

    pub fn try_open_project(&mut self) {
        if let Some(mut path) = rfd::FileDialog::default()
            .add_filter("project file", &["rxproj", "lum"])
            .pick_file()
        {
            path.pop(); // Pop off filename
            self.load_project(path)
        }
    }
}
