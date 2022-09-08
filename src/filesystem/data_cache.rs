use crate::data::rmxp_structs::rpg;
use std::{cell::{RefCell,}, collections::HashMap, sync::{Mutex, MutexGuard}};

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
pub struct DataCache {
    inner: Mutex<RefCell<Inner>>,
}

#[derive(Default)]
pub struct Inner {
    pub mapinfos: Option<HashMap<i32, rpg::MapInfo>>,
}

impl DataCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(RefCell::new(Inner::default())),
        }
    }

    pub fn load(&self, filesystem: &crate::filesystem::Filesystem) {
        let inner = self.inner.lock().unwrap();
        let mut inner = inner.borrow_mut();
        inner.mapinfos = Some(filesystem
            .read_data("MapInfos.ron")
            .expect("Failed to load Map Infos"));
    }

    pub fn get(&self) -> MutexGuard<'_, RefCell<Inner>> {
        self.inner.lock().unwrap()
    }

    pub fn save(&self) {}
}
