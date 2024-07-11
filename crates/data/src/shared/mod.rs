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

mod audio_file;
mod event;
mod mapinfo;
mod move_route;
mod script;

pub use audio_file::*;
pub use event::*;
pub use mapinfo::*;
pub use move_route::*;
pub use script::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Default, Hash)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum BlendMode {
    #[default]
    Normal = 0,
    Add = 1,
    Subtract = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Scope {
    #[default]
    None = 0,
    #[strum(to_string = "One Enemy")]
    OneEnemy = 1,
    #[strum(to_string = "All Enemies")]
    AllEnemies = 2,
    #[strum(to_string = "One Ally")]
    OneAlly = 3,
    #[strum(to_string = "All Allies")]
    AllAllies = 4,
    #[strum(to_string = "One Ally (HP 0)")]
    OneAllyHP0 = 5,
    #[strum(to_string = "All Allies (HP 0)")]
    AllAlliesHP0 = 6,
    #[strum(to_string = "The User")]
    User = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[derive(
    num_enum::TryFromPrimitive,
    num_enum::IntoPrimitive,
    strum::Display,
    strum::EnumIter
)]
#[derive(serde::Deserialize, serde::Serialize)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[repr(u8)]
#[serde(into = "u8")]
#[serde(try_from = "u8")]
#[marshal(into = "u8")]
#[marshal(try_from = "u8")]
pub enum Occasion {
    #[default]
    Always = 0,
    #[strum(to_string = "Only in battle")]
    OnlyBattle = 1,
    #[strum(to_string = "Only from the menu")]
    OnlyMenu = 2,
    Never = 3,
}
