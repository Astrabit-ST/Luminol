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
use mlua::{Lua, LuaOptions, StdLib};
use once_cell::sync::Lazy;
use parking_lot::{Mutex, RwLock};
use result::{Error, Result};
use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

pub mod result;
pub mod ui;

static LOADER: Lazy<Loader> = Lazy::new(|| Loader::init());

macro_rules! global_data_path {
    () => {{
        let appdata = get_application_data_path();
        let mut buffer = PathBuf::from(appdata);
        buffer.push("Astrabit Studios");
        buffer.push("Luminol");
        buffer
    }};
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

static LUA: Lazy<mlua::Lua> = Lazy::new(|| {
    /* Create a sandboxed Lua Environment */
    let interp = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default()).unwrap();
    interp.sandbox(true).unwrap();
});

#[derive(Debug)]
pub struct LoaderInner {
    pub lookup_paths: Vec<PathBuf>,
    pub plugins: dashmap::DashMap<String, LoadedPlugin>,
}

#[derive(Debug)]
struct LoadedPlugin {
    manifest: Manifest,
    entry_fn: mlua::Function<'static>,
    thread: Option<mlua::Thread<'static>>,
}

static_assertions::assert_impl_all!(mlua::Lua: Send);
static_assertions::assert_not_impl_all!(mlua::Lua: Sync);

static_assertions::assert_impl_all!(parking_lot::Mutex<mlua::Lua>: Send, Sync);

impl LoaderInner {
    pub fn load<Id: ToString>(&self, id: Id) -> Result<()> {
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
                    let function = self.lua.load(&code).into_function()?;
                    self.plugins
                        .insert(manifest.id.clone(), (manifest, function));
                    return Ok(());
                }
            }
        }
        Err(Error::NotFound)
    }

    pub fn activate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        if let Some(entry) = self.plugins.get_mut(&id.to_string()) {
            entry.thread = Some(LUA.create_thread(entry.entry_fn.clone()));
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
            entry.thread = None;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Loader {
    inner: LoaderInner,
}
impl Loader {
    pub fn init() -> Self {
        Self::try_init().unwrap()
    }
    pub fn try_init() -> Result<Self> {
        Ok(Self {
            inner: LoaderInner {
                lookup_paths: vec![{
                    let mut buf = global_data_path!();
                    buf.push("plugins");
                    buf
                }],
                plugins: dashmap::DashMap::deafault(),
            },
        })
    }

    pub fn load<Id: ToString>(&self, id: Id) -> Result<()> {
        inner.load(id)
    }

    pub fn activate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        inner.activate_plugin(id)
    }
    pub fn is_plugin_active<Id: ToString>(&self, id: Id) -> bool {
        inner.is_plugin_active(id)
    }
    pub fn deactivate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        inner.deactivate_plugin(id)
    }
}
impl Default for Loader {
    fn default() -> Self {
        Self::init()
    }
}

fn get_application_data_path() -> String {
    let mut home_directory =
        env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).unwrap();

    home_directory.push(if cfg!(windows) {
        "\\AppData\\LocalLow"
    } else {
        "/.local"
    });

    home_directory.to_string_lossy().to_string()
}
