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
use crate::helpers::ParameterType;

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::MoveRoute")]
pub struct MoveRoute {
    pub repeat: bool,
    pub skippable: bool,
    pub list: Vec<MoveCommand>,
}

#[derive(Default, Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq)]
#[derive(alox_48::Deserialize, alox_48::Serialize)]
#[marshal(class = "RPG::MoveCommand")]
#[allow(missing_docs)]
pub struct MoveCommand {
    pub code: u16,
    pub parameters: Vec<ParameterType>,

    #[marshal(default = "rand::random")]
    #[marshal(skip)]
    #[serde(default = "rand::random")]
    #[serde(skip)]
    pub guid: u16,
}
