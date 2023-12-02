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

    #[derive(Debug, Default)]
    pub struct Actors {
        pub data: Vec<Actor>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Animations {
        pub data: Vec<Animation>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Armors {
        pub data: Vec<Armor>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Classes {
        pub data: Vec<Class>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct CommonEvents {
        pub data: Vec<CommonEvent>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Enemies {
        pub data: Vec<Enemy>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Items {
        pub data: Vec<Item>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct MapInfos {
        pub data: std::collections::HashMap<usize, MapInfo>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Scripts {
        pub data: Vec<Script>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Skills {
        pub data: Vec<Skill>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct States {
        pub data: Vec<State>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Tilesets {
        pub data: Vec<Tileset>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Troops {
        pub data: Vec<Troop>,
        pub modified: bool,
    }

    #[derive(Debug, Default)]
    pub struct Weapons {
        pub data: Vec<Weapon>,
        pub modified: bool,
    }
}

pub use shared::BlendMode;

pub type Path = Option<camino::Utf8PathBuf>;
