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
//
//     Additional permission under GNU GPL version 3 section 7
//
// If you modify this Program, or any covered work, by linking or combining
// it with Steamworks API by Valve Corporation, containing parts covered by
// terms of the Steamworks API by Valve Corporation, the licensors of this
// Program grant you additional permission to convey the resulting work.

mod error;
mod midi;
pub use error::{Error, Result};

pub use luminol_config::VolumeScale;

mod native;
#[cfg(target_arch = "wasm32")]
mod wrapper;

#[cfg(not(target_arch = "wasm32"))]
pub use native::Audio;
#[cfg(target_arch = "wasm32")]
pub use wrapper::Audio;

/// Different sound sources.
#[derive(strum::EnumIter, strum::Display, PartialEq, Eq, Clone, Copy, Hash)]
#[allow(clippy::upper_case_acronyms)]
#[allow(missing_docs)]
pub enum Source {
    BGM,
    BGS,
    ME,
    SE,
}

impl Source {
    pub fn as_path(&self) -> &camino::Utf8Path {
        camino::Utf8Path::new(match self {
            Source::BGM => "BGM",
            Source::BGS => "BGS",
            Source::ME => "ME",
            Source::SE => "SE",
        })
    }
}
