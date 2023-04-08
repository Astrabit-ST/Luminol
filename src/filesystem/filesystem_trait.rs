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

use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::{data::config::RGSSVer, UpdateInfo};

#[async_trait(?Send)]
/// Filesystem abstraction.
pub trait Filesystem {
    /// Unload the currently loaded project.
    /// Does nothing if none is open.
    fn unload_project(&self);

    /// Is there a project loaded?
    fn project_loaded(&self) -> bool;

    /// Get the project path.
    fn project_path(&self) -> Option<PathBuf>;

    /// Get the directory children of a path.
    async fn dir_children(&self, path: impl AsRef<Path>) -> Result<Vec<String>, String>;

    /// Read a data file and deserialize it with RON (rusty object notation)
    /// In the future this will take an optional parameter (type) to set the loading method.
    /// (Options would be Marshal, RON, Lumina)
    async fn read_data<T: serde::de::DeserializeOwned>(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<T, String>;

    /// Read bytes from a file.
    async fn read_bytes(&self, provided_path: impl AsRef<Path>) -> Result<Vec<u8>, String>;

    /// Save some file's data by serializing it with RON.
    async fn save_data(&self, path: impl AsRef<Path>, data: impl AsRef<[u8]>)
        -> Result<(), String>;

    /// Check if file path exists
    async fn file_exists(&self, path: impl AsRef<Path>) -> bool;

    /// Save all cached files. An alias for [`DataCache::save`];
    async fn save_cached(&self, info: &'static UpdateInfo) -> Result<(), String>;
    /// Try to open a project.
    async fn try_open_project(
        &self,
        info: &'static UpdateInfo,
        #[cfg(not(target_arch = "wasm32"))] path: impl AsRef<Path>,
    ) -> Result<(), String>;

    /// Create a directory at the specified path.
    async fn create_directory(&self, path: impl AsRef<Path>) -> Result<(), String>;

    /// Spawn a picker window and retriev
    async fn spawn_project_file_picker(&self, info: &'static UpdateInfo) -> Result<(), String>;

    /// Try to create a project.
    async fn try_create_project(
        &self,
        name: String,
        info: &'static UpdateInfo,
        rgss_ver: RGSSVer,
    ) -> Result<(), String>;
}
