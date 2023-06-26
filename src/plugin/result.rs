// Copyright (C) 2023 Lily Lyons
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
use std::{
    io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    plugin_id: Option<String>,
    path: Option<PathBuf>,
}
impl Error {
    pub fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            plugin_id: None,
            path: None,
        }
    }

    pub fn set_plugin_id<Id: ToString>(mut self, new_id: Id) -> Self {
        self.plugin_id = Some(new_id.to_string());
        self
    }

    pub fn set_path<P: AsRef<Path>>(mut self, new_path: P) -> Self {
        self.path = Some(new_path.as_ref().to_path_buf());
        self
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn plugin_id(&self) -> Option<&String> {
        self.plugin_id.as_ref()
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
}
impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            if let Some(id) = &self.plugin_id {
                format!("plugin with id of `{id}` ")
            } else if let Some(path) = &self.path {
                format!("manifest located in `{}` ", path.to_string_lossy())
            } else {
                String::from("unknown plugin/manifest")
            },
            match &self.kind {
                ErrorKind::AlreadyLoaded | ErrorKind::NotFound => {
                    format!("is {}", self.kind.to_string())
                }
                ErrorKind::ExpectedRelativePath | ErrorKind::ExpectedFileName => {
                    format!(
                        "has expected a {} in the `main_file` key",
                        /* TODO: Cannot derive PartialEq | Eq, provide a manual implementation later. */
                        if self.kind.to_string().ends_with("path") {
                            "relative path"
                        } else {
                            "path to a file, not a directory"
                        }
                    )
                }
                ErrorKind::Io(why) => format!("has experienced an I/O error: {why}"),
                _ => todo!(),
            }
        )
    }
}

impl std::error::Error for Error {}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("already loaded")]
    AlreadyLoaded,
    #[error("not found")]
    NotFound,
    #[error("expected relative path")]
    ExpectedRelativePath,
    #[error("expected file name")]
    ExpectedFileName,

    #[error("lua: {0}")]
    Lua(#[from] mlua::Error),
    #[error("i/o: {0}")]
    Io(#[from] io::Error),
    #[error("failure while parsing a toml file: {0}")]
    TomlParser(#[from] toml::de::Error),
}

pub type Result<T> = core::result::Result<T, Error>;
pub type BasicResult<T> = core::result::Result<T, ErrorKind>;
