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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

use luminol_data::rpg;

#[derive(Default, Debug)]
pub struct Data {}

impl Data {
    /// Load all data required when opening a project.
    /// Does not load config. That is expected to have been loaded beforehand.
    pub fn load(&mut self) -> Result<(), String> {
        todo!()
    }

    // TODO dependcy cycle
    pub fn defaults_from_config(config: &luminol_config::project::Config) -> Self {
        todo!()
    }

    pub fn rxdata_ext(&self) -> &'static str {
        todo!()
    }

    /// Save all cached data to disk.
    /// Will flush the cache too.
    pub fn save(&self) -> Result<(), String> {
        todo!()
    }

    /// Setup default values
    // FIXME: Code jank
    pub fn setup_defaults(&mut self) {
        todo!()
    }
}

macro_rules! nested_ref_getter {
    ($(
        $typ:ty, $name:ident, $($enum_type:ident :: $variant:ident),+
    );*) => {
        $(
            #[allow(unsafe_code, dead_code)]
            pub fn $name(&self) -> &mut $typ {
                todo!()
            }
        )+
    };

}

impl Data {
    nested_ref_getter! {
        rpg::Actors, actors, State::Loaded;
        rpg::Animations, animations, State::Loaded;
        rpg::Armors, armors, State::Loaded;
        rpg::Classes, classes, State::Loaded;
        rpg::CommonEvents, common_events, State::Loaded;
        rpg::Enemies, enemies, State::Loaded;
        rpg::Items, items, State::Loaded;
        rpg::MapInfos, mapinfos, State::Loaded;
        Vec<rpg::Script>, scripts, State::Loaded;
        rpg::Skills, skills, State::Loaded;
        rpg::States, states, State::Loaded;
        rpg::System, system, State::Loaded;
        rpg::Tilesets, tilesets, State::Loaded;
        rpg::Troops, troops, State::Loaded;
        rpg::Weapons, weapons, State::Loaded
    }

    /// Load a map.
    #[allow(clippy::panic)]
    pub fn map(&self, _id: usize) -> &mut rpg::Map {
        todo!()
    }
}
