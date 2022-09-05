use crate::data::rmxp_structs::rpg;
use std::collections::HashMap;

/// A struct representing a cache of the current data.
/// This is done so data stored here can be written to the disk on demand.
pub struct DataCache {
    pub mapinfos: HashMap<i32, rpg::MapInfo>,
}

impl DataCache {
    pub fn load(filesystem: &crate::filesystem::Filesystem) -> Self {
        Self {
            mapinfos: filesystem
                .read_data("MapInfos.ron")
                .expect("Failed to load Map Infos"),
        }
    }

    pub fn save(&self) {}
}
