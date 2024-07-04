#![allow(non_upper_case_globals)]

// Editor specific types
pub mod rmxp;

// Shared structs with the same layout
mod shared;

mod option_vec;

mod rgss_structs;

pub mod helpers;

pub mod commands;

pub use helpers::*;
pub use option_vec::OptionVec;
pub use rgss_structs::{Color, Table1, Table2, Table3, Tone};

pub mod rpg {
    pub use crate::rmxp::*;
    pub use crate::shared::*;

    pub trait DatabaseEntry
    where
        Self: Default,
    {
        fn default_with_id(id: usize) -> Self;
    }

    macro_rules! basic_container {
        ($($parent:ident, $child:ident),* $(,)?) => {
            $(
                #[derive(Debug, Default)]
                pub struct $parent {
                    pub data: Vec<$child>,
                    pub modified: bool,
                }
            )*
        };
    }

    macro_rules! database_entry {
        ($($type:ident),* $(,)?) => {
            $(
                impl DatabaseEntry for $type {
                    fn default_with_id(id: usize) -> Self {
                        Self { id, ..Default::default() }
                    }
                }
            )*
        };
    }

    basic_container! {
        Actors, Actor,
        Animations, Animation,
        Armors, Armor,
        Classes, Class,
        CommonEvents, CommonEvent,
        Enemies, Enemy,
        Items, Item,
        Scripts, Script,
        Skills, Skill,
        States, State,
        Tilesets, Tileset,
        Troops, Troop,
        Weapons, Weapon,
    }

    database_entry! {
        Actor,
        Animation,
        Armor,
        Class,
        CommonEvent,
        Enemy,
        Item,
        Skill,
        State,
        Tileset,
        Troop,
        Weapon,
    }

    #[derive(Debug, Default)]
    pub struct MapInfos {
        pub data: std::collections::HashMap<usize, MapInfo>,
        pub modified: bool,
    }
}

pub use shared::BlendMode;

pub type Path = Option<camino::Utf8PathBuf>;
