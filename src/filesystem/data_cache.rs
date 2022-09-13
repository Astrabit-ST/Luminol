use crate::data::rmxp_structs::rpg;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    inner: RefCell<Inner>,
}

#[derive(Default)]
pub struct Inner {
    pub mapinfos: Option<HashMap<i32, rpg::MapInfo>>,
}

impl DataCache {
    pub fn load(&self, filesystem: &crate::filesystem::Filesystem) {
        let mut inner = self.inner.borrow_mut();
        inner.mapinfos = Some(
            filesystem
                .read_data("MapInfos.ron")
                .expect("Failed to load Map Infos"),
        );
    }

    // TODO: Find a better way.
    pub fn borrow_mut(&self) -> RefMut<'_, Inner> {
        self.inner.borrow_mut()
    }

    #[allow(dead_code)]
    pub fn borrow(&self) -> Ref<'_, Inner> {
        self.inner.borrow()
    }

    pub fn save(&self) {}
}
