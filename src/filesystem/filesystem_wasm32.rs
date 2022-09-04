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
        Err(ron::error::SpannedError {
            code: ron::error::Error::Eof,
            position: ron::error::Position { line: 0, col: 0 },
        })
    }
}
