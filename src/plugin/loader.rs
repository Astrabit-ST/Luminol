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
use super::result::{BasicResult, Error, ErrorKind, Result};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};
pub static LUA: Lazy<Mutex<mlua::Lua>> = Lazy::new(|| {
    /* Create a sandboxed Lua Environment */
    let interp = mlua::Lua::new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()).unwrap();
    interp.sandbox(true).unwrap();
    interp.into()
});

#[macro_export]
macro_rules! lua {
    () => {
        $crate::plugin::loader::LUA.lock().unwrap()
    };
}

#[derive(Debug)]
pub struct LoadedPlugin {
    pub manifest: Manifest,
    pub entry_fn: mlua::RegistryKey,
    pub thread: mlua::RegistryKey,
}

#[derive(Deserialize, Serialize)]
pub struct RawManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub main_file: Option<String>,
}
impl From<RawManifest> for Manifest {
    fn from(value: RawManifest) -> Self {
        Self {
            id: value.id,
            name: value.name,
            version: value.version,
            authors: value.authors,
            main_file: PathBuf::from(value.main_file.unwrap_or({
                warn!(target: "luminol::plugin::raw_manifest_loader", "The `main_file` key is missing. Assuming that the path is `src/main.lua`");
                String::from(if cfg!(unix) {
                    "src/main.lua"
                } else {
                    "src\\main.lua"
                })
            })),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub main_file: PathBuf,
}

impl Manifest {
    pub fn from_directory<P: AsRef<Path>>(path: P) -> BasicResult<impl Iterator<Item = Self>> {
        let path: &Path = path.as_ref();

        if !path.exists() {
            return Err(ErrorKind::NotFound);
        }

        let mut manifests = Vec::new();
        for dir_entry in fs::read_dir(path)?.flatten() {
            let dir_entry_path = dir_entry.path();

            if dir_entry_path.is_dir() {
                let manifest_path = {
                    let mut path = dir_entry_path.clone();
                    path.push("plugin.toml");
                    path
                };

                if manifest_path.exists() {
                    manifests.push(Self::from_file(manifest_path)?);
                } else {
                    let mut manifests_in_dir = Self::from_directory(dir_entry_path)?.collect();
                    manifests.append(&mut manifests_in_dir);
                };
            } else {
                continue;
            }
        }

        Ok(manifests.into_iter())
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> BasicResult<Self> {
        let path: &Path = path.as_ref();
        info!(target: "luminol::plugin::manifest_loader", "Trying to load a `{path:?}` file as a manifest...");
        Self::from_string(fs::read_to_string(path)?, path)
    }

    pub fn from_string<S: ToString, P: AsRef<Path>>(string: S, path: P) -> BasicResult<Self> {
        let path = path.as_ref();

        let mut manifest: Self = toml::from_str::<RawManifest>(string.to_string().as_str())?.into();
        if manifest.main_file.is_absolute() {
            Err(ErrorKind::ExpectedRelativePath)
        } else {
            manifest.main_file = {
                let mut path = path.to_path_buf();
                path.pop();
                path.push(manifest.main_file);
                path
            };
            info!(target: "luminol::plugin::manifest_loader", "Manifest has been successfully loaded!");
            debug!(
                target: "luminol::plugin::manifest_loader",
                "Manifest = {{\n\tName = {}\n\tVersion = {}\n\tAuthors = {:?}\n\tMain script file location = {:?}\n}}",
                manifest.name,
                manifest.version,
                manifest.authors,
                manifest.main_file
            );
            Ok(manifest)
        }
    }
}

impl ToString for Manifest {
    fn to_string(&self) -> String {
        toml::to_string_pretty(&self).unwrap()
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for Manifest {
    fn into(self) -> String {
        self.to_string()
    }
}

pub fn load<P: AsRef<Path>, Id: ToString>(path: P, id: Id) -> Result<LoadedPlugin> {
    let id = id.to_string();
    let path = path.as_ref();
    info!(target: "luminol::plugin::loader", "Attempting to load a plugin with ID of {id}");
    fn internal(path: &Path, id: &str) -> std::result::Result<LoadedPlugin, ErrorKind> {
        info!(target: "luminol::plugin::loader", "Trying to find the plugin in the `{path:?}` directory...");
        fs::DirBuilder::new().recursive(true).create(path)?;
        for manifest in Manifest::from_directory(path)? {
            if manifest.id == id {
                info!(target: "luminol::plugin::loader", "Plugin found! Loading it's main script into the Lua Interpreter...");
                let code = fs::read_to_string(manifest.main_file.clone())?;
                let lua = lua!();
                let function = lua.load(&code).into_function()?;
                let entry_fn = lua.create_registry_value(function.clone())?;
                let thread = lua.create_thread(function)?;
                let thread = lua.create_registry_value(thread)?;

                info!(
                    target: "luminol::plugin::loader",
                    "Done. Plugin \"{}@{}\" by {} has been successfully loaded.",
                    manifest.name,
                    manifest.version,
                    manifest.authors.join(", ")
                );
                return Ok(LoadedPlugin {
                    manifest,
                    entry_fn,
                    thread,
                });
            }
        }
        Err(ErrorKind::NotFound)
    }
    internal(path, id.as_str()).map_err(|e| Error::new(e).set_plugin_id(id))
}
