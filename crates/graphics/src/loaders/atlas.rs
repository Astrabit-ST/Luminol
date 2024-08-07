// Copyright (C) 2024 Melody Madeline Lyons
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
use crate::primitives::cells::Atlas as AnimationAtlas;
use crate::{Atlas, GraphicsState};

#[derive(Default)]
pub struct Loader {
    atlases: dashmap::DashMap<usize, Atlas>,
    animation_atlases: dashmap::DashMap<usize, AnimationAtlas>,
}

impl Loader {
    pub fn load_atlas(
        &self,
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        tileset: &luminol_data::rpg::Tileset,
    ) -> Atlas {
        self.atlases
            .entry(tileset.id)
            .or_insert_with(|| Atlas::new(graphics_state, filesystem, tileset))
            .clone()
    }

    pub fn load_animation_atlas(
        &self,
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        animation: &luminol_data::rpg::Animation,
    ) -> AnimationAtlas {
        self.animation_atlases
            .entry(animation.id)
            .or_insert_with(|| AnimationAtlas::new(graphics_state, filesystem, animation))
            .clone()
    }

    pub fn reload_atlas(
        &self,
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        tileset: &luminol_data::rpg::Tileset,
    ) -> Atlas {
        self.atlases
            .entry(tileset.id)
            .insert(Atlas::new(graphics_state, filesystem, tileset))
            .clone()
    }

    pub fn reload_animation_atlas(
        &self,
        graphics_state: &GraphicsState,
        filesystem: &impl luminol_filesystem::FileSystem,
        animation: &luminol_data::rpg::Animation,
    ) -> AnimationAtlas {
        self.animation_atlases
            .entry(animation.id)
            .insert(AnimationAtlas::new(graphics_state, filesystem, animation))
            .clone()
    }

    pub fn get_atlas(&self, id: usize) -> Option<Atlas> {
        self.atlases.get(&id).map(|atlas| atlas.clone())
    }

    pub fn get_animation_atlas(&self, id: usize) -> Option<AnimationAtlas> {
        self.animation_atlases.get(&id).map(|atlas| atlas.clone())
    }

    pub fn get_expect(&self, id: usize) -> Atlas {
        self.atlases.get(&id).expect("Atlas not loaded!").clone()
    }

    pub fn get_animation_expect(&self, id: usize) -> AnimationAtlas {
        self.animation_atlases
            .get(&id)
            .expect("Atlas not loaded!")
            .clone()
    }

    pub fn clear(&self) {
        self.atlases.clear();
        self.animation_atlases.clear();
    }
}
