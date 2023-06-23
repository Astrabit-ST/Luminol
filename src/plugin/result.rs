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
use std::io;

#[derive(Debug)]
pub enum Error {
    AlreadyLoaded,
    NotFound,
    ExpectedRelativePath,
    ExpectedFileName,

    Lua(mlua::Error),
    Io(io::Error),
    TomlParser(toml::de::Error),
}
impl From<mlua::Error> for Error {
    fn from(value: mlua::Error) -> Self {
        Self::Lua(value)
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlParser(value)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
