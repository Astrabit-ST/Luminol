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
#![allow(unsafe_code)]
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use result::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Debug,
    fs,
    io::Read,
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
            main_file: PathBuf::from(value.main_file.unwrap_or(String::from("src/main.lua"))),
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
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = fs::OpenOptions::new().read(true).open(path)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;

        Self::from_string(buffer)
    }

    pub fn from_string<S: ToString>(string: S) -> Result<Self> {
        Ok(toml::from_str::<RawManifest>(string.to_string().as_str())?.into())
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
        println!("{self:?}");
        let id = id.to_string();
        for path in &self.lookup_paths {
            fs::DirBuilder::new().recursive(true).create(path)?;
            for direntry in fs::read_dir(path)? {
                let mut path = direntry?.path();
                path.push("plugin.toml");

                if !path.exists() {
                    continue;
                }

                let manifest = Manifest::from_file(path)?;
                if manifest.id == id {
                    if manifest.main_file.is_absolute() {
                        return Err(Error::ExpectedRelativePath);
                    }
                    if manifest.main_file.is_dir() {
                        return Err(Error::ExpectedFileName);
                    }
                    let code = {
                        let mut file = fs::OpenOptions::new()
                            .read(true)
                            .open(manifest.main_file.clone())?;
                        let mut buffer = String::new();
                        file.read_to_string(&mut buffer)?;
                        buffer
                    };

                    let lua = lua!();
                    let function = lua.load(&code).into_function()?;
                    let entry_fn = lua.create_registry_value(function)?;

                    self.plugins.insert(
                        manifest.id.clone(),
                        LoadedPlugin {
                            manifest,
                            entry_fn,
                            thread: None,
                        },
                    );
                    return Ok(());
                }
            }
        }
        Err(Error::NotFound)
    }

    pub fn activate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        if let Some(mut entry) = self.plugins.get_mut(&id.to_string()) {
            let lua = LUA.lock();
            let function = lua.registry_value(&entry.entry_fn)?;
            let thread = lua.create_thread(function)?;
            let thread = lua.create_registry_value(thread)?;
            entry.thread = Some(thread);
        }

        Ok(())
    }

    pub fn is_plugin_active<Id: ToString>(&self, id: Id) -> bool {
        self.plugins
            .get(&id.to_string())
            .is_some_and(|entry| entry.thread.is_some())
    }

    pub fn deactivate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        if let Some(mut entry) = self.plugins.get_mut(&id.to_string()) {
            if let Some(thread) = entry.thread.take() {
                let lua = lua!();
                let thread: mlua::Thread<'_> = lua.registry_value(&thread)?;
            }
        }

        Ok(())
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
