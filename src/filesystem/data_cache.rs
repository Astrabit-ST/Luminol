use crate::data::rmxp_structs::rpg;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

use super::Filesystem;

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
#[derive(Default)]
pub struct DataCache {
    inner: RefCell<Inner>,
}

#[derive(Default)]
pub struct Inner {
    pub tilesets: Option<Vec<rpg::Tileset>>,
    pub mapinfos: Option<HashMap<i32, rpg::MapInfo>>,
    pub maps: HashMap<i32, rpg::Map>,
}

impl DataCache {
    pub fn load(&self, filesystem: &Filesystem) {
        let mut inner = self.inner.borrow_mut();
        inner.mapinfos = Some(
            filesystem
                .read_data("MapInfos.ron")
                .expect("Failed to load Map Infos"),
        );

        inner.tilesets = Some(
            filesystem
                .read_data("Tilesets.ron")
                .expect("Failed to load Tilesets"),
        );
    }

    pub fn load_map(&self, filesystem: &Filesystem, id: i32) -> RefMut<'_, rpg::Map> {
        RefMut::map(self.inner.borrow_mut(), |inner| {
            inner.maps.entry(id).or_insert_with(|| {
                filesystem
                    .read_data(&format!("Map{:0>3}.ron", id))
                    .expect("Failed to load map")
            })
        })
    }

    pub fn map_infos(&self) -> RefMut<'_, Option<HashMap<i32, rpg::MapInfo>>> {
        RefMut::map(self.inner.borrow_mut(), |i| {
            &mut i.mapinfos
        })
    }

    pub fn tilesets(&self) -> RefMut<'_, Option<Vec<rpg::Tileset>>> {
        RefMut::map(self.inner.borrow_mut(), |i| {
            &mut i.tilesets
        })
    }

    pub fn save(&self, filesystem: &Filesystem) {
        // Write map data and clear map cache.
        let mut inner = self.inner.borrow_mut();
        for (id, map) in inner.maps.drain() {
            filesystem
                .save_data(&format!("Map{:0>3}.ron", id), &map)
                .expect("Failed to write Map data");
        }
        if let Some(tilesets) = inner.tilesets.as_ref() {
            filesystem
                .save_data("Tilesets.ron", tilesets)
                .expect("Failed to write Tileset data");
        }
        if let Some(mapinfos) = inner.mapinfos.as_ref() {
            filesystem
                .save_data("MapInfos.ron", mapinfos)
                .expect("Failed to write MapInfos data");
        }
    }
}
