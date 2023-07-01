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
use crate::prelude::*;

#[derive(Default, Debug)]
pub struct Cache {
    atlases: dashmap::DashMap<i32, Atlas>,
}

impl Cache {
    pub fn load_atlas(&self, tileset_id: i32) -> Result<Atlas, String> {
        let entry = self.atlases.entry(tileset_id).or_try_insert_with(|| {
            let tilesets = state!().data_cache.tilesets();
            // We subtract 1 because RMXP is stupid and pads arrays with nil to start at 1.
            let tileset = &tilesets[tileset_id as usize - 1];

            Atlas::new(tileset)
        })?;
        Ok(entry.clone())
    }
}
