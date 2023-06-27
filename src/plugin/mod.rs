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
use self::result::ErrorKind;
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use result::{BasicResult, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

pub mod result;
pub mod ui;

pub static LOADER: Lazy<Loader> = Lazy::new(Loader::init);

pub static LUA: Lazy<Mutex<mlua::Lua>> = Lazy::new(|| {
    /* Create a sandboxed Lua Environment */
    let interp = mlua::Lua::new_with(mlua::StdLib::ALL_SAFE, mlua::LuaOptions::default()).unwrap();
    interp.sandbox(true).unwrap();
    interp.into()
});

#[macro_export]
macro_rules! lua {
    () => {
        $crate::plugin::LUA.lock()
    };
}

#[macro_export]
macro_rules! plugin_loader {
    () => {
        &*$crate::plugin::LOADER
    };
}

macro_rules! global_data_path {
    () => {{
        let appdata = get_application_data_path();
        let mut buffer = PathBuf::from(appdata);
        buffer.push("Astrabit Studios");
        buffer.push("Luminol");
        buffer
    }};
}

#[derive(Debug)]
pub struct Loader {
    pub lookup_paths: Vec<PathBuf>,
    pub plugins: dashmap::DashMap<String, LoadedPlugin>,
}

#[derive(Debug)]
pub struct LoadedPlugin {
    manifest: Manifest,
    entry_fn: mlua::RegistryKey,
    thread: Option<mlua::RegistryKey>,
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
                warn!("The `main_file` key is missing. Assuming that the path is `src/main.lua`");
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
    fn from_directory<P: AsRef<Path>>(path: P) -> BasicResult<impl Iterator<Item = Self>> {
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

    fn from_file<P: AsRef<Path>>(path: P) -> BasicResult<Self> {
        let path: &Path = path.as_ref();
        info!("Trying to load a `{path:?}` file as a manifest...");
        Self::from_string(fs::read_to_string(path)?, path)
    }

    fn from_string<S: ToString, P: AsRef<Path>>(string: S, path: P) -> BasicResult<Self> {
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
            info!("Manifest has been successfully loaded!");
            debug!(
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

impl Loader {
    pub fn init() -> Self {
        Self::try_init().unwrap()
    }

    pub fn try_init() -> Result<Self> {
        info!("Initializing the plugin loader... Done");
        Ok(Loader {
            lookup_paths: vec![{
                let mut buf = global_data_path!();
                buf.push("plugins");
                buf
            }],
            plugins: dashmap::DashMap::default(),
        })
    }

    pub fn load<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        info!("Attempting to load a plugin with ID of {id}");
        fn internal(this: &Loader, id: &str) -> std::result::Result<(), ErrorKind> {
            if this.plugins.contains_key(id) {
                return Err(ErrorKind::AlreadyLoaded);
            }

            info!("Requested plugin is not loaded, continuing the loading process.");

            for path in &this.lookup_paths {
                info!("Trying to find the plugin in the `{path:?}` directory...");
                fs::DirBuilder::new().recursive(true).create(path)?;
                for manifest in Manifest::from_directory(path)? {
                    if manifest.id == id {
                        info!("Plugin found! Loading it's main script into the Lua Interpreter...");
                        let code = fs::read_to_string(manifest.main_file.clone())?;
                        let lua = lua!();
                        let function = lua.load(&code).into_function()?;
                        let entry_fn = lua.create_registry_value(function)?;

                        this.plugins.insert(
                            manifest.id.clone(),
                            LoadedPlugin {
                                manifest: manifest.clone(),
                                entry_fn,
                                thread: None,
                            },
                        );

                        info!(
                            "Done. Plugin \"{}@{}\" by {} has been successfully loaded.",
                            manifest.name, manifest.version, manifest.authors.join(", ")
                        );

                        return Ok(());
                    }
                }
            }
            Ok(())
        }
        internal(self, id.as_str()).map_err(|e| Error::new(e).set_plugin_id(id))
    }
    pub fn reload<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();

        self.unload(id.clone());
        self.load(id)
    }
    pub fn unload<Id: ToString>(&self, id: Id) {
        self.plugins.remove(&id.to_string());
    }

    pub fn activate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        info!("Checking if the plugin with an ID of `{id}` exists...");
        let result = || -> core::result::Result<(), ErrorKind> {
            if let Some(mut entry) = self.plugins.get_mut(&id) {
                info!("Plugin found, activating...");
                let lua = LUA.lock();
                let function = lua.registry_value(&entry.entry_fn)?;
                debug!("entry_fn registry value: {function:?}");
                let thread = lua.create_thread(function)?;
                thread.resume::<_, ()>(())?;
                debug!("created a lua thread: {thread:?}");
                let thread = lua.create_registry_value(thread)?;
                debug!("successfully registered the newly created thread as a registry value");
                entry.thread = Some(thread);
                info!("All done, plugin active.");
            }

            Ok(())
        }();

        result.map_err(|why| Error::new(why).set_plugin_id(id))
    }

    pub fn is_plugin_active<Id: ToString>(&self, id: Id) -> bool {
        self.plugins
            .get(&id.to_string())
            .is_some_and(|entry| entry.thread.is_some())
    }

    pub fn deactivate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        info!("Checking if the plugin with an ID of `{id}` exists...");
        fn internal(this: &Loader, id: &str) -> BasicResult<()> {
            if let Some(mut entry) = this.plugins.get_mut(id) {
                info!("Plugin found, checking it's activation state...");
                if let Some(thread) = entry.thread.take() {
                    info!("Active. Deactivating...");
                    let lua = lua!();
                    lua.remove_registry_value(thread)?;
                    info!("Done. `{id}` is loaded, but inactive.");
                } else {
                    info!("Plugin is already inactive, nothing left to do.");
                }
            }

            Ok(())
        }

        internal(self, id.as_str()).map_err(|e| Error::new(e).set_plugin_id(id))
    }

    pub fn get_manifests(&self) -> impl Iterator<Item = Manifest> {
        self.plugins
            .iter()
            .map(|x| x.manifest.clone()).collect::<Vec<Manifest>>().into_iter()
    }
}

fn get_application_data_path() -> String {
    let mut home_directory = env::var(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap();

    home_directory.push_str(if cfg!(windows) {
        "\\AppData\\LocalLow"
    } else {
        "/.local"
    });

    home_directory
}
