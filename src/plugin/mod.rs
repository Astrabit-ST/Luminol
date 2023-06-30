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
use crate::{lua, state};
use dashmap::DashMap;
use loader::{LoadedPlugin, Manifest, LUA};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use result::{BasicResult, Error, ErrorKind, Result};
use std::{env, path::PathBuf};

pub mod api;
pub mod loader;
pub mod result;
pub mod ui;

#[macro_export]
macro_rules! global_data_path {
    () => {{
        let appdata = $crate::plugin::get_application_data_path();
        let mut buffer = PathBuf::from(appdata);
        buffer.push("Astrabit Studios");
        buffer.push("Luminol");
        buffer
    }};
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

static MANAGER: Lazy<Manager> = Lazy::new(|| {
    let manager = Manager::initialize();
    if let Err(why) = manager.load_all() {
        state!().toasts.error(why.to_string());
    }
    manager
});
#[derive(Debug)]
pub struct Manager {
    lookup_paths: RwLock<Vec<PathBuf>>,
    plugins: DashMap<String, LoadedPlugin>,
}
impl Manager {
    pub fn initialize() -> Self {
        api::bind().unwrap();
        Self {
            lookup_paths: RwLock::new(vec![{
                let mut path = global_data_path!();
                path.push("plugins");
                path
            }]),
            plugins: DashMap::new(),
        }
    }
    pub fn get() -> &'static Manager {
        &MANAGER
    }

    pub fn load<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        Err(if self.plugins.contains_key(&id) {
            Error::new(ErrorKind::AlreadyLoaded).set_plugin_id(id)
        } else {
            for path in self.lookup_paths.read().iter() {
                match loader::load(path, id.clone()) {
                    Ok(loaded_plugin) => {
                        self.plugins
                            .insert(loaded_plugin.manifest.id.clone(), loaded_plugin);
                        return Ok(());
                    }
                    Err(why) => match why.kind() {
                        ErrorKind::NotFound => continue,
                        _ => return Err(why),
                    },
                }
            }

            Error::new(ErrorKind::NotFound).set_plugin_id(id)
        })
    }
    pub fn unload<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        if self.plugins.remove(&id).is_none() {
            Err(Error::new(ErrorKind::NotFound).set_plugin_id(id))
        } else {
            Ok(())
        }
    }
    pub fn reload<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        self.unload(&id)?;
        self.load(id)
    }

    pub fn load_all(&self) -> Result<()> {
        for path in self.lookup_paths.read().iter() {
            for manifest in
                Manifest::from_directory(path).map_err(|why| Error::new(why).set_path(path))?
            {
                self.load(manifest.id)?;
            }
        }

        Ok(())
    }
    pub fn unload_all(&self) -> Result<()> {
        for id in self.plugins.iter().map(|x| x.key().clone()) {
            self.unload(id)?;
        }

        Ok(())
    }
    pub fn reload_all(&self) -> Result<()> {
        self.unload_all()?;
        self.load_all()
    }

    pub fn get_manifests(&self) -> impl Iterator<Item = Manifest> {
        self.plugins
            .iter()
            .map(|x| x.manifest.clone())
            .collect::<Vec<Manifest>>()
            .into_iter()
    }

    pub fn is_plugin_loaded<Id: ToString>(&self, id: Id) -> bool {
        self.plugins.contains_key(&id.to_string())
    }
    pub fn is_plugin_active<Id: ToString>(&self, id: Id) -> bool {
        self.plugins.get(&id.to_string()).is_some_and(|entry| {
            let thread: mlua::Thread<'_> = lua!().registry_value(&entry.thread).unwrap();
            thread.status() == mlua::ThreadStatus::Resumable
        })
    }

    pub fn activate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        info!("Checking if the plugin with an ID of `{id}` exists...");
        let result = || -> core::result::Result<(), ErrorKind> {
            if let Some(mut entry) = self.plugins.get_mut(&id) {
                info!(target: "luminol::plugin::manager", "Plugin found, activating...");
                let lua = LUA.lock().unwrap();
                let function: mlua::Function<'_> = lua.registry_value(&entry.entry_fn)?;
                debug!(target: "luminol::plugin::manager", "entry_fn registry value: {function:?}");
                let thread: mlua::Thread<'_> = lua.registry_value(&entry.thread)?;
                thread.resume::<_, ()>(())?;
                info!(target: "luminol::plugin::manager", "All done, plugin active.");
            }

            Ok(())
        }();

        result.map_err(|why| Error::new(why).set_plugin_id(id))
    }
    pub fn deactivate_plugin<Id: ToString>(&self, id: Id) -> Result<()> {
        let id = id.to_string();
        info!(target: "luminol::plugin::loader", "Checking if the plugin with an ID of `{id}` exists...");
        fn internal(this: &Manager, id: &str) -> BasicResult<()> {
            let lua = lua!();
            if let Some(entry) = this.plugins.get_mut(id) {
                info!(target: "luminol::plugin::loader", "Plugin found, checking it's activation state...");
                let thread: mlua::Thread<'_> = lua.registry_value(&entry.thread)?;
                if thread.status() == mlua::ThreadStatus::Unresumable {
                    info!(target: "luminol::plugin::loader", "Active. Deactivating...");
                    let function: mlua::Function<'_> = lua.registry_value(&entry.entry_fn)?;
                    thread.reset(function)?;
                    info!(target: "luminol::plugin::loader", "Done. `{id}` is loaded, but inactive.");
                } else {
                    warn!(target: "luminol::plugin::loader", "Plugin is already inactive, nothing left to do.");
                }
            }

            Ok(())
        }

        internal(self, id.as_str()).map_err(|e| Error::new(e).set_plugin_id(id))
    }
}
