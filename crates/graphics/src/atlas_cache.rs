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
use crate::tiles::Atlas;

#[derive(Default, Debug)]
pub struct Cache {
    atlases: dashmap::DashMap<usize, Atlas>,
}

impl Cache {
    pub fn load_atlas(
        &self,
        graphics_state: &crate::GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        tileset: &luminol_data::rpg::Tileset,
    ) -> anyhow::Result<Atlas> {
        Ok(self
            .atlases
            .entry(tileset.id)
            .or_try_insert_with(|| Atlas::new(graphics_state, filesystem, tileset))?
            .clone())
    }

    pub fn reload_atlas(
        &self,
        graphics_state: &crate::GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        tileset: &luminol_data::rpg::Tileset,
    ) -> anyhow::Result<Atlas> {
        Ok(self
            .atlases
            .entry(tileset.id)
            .insert(Atlas::new(graphics_state, filesystem, tileset)?)
            .clone())
    }

    pub fn clear(&self) {
        self.atlases.clear()
    }
}
