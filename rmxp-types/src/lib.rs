#![feature(min_specialization)]

// Editor specific types
pub mod rmxp;

// Shared structs with the same layout
mod shared;

mod rgss_structs;

mod helpers;

pub use helpers::*;
pub use rgss_structs::{Color, Table1, Table2, Table3, Tone};

pub mod rpg {
    pub use crate::rmxp::*;
    pub use crate::shared::*;

    pub type Actors = Vec<Actor>;
    pub type Animations = Vec<Animation>;
    pub type Armors = Vec<Armor>;
    pub type Classes = Vec<Class>;
    pub type CommonEvents = Vec<CommonEvent>;
    pub type Enemies = Vec<Enemy>;
    pub type Items = Vec<Item>;
    pub type MapInfos = std::collections::HashMap<usize, MapInfo>;
    pub type Skills = Vec<Skill>;
    pub type States = Vec<State>;
    pub type Tilesets = Vec<Tileset>;
    pub type Troops = Vec<Troop>;
    pub type Weapons = Vec<Weapon>;
}

pub type Path = Option<camino::Utf8PathBuf>;
