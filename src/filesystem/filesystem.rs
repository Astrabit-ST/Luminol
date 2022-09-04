use std::fs;
use std::path::PathBuf;

pub struct Filesystem {
    pub project_path: Option<PathBuf>,
}

impl Filesystem {
    pub fn new() -> Self {
        Self { project_path: None }
    }

    pub fn read_data<T>(&self, path: String) -> ron::error::SpannedResult<T>
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
}
