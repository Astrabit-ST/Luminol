#![feature(min_specialization)]
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

    #[derive(Debug, Default)]
    pub struct MapInfos {
        pub data: std::collections::HashMap<usize, MapInfo>,
        pub modified: bool,
    }
}

pub use shared::BlendMode;

pub type Path = Option<camino::Utf8PathBuf>;
