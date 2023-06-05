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
use crate::prelude::*;
use std::io::prelude::*;

mod rgss2a;
mod rgss3a;
mod rgssad;

mod host;

#[derive(Default, Debug)]
pub struct LuminolFS {
    state: AtomicRefCell<State>,
}

#[derive(Default, Debug)]
pub enum State {
    #[default]
    Unloaded,
    HostLoaded {
        host: host::HostFS,
        project_path: camino::Utf8PathBuf,
    },
    Loaded {
        host: host::HostFS,
        archiver: Archiver,
        project_path: camino::Utf8PathBuf,
    },
}

#[derive(Debug)]
enum Archiver {
    RGSSAD(rgssad::Archiver),
}

trait File: Read + Write + Seek {}

trait FileSystem {
    type File: File;
    type Error: std::error::Error;

    fn open_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<Self::File, Self::Error>;

    fn exists(&self, path: impl AsRef<camino::Utf8Path>) -> Result<bool, Self::Error>;

    fn create_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error>;

    fn remove_dir(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error>;

    fn remove_file(&self, path: impl AsRef<camino::Utf8Path>) -> Result<(), Self::Error>;

    fn read_dir(
        &self,
        path: impl AsRef<camino::Utf8Path>,
    ) -> Result<Vec<camino::Utf8PathBuf>, Self::Error>;
}
